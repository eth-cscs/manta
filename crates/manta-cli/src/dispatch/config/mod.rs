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
pub mod set_site;
pub mod show;
pub mod unset_auth;
pub mod unset_hsm;

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
      // SessionContext (built at the top of process_cli) carries
      // everything the renderer needs — no per-call auth or client
      // construction here.
      show::exec(ctx, output_opt).await?;
    }
    Some(("set", m)) => match m.subcommand() {
      Some(("hsm", m)) => {
        let token = get_api_token(ctx).await?;
        let client = MantaClient::from_app_ctx(ctx, Some(&token))?;
        // SessionContext is guaranteed Some here: this arm only runs
        // for verbs that don't skip the session build in process_cli
        // (set_hsm requires a site, so a session was built).
        let accessible_groups = ctx
          .session
          .as_ref()
          .map(|s| s.accessible_groups.as_slice())
          .unwrap_or_default();
        set_hsm::exec(m, &client, &token, accessible_groups).await?;
      }
      Some(("site", m)) => set_site::exec(m)?,
      Some(("log", m)) => set_log::exec(m)?,
      Some((other, _)) => bail!("Unknown 'config set' subcommand: {other}"),
      None => bail!("No 'config set' subcommand provided"),
    },
    Some(("unset", m)) => match m.subcommand() {
      Some(("hsm", _)) => unset_hsm::exec()?,
      Some(("auth", _)) => unset_auth::exec()?,
      Some((other, _)) => bail!("Unknown 'config unset' subcommand: {other}"),
      None => bail!("No 'config unset' subcommand provided"),
    },
    Some((other, _)) => bail!("Unknown 'config' subcommand: {other}"),
    None => bail!("No 'config' subcommand provided"),
  }
  Ok(())
}
