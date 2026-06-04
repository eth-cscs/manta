//! Routes the `manta gen-autocomplete` command to its exec function.
//!
//! Like `manta upgrade`, this handler does NOT call `get_api_token` —
//! generating shell-completion scripts is purely local.

use anyhow::Error;
use clap::ArgMatches;

use crate::dispatch::gen_autocomplete;
use crate::common::app_context::AppContext;

/// Dispatch `manta gen-autocomplete`.
pub async fn handle_gen_autocomplete(
  cli_gen_autocomplete: &ArgMatches,
  _ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let cli = crate::build::build_cli();
  gen_autocomplete::exec(cli, cli_gen_autocomplete)
}
