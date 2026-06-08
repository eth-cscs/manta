//! `manta power` — argument types, dispatch, and poll loop for the
//! on/off/reset subcommands, targeting nodes or groups.
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

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

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

  /// Lowercase string form used by the server's `POST /power`
  /// `action` field. Distinct from [`Self::to_wire`] which produces
  /// the typed [`http_client::PowerAction`] enum for typed request
  /// bodies; this `&str` variant is used by the polling status
  /// renderer where a typed enum would be needlessly heavy.
  fn wire(self) -> &'static str {
    match self {
      PowerAction::On => "on",
      PowerAction::Off => "off",
      PowerAction::Reset => "reset",
    }
  }

  /// Convert into the typed wire enum sent in the `POST /power`
  /// request body.
  fn to_wire(self) -> crate::http_client::PowerAction {
    match self {
      PowerAction::On => crate::http_client::PowerAction::On,
      PowerAction::Off => crate::http_client::PowerAction::Off,
      PowerAction::Reset => crate::http_client::PowerAction::Reset,
    }
  }
}

/// Options shared by `exec_nodes` and `exec_cluster`.
pub struct PowerOpts<'a> {
  pub action: PowerAction,
  pub target: &'a str,
  pub force: bool,
  pub no_wait: bool,
  pub assume_yes: bool,
  pub output: &'a str,
}

/// Execute a power action against a list of nodes resolved
/// from a hosts expression.
pub async fn exec_nodes(
  ctx: &AppContext<'_>,
  token: &str,
  opts: PowerOpts<'_>,
) -> Result<(), Error> {
  // Interactive context printed before the confirm prompt; intentionally
  // plain stdout so it doesn't get wrapped in a JSON envelope.
  println!("Nodes expression: {}", opts.target);
  if !common::confirm::confirm(opts.action.confirmation_text(), opts.assume_yes)
  {
    bail!("Operation cancelled by user");
  }
  dispatch_and_wait(
    ctx,
    token,
    &opts,
    crate::http_client::PowerTargetType::Nodes,
  )
  .await
}

/// Execute a power action against all nodes in an HSM group.
pub async fn exec_cluster(
  ctx: &AppContext<'_>,
  token: &str,
  opts: PowerOpts<'_>,
) -> Result<(), Error> {
  // Interactive context printed before the confirm prompt; intentionally
  // plain stdout so it doesn't get wrapped in a JSON envelope.
  println!("Group: {}", opts.target);
  if !common::confirm::confirm(opts.action.confirmation_text(), opts.assume_yes)
  {
    bail!("Operation cancelled by user");
  }
  dispatch_and_wait(
    ctx,
    token,
    &opts,
    crate::http_client::PowerTargetType::Cluster,
  )
  .await
}

/// POST `/power` to start the transition, then (unless `no_wait`)
/// poll `GET /power/transitions/{id}` until the transition reports
/// `completed`. Renders a one-line progress summary on every poll,
/// prints a final summary, and exits non-zero if any task failed.
async fn dispatch_and_wait(
  ctx: &AppContext<'_>,
  token: &str,
  opts: &PowerOpts<'_>,
  target_type: crate::http_client::PowerTargetType,
) -> Result<(), Error> {
  let action_str = opts.action.wire();
  let client = MantaClient::new(ctx.manta_server_url, ctx.site_name)?;

  let req = crate::http_client::PowerRequest {
    action: opts.action.to_wire(),
    host_expression: opts.target.to_string(),
    target_type,
    force: opts.force,
  };
  let started = client.power(token, &req).await?;
  let transition_id = started
    .get("transitionID")
    .and_then(Value::as_str)
    .ok_or_else(|| {
      anyhow!("server response did not include a transitionID: {started}")
    })?
    .to_string();

  if opts.no_wait {
    action_result::print_with_data(
      &format!(
        "Power {action_str} transition started: {transition_id}. \
         Run `manta power transition show {transition_id}` (or re-POST without --no-wait) to follow."
      ),
      &started,
      Some(opts.output),
    )?;
    return Ok(());
  }

  let final_snapshot = poll_until_done(&client, token, &transition_id).await?;

  let failed = failed_count(&final_snapshot);
  let message = if failed > 0 {
    format!("Power {action_str} completed with {failed} failure(s).")
  } else {
    format!("Power {action_str} completed.")
  };
  action_result::print_with_data(&message, &final_snapshot, Some(opts.output))?;
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
    tracing::info!(
      "{}",
      progress_summary(&snapshot, attempt, MAX_POLL_ATTEMPTS)
    );

    if is_complete(&snapshot) {
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

/// `true` when the PCS snapshot reports `transitionStatus =
/// "completed"`. Termination predicate for the CLI poll loop.
fn is_complete(snapshot: &Value) -> bool {
  snapshot
    .get("transitionStatus")
    .and_then(Value::as_str)
    .is_some_and(|s| s == "completed")
}

/// Number of failed sub-tasks in the snapshot. Drives the
/// exit-code logic: any failure → non-zero exit.
fn failed_count(snapshot: &Value) -> u64 {
  snapshot
    .get("taskCounts")
    .and_then(|c| c.get("failed"))
    .and_then(Value::as_u64)
    .unwrap_or(0)
}

/// One-line progress summary rendered on every poll. Matches the
/// wording csm-rs used to log so operator muscle-memory carries
/// over. Field names are PCS-style (`in_progress` is the snake-case
/// form the manta server re-serializes; csm-rs upstream uses
/// `in-progress` but that's not what the CLI sees).
fn progress_summary(
  snapshot: &Value,
  attempt: usize,
  max_attempts: usize,
) -> String {
  let status = snapshot
    .get("transitionStatus")
    .and_then(Value::as_str)
    .unwrap_or("unknown");
  let operation = snapshot
    .get("operation")
    .and_then(Value::as_str)
    .unwrap_or("?");
  let counts = snapshot.get("taskCounts").cloned().unwrap_or(Value::Null);
  let count_u64 = |k: &str| counts.get(k).and_then(Value::as_u64).unwrap_or(0);

  format!(
    "Power '{}' progress (attempt {}/{}) — status: {}, failed: {}, in-progress: {}, succeeded: {}, total: {}",
    operation,
    attempt,
    max_attempts,
    status,
    count_u64("failed"),
    count_u64("in_progress"),
    count_u64("succeeded"),
    count_u64("total"),
  )
}

#[cfg(test)]
mod tests {
  //! Pure-logic locks for the JSON paths the poll loop reads.
  //! Catches accidental rename of `transitionStatus` / `taskCounts.*`
  //! either in the manta-backend-dispatcher wire types or in the
  //! server's pass-through.

  use super::{failed_count, is_complete, progress_summary};
  use serde_json::json;

  #[test]
  fn is_complete_true_only_for_completed_status() {
    assert!(is_complete(&json!({ "transitionStatus": "completed" })));
    assert!(!is_complete(&json!({ "transitionStatus": "in-progress" })));
    assert!(!is_complete(&json!({ "transitionStatus": "new" })));
    assert!(!is_complete(&json!({})));
    assert!(!is_complete(&json!({ "transitionStatus": 42 })));
  }

  #[test]
  fn failed_count_extracts_task_counts_failed() {
    let snap = json!({
      "taskCounts": { "failed": 3, "succeeded": 10, "total": 13 }
    });
    assert_eq!(failed_count(&snap), 3);
  }

  #[test]
  fn failed_count_defaults_to_zero_on_missing_fields() {
    assert_eq!(failed_count(&json!({})), 0);
    assert_eq!(failed_count(&json!({ "taskCounts": {} })), 0);
    assert_eq!(
      failed_count(&json!({ "taskCounts": { "failed": "not-a-number" } })),
      0
    );
  }

  #[test]
  fn progress_summary_renders_pcs_fields() {
    let snap = json!({
      "transitionStatus": "in-progress",
      "operation": "Reset",
      "taskCounts": {
        "total": 17, "failed": 0, "in_progress": 5, "succeeded": 12,
      }
    });
    let line = progress_summary(&snap, 7, 300);
    assert!(line.contains("Reset"), "operation missing: {line}");
    assert!(line.contains("attempt 7/300"), "attempt missing: {line}");
    assert!(
      line.contains("status: in-progress"),
      "status missing: {line}"
    );
    assert!(line.contains("failed: 0"), "failed missing: {line}");
    assert!(
      line.contains("in-progress: 5"),
      "in-progress missing: {line}"
    );
    assert!(line.contains("succeeded: 12"), "succeeded missing: {line}");
    assert!(line.contains("total: 17"), "total missing: {line}");
  }

  /// Defensive: a snapshot with no `taskCounts` shouldn't panic the
  /// renderer; it should fall back to zeros (which is what an
  /// operator sees on a fresh transition).
  #[test]
  fn progress_summary_tolerates_missing_task_counts() {
    let snap = json!({
      "transitionStatus": "new",
      "operation": "On",
    });
    let line = progress_summary(&snap, 1, 300);
    assert!(line.contains("status: new"));
    assert!(line.contains("failed: 0"));
    assert!(line.contains("total: 0"));
  }
}
