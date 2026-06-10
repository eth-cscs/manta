//! Per-image apply pipeline: drives the CLI-side
//! create-session → monitor → stamp sequence the user sees when
//! `manta apply sat-file` reaches an `images[]` entry.
//!
//! The server's monolithic `POST /sat-file/images` exists for callers
//! that prefer one round-trip per image; manta-cli takes the longer
//! road so the operator can observe (`--watch-logs`) or just poll
//! status as the CFS session runs.
//!
//! Step-by-step:
//! 1. `POST /sat-file/images/cfs-session` — the server translates the
//!    SAT image entry into a CFS session payload and creates it,
//!    returning the (still-running) `CfsSessionGetResponse`.
//! 2. Monitor the session.
//!    - `--watch-logs` streams `GET /sessions/{name}/logs` (SSE) and,
//!      when the stream ends, polls the session once to surface its
//!      terminal status.
//!    - Otherwise, polls `GET /sessions?name=…` until the status is
//!      terminal (`complete` / `succeeded` ⇒ success, anything with
//!      `fail` ⇒ error).
//! 3. `POST /sat-file/images/stamp` — server fetches the now-complete
//!    session, derives `manta.image_session.{base,groups,configuration}`
//!    from it, and PATCHes the produced IMS image. Returns the stamped
//!    `Image`.
//!
//! Dry-run short-circuits after step 1: the server returns a mocked
//! complete session with a `DRYRUN-…` `result_id`; the pipeline
//! synthesises the matching `Image` JSON locally and returns. No
//! monitor poll and no stamp PATCH are attempted because both would
//! reach against a CFS session and IMS image that do not exist.

use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use anyhow::{Context, bail};
use serde_json::Value;
use tokio::io::AsyncBufReadExt as _;

use manta_shared::types::dto::CfsSessionGetResponse;

use super::exec::SatApplyOptions;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::{
  CreateImageCfsSessionRequest, StampImageFromSessionRequest,
};

/// Poll interval for the no-`--watch-logs` monitor branch. Long enough
/// that we don't hammer the server (CFS sessions take minutes to
/// hours), short enough that a fast-failing session surfaces promptly.
const POLL_INTERVAL: Duration = Duration::from_secs(10);

/// Hard cap on the monitor loop. Image builds top out around 1-2 hours;
/// a 4-hour budget covers worst-case CFS jobs while still preventing an
/// unattended CLI from spinning forever on a stuck pod. On expiry the
/// CLI bails with a clear pointer to manual investigation — the
/// underlying session is not cancelled.
const POLL_BUDGET: Duration = Duration::from_secs(4 * 60 * 60);

/// Cap on consecutive "session not yet visible" responses. CFS normally
/// surfaces a newly-created session within a few seconds; sitting at
/// `None` for minutes means the create call landed somewhere we can't
/// see (server-side error, backend filter mismatch, …). Bail rather
/// than mask the gap.
const NOT_VISIBLE_BUDGET: Duration = Duration::from_secs(5 * 60);

/// Run the create-session → monitor → stamp pipeline for one SAT
/// `images[]` entry. Returns the resulting `Image` as `serde_json::Value`
/// so the caller (`dispatch_plan`) can drop it into the `images: [...]`
/// summary list unchanged.
pub async fn run_image_pipeline(
  client: &MantaClient,
  image: &Value,
  ref_lookup: &HashMap<String, String>,
  opts: &SatApplyOptions<'_>,
) -> anyhow::Result<Value> {
  // 1. Translate the SAT image entry into a CFS session and create it.
  let session_value = client
    .openapi
    .post_sat_image_cfs_session(
      client.site_name(),
      &CreateImageCfsSessionRequest {
        image: image.clone(),
        ref_lookup: ref_lookup.clone(),
        ansible_verbosity: opts.ansible_verbosity_opt.map(i32::from),
        ansible_passthrough: opts.ansible_passthrough_opt.map(str::to_string),
        dry_run: Some(opts.dry_run),
      },
    )
    .await
    .into_anyhow()
    .context("create CFS session from SAT image entry")?;

  // Server returns the freshly-created session as JSON. Round-trip
  // into manta-shared's typed shape so `.status()` /
  // `.get_first_result_id()` (helpers on the dto type) stay
  // available without rebuilding them on the generated client.
  let session: CfsSessionGetResponse =
    serde_json::from_value(session_value.clone())
      .context("deserialise CFS session response")?;

  let session_name = session.name.clone();
  let image_name = image
    .get("name")
    .and_then(Value::as_str)
    .unwrap_or("<unnamed>");
  tracing::info!(
    "CFS session '{session_name}' created for SAT image '{image_name}'"
  );

  if opts.dry_run {
    // Server returned a mocked complete session with a synthetic
    // result_id (e.g. `DRYRUN-<uuid>`). Synthesise the matching Image
    // locally so `dispatch_plan`'s ref_lookup accumulation still works.
    let id = session
      .get_first_result_id()
      .unwrap_or_else(|| format!("DRYRUN-{session_name}"));
    let image_name = image
      .get("name")
      .and_then(Value::as_str)
      .unwrap_or_default()
      .to_string();
    return Ok(serde_json::json!({ "id": id, "name": image_name }));
  }

  // 2. Monitor the session until it reports terminal status.
  if opts.watch_logs {
    stream_session_until_terminal(client, &session_name, opts.timestamps)
      .await?;
  } else {
    poll_session_until_terminal(client, &session_name).await?;
  }

  // 3. Stamp + PATCH the produced IMS image (server does the fetch +
  //    derive + PATCH).
  let stamped = client
    .openapi
    .post_sat_image_stamp(
      client.site_name(),
      &StampImageFromSessionRequest {
        cfs_session_name: session_name.clone(),
      },
    )
    .await
    .into_anyhow()
    .with_context(|| {
      format!("stamp image from CFS session '{session_name}'")
    })?;

  Ok(stamped)
}

/// `--watch-logs` branch: stream the SSE log feed to stdout until the
/// stream ends, then poll until the session reports terminal status.
/// The poll fallback covers the race between the CFS pod's log
/// channel closing and the session resource flipping to `complete` —
/// without it we would sometimes call the stamp endpoint against a
/// still-running session and 400 on the missing `result_id`.
async fn stream_session_until_terminal(
  client: &MantaClient,
  session_name: &str,
  timestamps: bool,
) -> anyhow::Result<()> {
  tracing::info!("Streaming logs for CFS session '{session_name}' ...");
  let reader = client
    .stream_session_logs(session_name, timestamps)
    .await
    .with_context(|| {
      format!("open SSE log stream for CFS session '{session_name}'")
    })?;
  let mut lines = reader.lines();
  while let Some(raw) = lines
    .next_line()
    .await
    .context("read CFS session log stream")?
  {
    if let Some(content) = raw.strip_prefix("data: ") {
      println!("{content}");
    }
  }

  poll_session_until_terminal(client, session_name).await
}

/// Default branch: poll session status until terminal.
async fn poll_session_until_terminal(
  client: &MantaClient,
  session_name: &str,
) -> anyhow::Result<()> {
  tracing::info!(
    "Polling CFS session '{session_name}' until it reaches terminal status \
     (poll interval: {}s, hard cap: {} h)",
    POLL_INTERVAL.as_secs(),
    POLL_BUDGET.as_secs() / 3600,
  );
  let start = Instant::now();
  let mut first_not_visible_at: Option<Instant> = None;
  loop {
    if start.elapsed() > POLL_BUDGET {
      bail!(
        "CFS session '{session_name}' did not reach terminal status \
         within {} h; aborting monitor. The session may still be running — \
         inspect it directly with `manta get sessions --name {session_name}`.",
        POLL_BUDGET.as_secs() / 3600,
      );
    }
    match fetch_session_opt(client, session_name).await? {
      None => {
        // Session may still be creating in CFS; retry until we cross
        // NOT_VISIBLE_BUDGET, then bail — a session that never appears
        // is almost certainly a server-side or backend-filter issue.
        let stuck_for = first_not_visible_at
          .get_or_insert_with(Instant::now)
          .elapsed();
        if stuck_for > NOT_VISIBLE_BUDGET {
          bail!(
            "CFS session '{session_name}' was never visible after {} min of \
             polling. The create call returned a session name we can't \
             fetch back — check the manta-server log for backend errors.",
            NOT_VISIBLE_BUDGET.as_secs() / 60,
          );
        }
        tokio::time::sleep(POLL_INTERVAL).await;
      }
      Some(session) => {
        first_not_visible_at = None;
        match session.status().as_deref() {
          Some("complete") | Some("succeeded") | Some("success") => {
            tracing::info!("CFS session '{session_name}' complete");
            return Ok(());
          }
          Some(s) if s.contains("fail") => {
            bail!("CFS session '{session_name}' failed (status: '{s}')");
          }
          Some(s) => {
            tracing::debug!(
              "CFS session '{session_name}' still running (status: '{s}')",
            );
            tokio::time::sleep(POLL_INTERVAL).await;
          }
          None => {
            tracing::debug!("CFS session '{session_name}' has no status yet",);
            tokio::time::sleep(POLL_INTERVAL).await;
          }
        }
      }
    }
  }
}

/// Pull the session by name. `None` is "not yet visible" (poll path
/// retries); the caller decides whether to error or wait.
async fn fetch_session_opt(
  client: &MantaClient,
  session_name: &str,
) -> anyhow::Result<Option<CfsSessionGetResponse>> {
  let sessions_value = client
    .openapi
    .get_sessions(
      None,
      None,
      None,
      None,
      Some(session_name),
      None,
      None,
      None,
      client.site_name(),
    )
    .await
    .into_anyhow()
    .with_context(|| format!("fetch CFS session '{session_name}'"))?;

  // Server returns an array under the wire shape. Round-trip into
  // the typed shape so callers keep using `.status()`.
  let sessions: Vec<CfsSessionGetResponse> =
    serde_json::from_value(sessions_value)
      .context("deserialise CFS session list response")?;
  Ok(sessions.into_iter().next())
}
