//! Routes the `manta log` command to its exec function.

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::shared::params::session::GetSessionParams;
use anyhow::{Context, Error};
use clap::ArgMatches;

/// Dispatch the `manta log` command to stream CFS session logs.
///
/// In server mode the session is resolved via the server's `GET /sessions`
/// endpoint; logs are then streamed as SSE from `GET /sessions/{name}/logs`.
/// In direct mode the existing k8s-backed path is used unchanged.
pub async fn handle_log(
  cli_log: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  let user_input = cli_log
    .get_one::<String>("VALUE")
    .context("The 'VALUE' argument is mandatory")?;
  let timestamps = cli_log.get_flag("timestamps");

  use tokio::io::AsyncBufReadExt as _;
  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  let client = MantaClient::new(server_url, ctx.infra.site_name)?;

  // Try user input as a session name first, then as an xname.
  let sessions_rslt = client
    .get_sessions(
      &token,
      &GetSessionParams {
        name: Some(user_input.clone()),
        xnames: vec![],
        hsm_group: None,
        min_age: None,
        max_age: None,
        session_type: None,
        status: None,
        limit: Some(1),
      },
    )
    .await;

  let session_name = if let Ok([s, ..]) = sessions_rslt.as_deref() {
    s.name.clone()
  } else {
    let by_xname = client
      .get_sessions(
        &token,
        &GetSessionParams {
          name: None,
          xnames: vec![user_input.clone()],
          hsm_group: None,
          min_age: None,
          max_age: None,
          session_type: None,
          status: None,
          limit: Some(1),
        },
      )
      .await
      .context("Failed to query CFS sessions by xname")?;
    by_xname
      .into_iter()
      .next()
      .context(format!("No CFS session found for '{}'", user_input))?
      .name
      .clone()
  };

  let reader = client
    .stream_session_logs(&token, &session_name, timestamps)
    .await
    .context("Failed to get CFS session log stream from server")?;

  let mut lines = reader.lines();
  while let Some(raw) = lines
    .next_line()
    .await
    .context("Failed to read CFS session log stream")?
  {
    if let Some(content) = raw.strip_prefix("data: ") {
      println!("{}", content);
    }
  }
  println!("Log streaming ended");
  Ok(())
}
