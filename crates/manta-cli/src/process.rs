//! Top-level CLI dispatch: matches the parsed command tree and calls handlers.

use crate::common::app_context::AppContext;
use anyhow::Error;
use clap::ArgMatches;

use crate::handlers::{
  add, apply, backup, config, console, delete, deprecated_aliases, get, log,
  migrate, power, restore, run, update,
};

/// Parse CLI arguments and dispatch to the appropriate
/// subcommand handler.
pub async fn process_cli(
  cli_root: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  match cli_root.subcommand() {
    Some(("config", m)) => config::handle_config(m, ctx).await?,
    Some(("power", m)) => power::handle_power(m, ctx).await?,
    Some(("add", m)) => add::handle_add(m, ctx).await?,
    Some(("update", m)) => update::handle_update(m, ctx).await?,
    Some(("get", m)) => get::handle_get(m, ctx).await?,
    Some(("apply", m)) => apply::handle_apply(m, ctx).await?,
    Some(("log", m)) => log::handle_log(m, ctx).await?,
    Some(("console", m)) => console::handle_console(m, ctx).await?,
    Some(("migrate", m)) => migrate::handle_migrate(m, ctx).await?,
    Some(("backup", m)) => backup::handle_backup(m, ctx).await?,
    Some(("restore", m)) => restore::handle_restore(m, ctx).await?,
    Some(("run", m)) => run::handle_run(m, ctx).await?,
    Some(("delete", m)) => delete::handle_delete(m, ctx).await?,
    _ => deprecated_aliases::handle_deprecated_aliases(cli_root, ctx).await?,
  }
  Ok(())
}
