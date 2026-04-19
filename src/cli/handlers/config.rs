use crate::cli::commands;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta config` subcommands (show, set, unset,
/// gen-autocomplete).
pub async fn handle_config(
  cli_config: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
    let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
    commands::config_show::exec(ctx.infra.backend, &token, ctx.cli.settings)
      .await?
  } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
    if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
      let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
      commands::config_set_hsm::exec(
        cli_config_set_hsm,
        ctx.infra.backend,
        &token,
      )
      .await?
    }
    if let Some(cli_config_set_parent_hsm) =
      cli_config_set.subcommand_matches("parent-hsm")
    {
      let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
      commands::config_set_parent_hsm::exec(
        cli_config_set_parent_hsm,
        ctx.infra.backend,
        &token,
      )
      .await?;
    }
    if let Some(cli_config_set_site) = cli_config_set.subcommand_matches("site")
    {
      commands::config_set_site::exec(cli_config_set_site)?;
    }
    if let Some(cli_config_set_log) = cli_config_set.subcommand_matches("log") {
      commands::config_set_log::exec(cli_config_set_log)?;
    }
  } else if let Some(cli_config_unset) = cli_config.subcommand_matches("unset")
  {
    if let Some(_cli_config_unset_hsm) =
      cli_config_unset.subcommand_matches("hsm")
    {
      commands::config_unset_hsm::exec()?;
    }
    if let Some(_cli_config_unset_parent_hsm) =
      cli_config_unset.subcommand_matches("parent-hsm")
    {
      let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
      commands::config_unset_parent_hsm::exec(ctx.infra.backend, &token)
        .await?;
    }
    if let Some(_cli_config_unset_auth) =
      cli_config_unset.subcommand_matches("auth")
    {
      commands::config_unset_auth::exec()?;
    }
  } else if let Some(cli_config_generate_autocomplete) =
    cli_config.subcommand_matches("gen-autocomplete")
  {
    // Rebuild CLI tree only when actually needed for
    // shell completion generation
    let cli = crate::cli::build::build_cli();
    commands::config_gen_autocomplete::exec(
      cli,
      cli_config_generate_autocomplete,
    )?;
  } else {
    bail!("Unknown 'config' subcommand");
  }
  Ok(())
}
