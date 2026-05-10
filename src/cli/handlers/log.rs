//! Routes the `manta log` command to its exec function.

use crate::cli::commands;
use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::service::session::GetSessionParams;
use anyhow::{Context, Error, bail};
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

  if let Some(server_url) = ctx.infra.manta_server_url {
    use tokio::io::AsyncBufReadExt as _;

    let client = MantaClient::new(server_url, ctx.infra.site_name)?;

    // Try user input as a session name first, then as an xname.
    let sessions = client
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
      .await
      .context("Failed to query CFS sessions")?;

    let session_name = if let Some(s) = sessions.into_iter().next() {
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
    return Ok(());
  }

  let site = ctx
    .cli
    .configuration
    .sites
    .get(&ctx.cli.configuration.site)
    .context("Site not found in configuration")?;
  let k8s_details = site
    .k8s
    .as_ref()
    .context("k8s section not found in configuration")?;
  match commands::log::exec(
    ctx.infra.backend,
    ctx.infra.site_name,
    &token,
    ctx.infra.shasta_base_url,
    ctx.infra.shasta_root_cert,
    user_input,
    timestamps,
    k8s_details,
  )
  .await
  {
    Ok(_) => {
      println!("Log streaming ended");
      Ok(())
    }
    Err(e) => bail!("{e}"),
  }
}
