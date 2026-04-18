use crate::common::app_context::AppContext;
use anyhow::Error;
use clap::ArgMatches;

use crate::cli::handlers::{
  add, apply, config, console, delete, get, log, migrate, misc, power, update,
};

/// Parse CLI arguments and dispatch to the appropriate
/// subcommand handler.
pub async fn process_cli(
  cli_root: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(cli_config) = cli_root.subcommand_matches("config") {
    config::handle_config(cli_config, ctx).await?;
  } else if let Some(cli_power) = cli_root.subcommand_matches("power") {
    power::handle_power(cli_power, ctx).await?;
  } else if let Some(cli_add) = cli_root.subcommand_matches("add") {
    add::handle_add(cli_add, ctx).await?;
  } else if let Some(cli_update) = cli_root.subcommand_matches("update") {
    update::handle_update(cli_update, ctx).await?;
  } else if let Some(cli_get) = cli_root.subcommand_matches("get") {
    get::handle_get(cli_get, ctx).await?;
  } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
    apply::handle_apply(cli_apply, ctx).await?;
  } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
    log::handle_log(cli_log, ctx).await?;
  } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
    console::handle_console(cli_console, ctx).await?;
  } else if let Some(cli_migrate) = cli_root.subcommand_matches("migrate") {
    migrate::handle_migrate(cli_migrate, ctx).await?;
  } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
    delete::handle_delete(cli_delete, ctx).await?;
  } else {
    misc::handle_misc(&cli_root, ctx).await?;
  }

  Ok(())
}
