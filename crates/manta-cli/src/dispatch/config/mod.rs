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
use crate::common::clap_ext::ArgMatchesExt;
use crate::common::session::fetch_accessible_groups;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta config` subcommands (`show`, `set`, `unset`).
///
/// Routes the parsed clap matches to one of the per-subcommand `exec`
/// handlers in this module. Only `show` (when a site is selected) and
/// `set hsm` use the server, through the session `process_cli` built;
/// the other handlers operate purely on the local config file.
///
/// # Errors
///
/// Returns an error if `set hsm` runs without a site, the selected
/// handler fails, or no recognised subcommand was provided.
pub async fn handle_config(
  cli_config: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  match cli_config.subcommand() {
    Some(("show", m)) => {
      let output_opt = m.opt_str("output");
      show::exec(ctx, output_opt).await?;
    }
    Some(("set", m)) => match m.subcommand() {
      Some(("hsm", m)) => {
        // The accessible groups come from the site.
        ctx.require_site()?;
        let client = MantaClient::from_app_ctx(ctx)?;
        let accessible_groups = fetch_accessible_groups(&client).await?;
        set_hsm::exec(m, &accessible_groups).await?;
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
