//! Routes `manta config *` subcommands to their exec functions.

use crate::commands;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

/// Dispatch `manta config` subcommands (show, set, unset,
/// gen-autocomplete).
pub async fn handle_config(
  cli_config: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  match cli_config.subcommand() {
    Some(("show", m)) => {
      let token = get_api_token(ctx).await?;
      let client = MantaClient::new(ctx.manta_server_url, ctx.site_name)?;
      let output_opt = m.opt_str("output");
      commands::config::show::exec(&client, &token, ctx.settings, output_opt)
        .await?;
    }
    Some(("set", m)) => match m.subcommand() {
      Some(("hsm", m)) => {
        let token = get_api_token(ctx).await?;
        let client = MantaClient::new(ctx.manta_server_url, ctx.site_name)?;
        commands::config::set_hsm::exec(m, &client, &token).await?;
      }
      Some(("parent-hsm", m)) => {
        let token = get_api_token(ctx).await?;
        let client = MantaClient::new(ctx.manta_server_url, ctx.site_name)?;
        commands::config::set_parent_hsm::exec(m, &client, &token).await?;
      }
      Some(("site", m)) => commands::config::set_site::exec(m)?,
      Some(("log", m)) => commands::config::set_log::exec(m)?,
      Some((other, _)) => bail!("Unknown 'config set' subcommand: {other}"),
      None => bail!("No 'config set' subcommand provided"),
    },
    Some(("unset", m)) => match m.subcommand() {
      Some(("hsm", _)) => commands::config::unset_hsm::exec()?,
      Some(("parent-hsm", _)) => commands::config::unset_parent_hsm::exec()?,
      Some(("auth", _)) => commands::config::unset_auth::exec()?,
      Some((other, _)) => bail!("Unknown 'config unset' subcommand: {other}"),
      None => bail!("No 'config unset' subcommand provided"),
    },
    Some(("gen-autocomplete", m)) => {
      let cli = crate::build::build_cli();
      commands::config::gen_autocomplete::exec(cli, m)?;
    }
    Some((other, _)) => bail!("Unknown 'config' subcommand: {other}"),
    None => bail!("No 'config' subcommand provided"),
  }
  Ok(())
}
