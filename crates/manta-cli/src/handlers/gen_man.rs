//! Routes the `manta gen-man` command to its exec function.
//!
//! Like `gen-autocomplete`, this handler does NOT call
//! `get_api_token` — installing man pages is purely local.

use std::path::PathBuf;

use anyhow::Error;
use clap::ArgMatches;

use crate::dispatch::gen_man;
use crate::common::app_context::AppContext;

/// Dispatch `manta gen-man`.
pub async fn handle_gen_man(
  cli_gen_man: &ArgMatches,
  _ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let path = cli_gen_man.get_one::<PathBuf>("path").cloned();
  let output_opt = cli_gen_man.get_one::<String>("output").map(String::as_str);

  let cli = crate::build::build_cli();
  gen_man::exec(cli, path, output_opt)
}
