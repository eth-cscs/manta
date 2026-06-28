//! Root CLI dispatcher: matches the parsed top-level verb and calls
//! the per-verb handler in this module's siblings.
//!
//! All verbs except the local-only `gen-man` / `gen-autocomplete` and
//! the read-only `get` family go through the read-only gate first; the
//! gate refuses backend-mutating verbs when `read_only = true` is set
//! in `cli.toml`, before any HTTP request leaves the process.

use crate::common::app_context::AppContext;
use anyhow::{Error, bail};
use clap::ArgMatches;

use crate::dispatch::{
  add, apply, backup, config, console, delete, gen_autocomplete, gen_man, get,
  log, migrate, power, restore, run, upgrade,
};

/// Parse CLI arguments and dispatch to the appropriate
/// subcommand handler.
///
/// # Errors
///
/// Returns an error when the read-only gate refuses a mutating verb,
/// when no subcommand is provided, when the subcommand name is unknown,
/// or when the chosen verb's handler returns an error (authentication,
/// HTTP, validation, or backend failures).
pub async fn process_cli(
  cli_root: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  // Read-only gate: when `read_only = true` is set in cli.toml, refuse
  // backend-mutating verbs before any HTTP request leaves the process.
  // `--dry-run` invocations and read verbs bypass the gate; see the
  // policy module for the full classification.
  crate::common::read_only::read_only_gate(cli_root, ctx.read_only)?;

  match cli_root.subcommand() {
    Some(("config", m)) => config::handle_config(m, ctx).await?,
    Some(("power", m)) => power::handle_power(m, ctx).await?,
    Some(("add", m)) => add::handle_add(m, ctx).await?,
    Some(("get", m)) => get::handle_get(m, ctx).await?,
    Some(("apply", m)) => apply::handle_apply(m, ctx).await?,
    Some(("log", m)) => log::handle_log(m, ctx).await?,
    Some(("console", m)) => console::handle_console(m, ctx).await?,
    Some(("migrate", m)) => migrate::handle_migrate(m, ctx).await?,
    Some(("backup", m)) => backup::handle_backup(m, ctx).await?,
    Some(("restore", m)) => restore::handle_restore(m, ctx).await?,
    Some(("run", m)) => run::handle_run(m, ctx).await?,
    Some(("delete", m)) => delete::handle_delete(m, ctx).await?,
    Some(("gen-autocomplete", m)) => {
      gen_autocomplete::handle_gen_autocomplete(m, ctx).await?;
    }
    Some(("gen-man", m)) => gen_man::handle_gen_man(m, ctx).await?,
    Some(("upgrade", m)) => upgrade::handle_upgrade(m, ctx).await?,
    Some((other, _)) => bail!("Unknown command: {other}"),
    None => bail!("No command provided"),
  }
  Ok(())
}
