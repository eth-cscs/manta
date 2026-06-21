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

use anyhow::{Context, Error, anyhow, bail};
use clap::ArgMatches;
use serde_json::Value;

use crate::common;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::{PowerRequest, PowerTargetType};
use crate::output::action_result;

/// Default seconds between snapshot polls when `cli.toml` does not
/// set `power_poll_interval_secs`. Matches the historical csm-rs
/// `pcs_transitions_wait_to_complete` interval.
pub const DEFAULT_POWER_POLL_INTERVAL_SECS: u64 = 3;
/// Default cap on poll attempts when `cli.toml` does not set
/// `power_max_poll_attempts`. 300 × 3 s = 15 minutes total.
/// Operators with longer transitions should bump
/// `power_max_poll_attempts` rather than re-run `manta power
/// transition show <id>` by hand.
pub const DEFAULT_POWER_MAX_POLL_ATTEMPTS: u32 = 300;

/// Dispatch a `power on group` invocation.
async fn dispatch_power_on_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  exec_cluster(
    ctx,
    token,
    PowerOpts {
      action: PowerAction::On,
      target: m.req_str("GROUP_NAME")?,
      force: false,
      no_wait: m.get_flag("no-wait"),
      assume_yes: m.get_flag("assume-yes"),
      output: m.req_str("output")?,
      dry_run: m.get_flag("dry-run"),
    },
  )
  .await
}

/// Dispatch a `power off group` invocation.
async fn dispatch_power_off_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let graceful = m
    .get_one::<bool>("graceful")
    .context("The 'graceful' argument must have a value")?;
  exec_cluster(
    ctx,
    token,
    PowerOpts {
      action: PowerAction::Off,
      target: m.req_str("GROUP_NAME")?,
      force: !graceful,
      no_wait: m.get_flag("no-wait"),
      assume_yes: m.get_flag("assume-yes"),
      output: m.req_str("output")?,
      dry_run: m.get_flag("dry-run"),
    },
  )
  .await
}

/// Dispatch a `power reset group` invocation.
async fn dispatch_power_reset_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let force = m
    .get_one::<bool>("graceful")
    .context("The 'graceful' argument must have a value")?;
  exec_cluster(
    ctx,
    token,
    PowerOpts {
      action: PowerAction::Reset,
      target: m.req_str("GROUP_NAME")?,
      force: *force,
      no_wait: m.get_flag("no-wait"),
      assume_yes: m.get_flag("assume-yes"),
      output: m.req_str("output")?,
      dry_run: m.get_flag("dry-run"),
    },
  )
  .await
}

/// Dispatch `manta power` subcommands (on, off, reset —
/// each targeting nodes or groups).
pub async fn handle_power(
  cli_power: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_power.subcommand() {
    Some(("on", m)) => match m.subcommand() {
      Some(("group", m)) => dispatch_power_on_group(m, ctx, &token).await?,
      Some(("nodes", m)) => {
        exec_nodes(
          ctx,
          &token,
          PowerOpts {
            action: PowerAction::On,
            target: m.req_str("VALUE")?,
            force: false,
            no_wait: m.get_flag("no-wait"),
            assume_yes: m.get_flag("assume-yes"),
            output: m.req_str("output")?,
            dry_run: m.get_flag("dry-run"),
          },
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'power on' subcommand: {other}"),
      None => bail!("No 'power on' subcommand provided"),
    },
    Some(("off", m)) => match m.subcommand() {
      Some(("group", m)) => dispatch_power_off_group(m, ctx, &token).await?,
      Some(("nodes", m)) => {
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        exec_nodes(
          ctx,
          &token,
          PowerOpts {
            action: PowerAction::Off,
            target: m.req_str("VALUE")?,
            force: !graceful,
            no_wait: m.get_flag("no-wait"),
            assume_yes: m.get_flag("assume-yes"),
            output: m.req_str("output")?,
            dry_run: m.get_flag("dry-run"),
          },
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'power off' subcommand: {other}"),
      None => bail!("No 'power off' subcommand provided"),
    },
    Some(("reset", m)) => match m.subcommand() {
      Some(("group", m)) => dispatch_power_reset_group(m, ctx, &token).await?,
      Some(("nodes", m)) => {
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        exec_nodes(
          ctx,
          &token,
          PowerOpts {
            action: PowerAction::Reset,
            target: m.req_str("VALUE")?,
            force: !graceful,
            no_wait: m.get_flag("no-wait"),
            assume_yes: m.get_flag("assume-yes"),
            output: m.req_str("output")?,
            dry_run: m.get_flag("dry-run"),
          },
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'power reset' subcommand: {other}"),
      None => bail!("No 'power reset' subcommand provided"),
    },
    Some((other, _)) => bail!("Unknown 'power' subcommand: {other}"),
    None => bail!("No 'power' subcommand provided"),
  }
  Ok(())
}

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
  /// the typed [`crate::openapi_client::types::PowerAction`] enum
  /// for typed request bodies; this `&str` variant is used by the
  /// polling status renderer where a typed enum would be needlessly
  /// heavy.
  fn wire(self) -> &'static str {
    match self {
      PowerAction::On => "on",
      PowerAction::Off => "off",
      PowerAction::Reset => "reset",
    }
  }

  /// Convert into the typed wire enum sent in the `POST /power`
  /// request body.
  fn to_wire(self) -> crate::openapi_client::types::PowerAction {
    match self {
      PowerAction::On => crate::openapi_client::types::PowerAction::On,
      PowerAction::Off => crate::openapi_client::types::PowerAction::Off,
      PowerAction::Reset => crate::openapi_client::types::PowerAction::Reset,
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
  pub dry_run: bool,
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
  if !opts.dry_run
    && !common::confirm::confirm(opts.action.confirmation_text(), opts.assume_yes)
  {
    bail!("Operation cancelled by user");
  }
  dispatch_and_wait(ctx, token, &opts, PowerTargetType::Nodes).await
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
  if !opts.dry_run
    && !common::confirm::confirm(opts.action.confirmation_text(), opts.assume_yes)
  {
    bail!("Operation cancelled by user");
  }
  dispatch_and_wait(ctx, token, &opts, PowerTargetType::Cluster).await
}

/// POST `/power` to start the transition, then (unless `no_wait`)
/// poll `GET /power/transitions/{id}` until the transition reports
/// `completed`. Renders a one-line progress summary on every poll,
/// prints a final summary, and exits non-zero if any task failed.
async fn dispatch_and_wait(
  ctx: &AppContext<'_>,
  token: &str,
  opts: &PowerOpts<'_>,
  target_type: PowerTargetType,
) -> Result<(), Error> {
  let action_str = opts.action.wire();

  let req = PowerRequest {
    action: opts.action.to_wire(),
    host_expression: opts.target.to_string(),
    target_type,
    force: Some(opts.force),
  };

  if opts.dry_run {
    action_result::print_with_data(
      "Would POST /power:",
      &req,
      Some(opts.output),
    )?;
    return Ok(());
  }

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let started = client
    .openapi
    .post_power(client.site_name(), &req)
    .await
    .into_anyhow()?;
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

  let poll_interval = Duration::from_secs(
    ctx
      .power_poll_interval_secs
      .unwrap_or(DEFAULT_POWER_POLL_INTERVAL_SECS),
  );
  let max_attempts = ctx
    .power_max_poll_attempts
    .unwrap_or(DEFAULT_POWER_MAX_POLL_ATTEMPTS);
  let final_snapshot = poll_until_done(
    &client,
    &transition_id,
    poll_interval,
    max_attempts,
  )
  .await?;

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

/// Snapshot the transition every `poll_interval` until it reaches
/// `transitionStatus == "completed"` or `max_attempts` runs out.
/// Each poll logs a single progress line; the final snapshot is
/// returned to the caller for the summary print.
async fn poll_until_done(
  client: &MantaClient,
  transition_id: &str,
  poll_interval: Duration,
  max_attempts: u32,
) -> Result<Value, Error> {
  let max_attempts_usize = max_attempts as usize;
  let mut snapshot = client
    .openapi
    .get_power_transition(transition_id, client.site_name())
    .await
    .into_anyhow()?;

  for attempt in 1..=max_attempts_usize {
    tracing::info!(
      "{}",
      progress_summary(&snapshot, attempt, max_attempts_usize)
    );

    if is_complete(&snapshot) {
      return Ok(snapshot);
    }

    tokio::time::sleep(poll_interval).await;
    snapshot = client
      .openapi
      .get_power_transition(transition_id, client.site_name())
      .await
      .into_anyhow()?;
  }

  bail!(
    "power transition {transition_id} did not complete after {max_attempts} poll attempts \
     (interval {poll_interval:?}); re-run `manta power transition show {transition_id}` to check later",
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

  fn parse(argv: &[&str]) -> Result<clap::ArgMatches, clap::Error> {
    crate::build::build_cli().try_get_matches_from(argv)
  }

  // power on nodes ────────────────────────────────────────────
  #[test]
  fn power_on_nodes_accepts_dry_run() {
    let r = parse(&["manta", "power", "on", "nodes", "x1000", "--dry-run"]);
    assert!(r.is_ok(), "expected --dry-run on `power on nodes`: {r:?}");
  }
  #[test]
  fn power_on_nodes_accepts_dry_run_short_alias() {
    let r = parse(&["manta", "power", "on", "nodes", "x1000", "-d"]);
    assert!(r.is_ok(), "expected -d on `power on nodes`: {r:?}");
  }

  // power off nodes ───────────────────────────────────────────
  #[test]
  fn power_off_nodes_accepts_dry_run() {
    let r = parse(&["manta", "power", "off", "nodes", "x1000", "--dry-run"]);
    assert!(r.is_ok(), "expected --dry-run on `power off nodes`: {r:?}");
  }
  #[test]
  fn power_off_nodes_accepts_dry_run_short_alias() {
    let r = parse(&["manta", "power", "off", "nodes", "x1000", "-d"]);
    assert!(r.is_ok(), "expected -d on `power off nodes`: {r:?}");
  }

  // power reset nodes ─────────────────────────────────────────
  #[test]
  fn power_reset_nodes_accepts_dry_run() {
    let r = parse(&["manta", "power", "reset", "nodes", "x1000", "--dry-run"]);
    assert!(r.is_ok(), "expected --dry-run on `power reset nodes`: {r:?}");
  }
  #[test]
  fn power_reset_nodes_accepts_dry_run_short_alias() {
    let r = parse(&["manta", "power", "reset", "nodes", "x1000", "-d"]);
    assert!(r.is_ok(), "expected -d on `power reset nodes`: {r:?}");
  }

  // power on group ────────────────────────────────────────────
  #[test]
  fn power_on_group_accepts_dry_run() {
    let r = parse(&["manta", "power", "on", "group", "compute", "--dry-run"]);
    assert!(r.is_ok(), "expected --dry-run on `power on group`: {r:?}");
  }
  #[test]
  fn power_on_group_accepts_dry_run_short_alias() {
    let r = parse(&["manta", "power", "on", "group", "compute", "-d"]);
    assert!(r.is_ok(), "expected -d on `power on group`: {r:?}");
  }

  // power off group ───────────────────────────────────────────
  #[test]
  fn power_off_group_accepts_dry_run() {
    let r = parse(&["manta", "power", "off", "group", "compute", "--dry-run"]);
    assert!(r.is_ok(), "expected --dry-run on `power off group`: {r:?}");
  }
  #[test]
  fn power_off_group_accepts_dry_run_short_alias() {
    let r = parse(&["manta", "power", "off", "group", "compute", "-d"]);
    assert!(r.is_ok(), "expected -d on `power off group`: {r:?}");
  }

  // power reset group ─────────────────────────────────────────
  #[test]
  fn power_reset_group_accepts_dry_run() {
    let r = parse(&["manta", "power", "reset", "group", "compute", "--dry-run"]);
    assert!(r.is_ok(), "expected --dry-run on `power reset group`: {r:?}");
  }
  #[test]
  fn power_reset_group_accepts_dry_run_short_alias() {
    let r = parse(&["manta", "power", "reset", "group", "compute", "-d"]);
    assert!(r.is_ok(), "expected -d on `power reset group`: {r:?}");
  }

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
