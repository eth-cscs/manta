//! Routes `manta console *` subcommands to their exec functions.

use crate::cli::commands::console_common;
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use crate::cli::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::cli::common::app_context::AppContext;
use std::io::IsTerminal;

/// Dispatch `manta console` subcommands (node, target-ansible).
pub async fn handle_console(
  cli_console: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_console.subcommand() {
    Some(("node", m)) => {
      if !std::io::stdout().is_terminal() {
        bail!("This command needs to run in interactive mode");
      }
      let xname = m.req_str("XNAME")?;

      let server_url = ctx.manta_server_url;
      let (cols, rows) = crossterm::terminal::size()?;
      let (a_input, a_output) = MantaClient::new(server_url, ctx.site_name)?
        .console_node(&token, xname, cols, rows)
        .await?;
      let result = console_common::run_console_loop(a_input, a_output).await;
      console_common::handle_console_result(result);
    }
    Some(("target-ansible", m)) => {
      if !std::io::stdout().is_terminal() {
        bail!("This command needs to run in interactive mode");
      }
      let session_name = m.req_str("SESSION_NAME")?;

      let server_url = ctx.manta_server_url;
      let (cols, rows) = crossterm::terminal::size()?;
      let (a_input, a_output) = MantaClient::new(server_url, ctx.site_name)?
        .console_session(&token, session_name, cols, rows)
        .await?;
      let result = console_common::run_console_loop(a_input, a_output).await;
      console_common::handle_console_result(result);
    }
    Some((other, _)) => bail!("Unknown 'console' subcommand: {other}"),
    None => bail!("No 'console' subcommand provided"),
  }
  Ok(())
}
