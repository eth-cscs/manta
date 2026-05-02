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
  match cli_config.subcommand() {
    Some(("show", _)) => {
      let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
      commands::config_show::exec(ctx.infra.backend, &token, ctx.cli.settings)
        .await?;
    }
    Some(("set", m)) => match m.subcommand() {
      Some(("hsm", m)) => {
        let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
        commands::config_set_hsm::exec(m, ctx.infra.backend, &token).await?;
      }
      Some(("parent-hsm", m)) => {
        let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
        commands::config_set_parent_hsm::exec(m, ctx.infra.backend, &token)
          .await?;
      }
      Some(("site", m)) => commands::config_set_site::exec(m)?,
      Some(("log", m)) => commands::config_set_log::exec(m)?,
      Some((other, _)) => bail!("Unknown 'config set' subcommand: {other}"),
      None => bail!("No 'config set' subcommand provided"),
    },
    Some(("unset", m)) => match m.subcommand() {
      Some(("hsm", _)) => commands::config_unset_hsm::exec()?,
      Some(("parent-hsm", _)) => {
        let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
        commands::config_unset_parent_hsm::exec(ctx.infra.backend, &token)
          .await?;
      }
      Some(("auth", _)) => commands::config_unset_auth::exec()?,
      Some((other, _)) => bail!("Unknown 'config unset' subcommand: {other}"),
      None => bail!("No 'config unset' subcommand provided"),
    },
    Some(("gen-autocomplete", m)) => {
      let cli = crate::cli::build::build_cli();
      commands::config_gen_autocomplete::exec(cli, m)?;
    }
    Some((other, _)) => bail!("Unknown 'config' subcommand: {other}"),
    None => bail!("No 'config' subcommand provided"),
  }
  Ok(())
}
