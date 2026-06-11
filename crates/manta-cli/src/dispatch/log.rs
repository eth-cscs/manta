//! Routes the `manta log` command to its exec function.

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use anyhow::{Context, Error};
use clap::ArgMatches;
use manta_shared::types::dto::CfsSessionGetResponse;

/// Dispatch the `manta log` command to stream CFS session logs.
///
/// In server mode the session is resolved via the server's `GET /sessions`
/// endpoint; logs are then streamed as SSE from `GET /sessions/{name}/logs`.
/// In direct mode the existing k8s-backed path is used unchanged.
pub async fn handle_log(
  cli_log: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  let user_input = cli_log.req_str("VALUE")?;
  let timestamps = cli_log.get_flag("timestamps");

  use tokio::io::AsyncBufReadExt as _;
  let client = MantaClient::from_app_ctx(ctx, Some(&token))?;

  // Try user input as a session name first, then as an xname. The
  // generated `get_sessions` returns `serde_json::Value`; we round-trip
  // into the typed shape so the existing `.name` field access works.
  let by_name = client
    .openapi
    .get_sessions(
      None,
      Some(1),
      None,
      None,
      Some(user_input),
      None,
      None,
      None,
      client.site_name(),
    )
    .await
    .into_anyhow()
    .ok()
    .and_then(|v| serde_json::from_value::<Vec<CfsSessionGetResponse>>(v).ok());

  let session_name = if let Some(first) =
    by_name.as_ref().and_then(|sessions| sessions.first())
  {
    first.name.clone()
  } else {
    let raw = client
      .openapi
      .get_sessions(
        None,
        Some(1),
        None,
        None,
        None,
        None,
        None,
        Some(user_input),
        client.site_name(),
      )
      .await
      .into_anyhow()
      .context("Failed to query CFS sessions by xname")?;
    let by_xname: Vec<CfsSessionGetResponse> = serde_json::from_value(raw)
      .context("Failed to deserialize CFS sessions list")?;
    by_xname
      .into_iter()
      .next()
      .context(format!("No CFS session found for '{user_input}'"))?
      .name
      .clone()
  };

  let reader = client
    .stream_session_logs(&session_name, timestamps)
    .await
    .context("Failed to get CFS session log stream from server")?;

  let mut lines = reader.lines();
  while let Some(raw) = lines
    .next_line()
    .await
    .context("Failed to read CFS session log stream")?
  {
    if let Some(content) = raw.strip_prefix("data: ") {
      println!("{content}");
    }
  }
  println!("Log streaming ended");
  Ok(())
}
