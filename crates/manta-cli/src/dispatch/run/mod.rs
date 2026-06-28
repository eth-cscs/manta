//! `manta run` subcommands.
//!
//! Today only [`session`] exists; it creates and runs a one-off CFS
//! session targeting a group or an `--ansible-limit` host list via
//! `POST /api/v1/sessions`. See [`session`] for the per-leaf
//! workflow.

pub mod session;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta run` subcommands.
///
/// # Errors
///
/// Returns an error when the auth token cannot be obtained, when no
/// subcommand is provided / the name is unknown, or when the leaf
/// handler fails.
pub async fn handle_run(
  cli_run: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_run.subcommand() {
    Some(("session", m)) => session::exec(m, ctx, &token).await?,
    Some((other, _)) => bail!("Unknown 'run' subcommand: {other}"),
    None => bail!("No 'run' subcommand provided"),
  }
  Ok(())
}
