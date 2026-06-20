//! `manta config` subcommands.

pub mod set_hsm;
pub mod set_log;
pub mod set_read_only;
pub mod set_site;
pub mod show;
pub mod unset_auth;
pub mod unset_hsm;
pub mod unset_read_only;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta config` subcommands (show, set, unset).
pub async fn handle_config(
  cli_config: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  match cli_config.subcommand() {
    Some(("show", m)) => {
      let token = get_api_token(ctx).await?;
      let client = MantaClient::from_app_ctx(ctx, Some(&token))?;
      let output_opt = m.opt_str("output");
      show::exec(&client, &token, ctx.settings, output_opt).await?;
    }
    Some(("set", m)) => match m.subcommand() {
      Some(("hsm", m)) => {
        let token = get_api_token(ctx).await?;
        let client = MantaClient::from_app_ctx(ctx, Some(&token))?;
        set_hsm::exec(m, &client, &token).await?;
      }
      Some(("site", m)) => set_site::exec(m)?,
      Some(("log", m)) => set_log::exec(m)?,
      Some(("read-only", _)) => set_read_only::exec().await?,
      Some((other, _)) => bail!("Unknown 'config set' subcommand: {other}"),
      None => bail!("No 'config set' subcommand provided"),
    },
    Some(("unset", m)) => match m.subcommand() {
      Some(("hsm", _)) => unset_hsm::exec()?,
      Some(("auth", _)) => unset_auth::exec()?,
      Some(("read-only", _)) => unset_read_only::exec().await?,
      Some((other, _)) => bail!("Unknown 'config unset' subcommand: {other}"),
      None => bail!("No 'config unset' subcommand provided"),
    },
    Some((other, _)) => bail!("Unknown 'config' subcommand: {other}"),
    None => bail!("No 'config' subcommand provided"),
  }
  Ok(())
}
