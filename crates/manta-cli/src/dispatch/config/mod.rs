//! `manta config` subcommands.
//!
//! Each submodule reads or mutates the local `cli.toml` via
//! [`manta_shared::common::config::read_config_toml`] /
//! [`manta_shared::common::config::write_config_toml`] — almost no
//! server traffic. The exceptions are `show` (optionally fetches the
//! per-site available-groups list) and `set hsm` (validates the target
//! group against `GET /groups`). Token acquisition only happens on the
//! paths that need it.

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

/// Dispatch `manta config` subcommands (`show`, `set`, `unset`).
///
/// Routes the parsed clap matches to one of the per-subcommand `exec`
/// handlers in this module. Only `show` (when a site is selected) and
/// `set hsm` resolve a token and build a [`MantaClient`]; the other
/// handlers operate purely on the local config file.
///
/// # Errors
///
/// Returns an error if token acquisition fails on a path that needs
/// one, the selected handler fails, or no recognised subcommand was
/// provided.
pub async fn handle_config(
  cli_config: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  match cli_config.subcommand() {
    Some(("show", m)) => {
      let output_opt = m.opt_str("output");
      // `config show` is mostly local config, so it works without a
      // site. Only when one is selected do we authenticate and build a
      // client to fetch the per-site list of available groups.
      let client = if ctx.site_name.is_some() {
        let token = get_api_token(ctx).await?;
        Some(MantaClient::from_app_ctx(ctx, Some(&token))?)
      } else {
        None
      };
      show::exec(client.as_ref(), ctx.settings, output_opt).await?;
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
