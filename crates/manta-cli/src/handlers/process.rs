//! Root CLI dispatcher: matches the parsed top-level verb and calls
//! the per-verb handler in this module's siblings.

use crate::common::app_context::AppContext;
use anyhow::{Error, bail};
use clap::ArgMatches;

use crate::handlers::{
  add, apply, backup, config, console, delete, gen_autocomplete, gen_man, get,
  log, migrate, power, restore, run, upgrade,
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
