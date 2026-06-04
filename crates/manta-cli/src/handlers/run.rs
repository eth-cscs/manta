//! Routes `manta run *` subcommands to their exec functions.

use crate::commands::apply::session as apply_session;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta run` subcommands.
pub async fn handle_run(
  cli_run: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_run.subcommand() {
    Some(("session", m)) => apply_session::exec(m, ctx, &token).await?,
    Some((other, _)) => bail!("Unknown 'run' subcommand: {other}"),
    None => bail!("No 'run' subcommand provided"),
  }
  Ok(())
}
