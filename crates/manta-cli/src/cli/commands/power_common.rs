//! Shared argument types and dispatch logic for power management
//! commands.
//!
//! Both `exec_nodes` and `exec_cluster` reduce to a POST + poll loop
//! against the manta server. The server's `POST /power` returns
//! immediately with the PCS `transitionID`; the CLI then snapshots
//! the transition via `GET /power/transitions/{id}` every few seconds
//! until it reports `completed`. `--no-wait` short-circuits the loop,
//! returning the transition id for the operator to follow up on
//! manually.

use std::{fmt, time::Duration};

use anyhow::{Error, anyhow, bail};
use serde_json::Value;

use crate::cli::common;
use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::app_context::AppContext;

/// How long the CLI sleeps between snapshot polls. Matches the
/// historical csm-rs `pcs_transitions_wait_to_complete` interval.
const POLL_INTERVAL: Duration = Duration::from_secs(3);
/// Hard cap on poll attempts — 300 × 3s = 15 minutes. Matches the
/// historical csm-rs cap; operators with longer transitions should
/// re-run `manta power transition show <id>` (or live with the
/// `--no-wait` flow) rather than tune this here.
const MAX_POLL_ATTEMPTS: usize = 300;

/// The three power operations supported by the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
  /// Power nodes on.
  On,
  /// Power nodes off.
  Off,
  /// Power-cycle (reset) nodes.
  Reset,
}

impl fmt::Display for PowerAction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PowerAction::On => write!(f, "power on"),
      PowerAction::Off => write!(f, "power off"),
      PowerAction::Reset => write!(f, "power reset"),
    }
  }
}

impl PowerAction {
  /// Human-readable confirmation prompt fragment.
  fn confirmation_text(&self) -> &'static str {
    match self {
      PowerAction::On => {
        "The nodes above will be powered on. \
         Please confirm to proceed?"
      }
      PowerAction::Off => {
        "The nodes above will be powered off. \
         Please confirm to proceed?"
      }
      PowerAction::Reset => {
        "The nodes above will restart. \
         Please confirm to proceed?"
      }
    }
  }

  fn wire(self) -> &'static str {
    match self {
      PowerAction::On => "on",
      PowerAction::Off => "off",
      PowerAction::Reset => "reset",
    }
  }
}

/// Execute a power action against a list of nodes resolved
/// from a hosts expression.
#[allow(clippy::too_many_arguments)]
pub async fn exec_nodes(
  ctx: &AppContext<'_>,
  action: PowerAction,
  hosts_expression: &str,
  force: bool,
  no_wait: bool,
  assume_yes: bool,
  output: &str,
  token: &str,
) -> Result<(), Error> {
  // Interactive context printed before the confirm prompt; intentionally
  // plain stdout so it doesn't get wrapped in a JSON envelope.
  println!("Nodes expression: {hosts_expression}");
  if !common::user_interaction::confirm(action.confirmation_text(), assume_yes)
  {
    bail!("Operation cancelled by user");
  }
  dispatch_and_wait(
    ctx,
    token,
    action,
    hosts_expression,
    "nodes",
    force,
    no_wait,
    output,
  )
  .await
}

/// Execute a power action against all nodes in an HSM group
/// (cluster).
#[allow(clippy::too_many_arguments)]
pub async fn exec_cluster(
  ctx: &AppContext<'_>,
  action: PowerAction,
  hsm_group_name_arg: &str,
  force: bool,
  no_wait: bool,
  assume_yes: bool,
  output: &str,
  token: &str,
) -> Result<(), Error> {
  // Interactive context printed before the confirm prompt; intentionally
  // plain stdout so it doesn't get wrapped in a JSON envelope.
  println!("Cluster: {hsm_group_name_arg}");
  if !common::user_interaction::confirm(action.confirmation_text(), assume_yes)
  {
    bail!("Operation cancelled by user");
  }
  dispatch_and_wait(
    ctx,
    token,
    action,
    hsm_group_name_arg,
    "cluster",
    force,
    no_wait,
    output,
  )
  .await
}

/// POST `/power` to start the transition, then (unless `no_wait`)
/// poll `GET /power/transitions/{id}` until the transition reports
/// `completed`. Renders a one-line progress summary on every poll,
/// prints a final summary, and exits non-zero if any task failed.
#[allow(clippy::too_many_arguments)]
async fn dispatch_and_wait(
  ctx: &AppContext<'_>,
  token: &str,
  action: PowerAction,
  targets_expression: &str,
  target_type: &str,
  force: bool,
  no_wait: bool,
  output: &str,
) -> Result<(), Error> {
  let action_str = action.wire();
  let client = MantaClient::new(ctx.manta_server_url, ctx.site_name)?;

  let started = client
    .power(token, action_str, targets_expression, target_type, force)
    .await?;
  let transition_id = started
    .get("transitionID")
    .and_then(Value::as_str)
    .ok_or_else(|| {
      anyhow!("server response did not include a transitionID: {started}")
    })?
    .to_string();

  if no_wait {
    action_result::print_with_data(
      &format!(
        "Power {action_str} transition started: {transition_id}. \
         Run `manta power transition show {transition_id}` (or re-POST without --no-wait) to follow."
      ),
      &started,
      Some(output),
    )?;
    return Ok(());
  }

  let final_snapshot = poll_until_done(&client, token, &transition_id).await?;

  let failed = final_snapshot
    .get("taskCounts")
    .and_then(|c| c.get("failed"))
    .and_then(Value::as_u64)
    .unwrap_or(0);
  let message = if failed > 0 {
    format!("Power {action_str} completed with {failed} failure(s).")
  } else {
    format!("Power {action_str} completed.")
  };
  action_result::print_with_data(&message, &final_snapshot, Some(output))?;
  if failed > 0 {
    bail!("power transition reported {failed} failed task(s)");
  }
  Ok(())
}

/// Snapshot the transition every [`POLL_INTERVAL`] until it reaches
/// `transitionStatus == "completed"` or [`MAX_POLL_ATTEMPTS`] runs
/// out. Each poll logs a single progress line; the final snapshot is
/// returned to the caller for the summary print.
async fn poll_until_done(
  client: &MantaClient,
  token: &str,
  transition_id: &str,
) -> Result<Value, Error> {
  let mut snapshot = client.power_transition(token, transition_id).await?;

  for attempt in 1..=MAX_POLL_ATTEMPTS {
    let status = snapshot
      .get("transitionStatus")
      .and_then(Value::as_str)
      .unwrap_or("unknown");
    let operation = snapshot
      .get("operation")
      .and_then(Value::as_str)
      .unwrap_or("?");
    let counts = snapshot
      .get("taskCounts")
      .cloned()
      .unwrap_or(Value::Null);
    let count_u64 = |k: &str| counts.get(k).and_then(Value::as_u64).unwrap_or(0);

    tracing::info!(
      "Power '{}' progress (attempt {}/{}) — status: {}, failed: {}, in-progress: {}, succeeded: {}, total: {}",
      operation,
      attempt,
      MAX_POLL_ATTEMPTS,
      status,
      count_u64("failed"),
      count_u64("in_progress"),
      count_u64("succeeded"),
      count_u64("total"),
    );

    if status == "completed" {
      return Ok(snapshot);
    }

    tokio::time::sleep(POLL_INTERVAL).await;
    snapshot = client.power_transition(token, transition_id).await?;
  }

  bail!(
    "power transition {transition_id} did not complete after {MAX_POLL_ATTEMPTS} poll attempts \
     (interval {:?}); re-run `manta power transition show {transition_id}` to check later",
    POLL_INTERVAL
  )
}
