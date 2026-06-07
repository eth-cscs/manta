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

use std::{collections::HashMap, time::Duration};

use anyhow::{Context, bail};
use serde_json::Value;
use tokio::io::AsyncBufReadExt as _;

use manta_shared::types::dto::CfsSessionGetResponse;
use manta_shared::types::params::session::GetSessionParams;

use super::exec::SatApplyOptions;
use crate::http_client::{CreateImageCfsSessionRequest, MantaClient};

/// Poll interval for the no-`--watch-logs` monitor branch. Long enough
/// that we don't hammer the server (CFS sessions take minutes to
/// hours), short enough that a fast-failing session surfaces promptly.
const POLL_INTERVAL: Duration = Duration::from_secs(10);

/// Run the create-session → monitor → stamp pipeline for one SAT
/// `images[]` entry. Returns the resulting `Image` as `serde_json::Value`
/// so the caller (`dispatch_plan`) can drop it into the `images: [...]`
/// summary list unchanged.
pub async fn run_image_pipeline(
  client: &MantaClient,
  token: &str,
  image: &Value,
  ref_lookup: &HashMap<String, String>,
  opts: &SatApplyOptions<'_>,
) -> anyhow::Result<Value> {
  // 1. Translate the SAT image entry into a CFS session and create it.
  let session = client
    .create_image_cfs_session(
      token,
      &CreateImageCfsSessionRequest {
        image,
        ref_lookup,
        ansible_verbosity: opts.ansible_verbosity_opt,
        ansible_passthrough: opts.ansible_passthrough_opt,
        dry_run: opts.dry_run,
      },
    )
    .await
    .context("create CFS session from SAT image entry")?;

  let session_name = session.name.clone();
  let image_name =
    image.get("name").and_then(Value::as_str).unwrap_or("<unnamed>");
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
    stream_session_until_terminal(
      client,
      token,
      &session_name,
      opts.timestamps,
    )
    .await?;
  } else {
    poll_session_until_terminal(client, token, &session_name).await?;
  }

  // 3. Stamp + PATCH the produced IMS image (server does the fetch +
  //    derive + PATCH).
  let stamped = client
    .stamp_image_from_session(token, &session_name)
    .await
    .with_context(|| {
      format!("stamp image from CFS session '{session_name}'")
    })?;

  serde_json::to_value(&stamped).context("serialise stamped image")
}

/// `--watch-logs` branch: stream the SSE log feed to stdout until the
/// stream ends, then poll the session once to confirm its terminal
/// status and surface a `failed`-state error.
async fn stream_session_until_terminal(
  client: &MantaClient,
  token: &str,
  session_name: &str,
  timestamps: bool,
) -> anyhow::Result<()> {
  tracing::info!("Streaming logs for CFS session '{session_name}' ...");
  let reader = client
    .stream_session_logs(token, session_name, timestamps)
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

  let session = fetch_session(client, token, session_name).await?;
  check_terminal_status(&session, session_name)?;
  Ok(())
}

/// Default branch: poll session status until terminal.
async fn poll_session_until_terminal(
  client: &MantaClient,
  token: &str,
  session_name: &str,
) -> anyhow::Result<()> {
  tracing::info!(
    "Polling CFS session '{session_name}' until it reaches terminal status \
     (poll interval: {}s)",
    POLL_INTERVAL.as_secs(),
  );
  loop {
    match fetch_session_opt(client, token, session_name).await? {
      None => {
        // Session may still be creating in CFS; retry shortly.
        tokio::time::sleep(POLL_INTERVAL).await;
        continue;
      }
      Some(session) => match session.status().as_deref() {
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
          tracing::debug!(
            "CFS session '{session_name}' has no status yet",
          );
          tokio::time::sleep(POLL_INTERVAL).await;
        }
      },
    }
  }
}

/// Pull the session by name; treat "not found" as an error (the
/// session_name came from the just-created session, so missing means
/// something went badly wrong).
async fn fetch_session(
  client: &MantaClient,
  token: &str,
  session_name: &str,
) -> anyhow::Result<CfsSessionGetResponse> {
  fetch_session_opt(client, token, session_name)
    .await?
    .with_context(|| {
      format!("CFS session '{session_name}' disappeared from CFS")
    })
}

/// Pull the session by name. `None` is "not yet visible" (poll path
/// retries); the caller decides whether to error or wait.
async fn fetch_session_opt(
  client: &MantaClient,
  token: &str,
  session_name: &str,
) -> anyhow::Result<Option<CfsSessionGetResponse>> {
  let params = GetSessionParams {
    hsm_group: None,
    xnames: Vec::new(),
    min_age: None,
    max_age: None,
    session_type: None,
    status: None,
    name: Some(session_name.to_string()),
    limit: None,
  };
  let sessions = client
    .get_sessions(token, &params)
    .await
    .with_context(|| format!("fetch CFS session '{session_name}'"))?;
  Ok(sessions.into_iter().next())
}

/// Surface a terminal-failed session as an error.
fn check_terminal_status(
  session: &CfsSessionGetResponse,
  session_name: &str,
) -> anyhow::Result<()> {
  let status = session
    .status()
    .with_context(|| format!("CFS session '{session_name}' has no status"))?;
  match status.as_str() {
    "complete" | "succeeded" | "success" => Ok(()),
    other => bail!(
      "CFS session '{session_name}' did not complete successfully \
       (status: '{other}')"
    ),
  }
}
