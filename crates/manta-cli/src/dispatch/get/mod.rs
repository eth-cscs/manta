//! `manta get` subcommands.
//!
//! Each submodule implements one read-only subcommand by parsing clap
//! matches into a typed `GetXxxParams`, calling the corresponding
//! `MantaClient::openapi.get_*` method, and handing the response to a
//! renderer in [`crate::output`]. Every handler authenticates once via
//! [`get_api_token`] before dispatching.

pub mod boot_parameters;
pub mod configurations;
pub mod group_nodes;
pub mod groups;
pub mod hardware_group;
pub mod hardware_nodes;
pub mod images;
pub mod kernel_parameters;
pub mod nodes;
pub mod redfish_endpoints;
pub mod sessions;
pub mod templates;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta get` subcommands.
///
/// Routes the parsed clap matches to one of the per-subcommand `exec`
/// handlers in this module: `groups`, `group-nodes`, `hardware nodes`,
/// `hardware group`, `configurations`, `sessions`, `templates`,
/// `nodes`, `images`, `boot-parameters`, `kernel-parameters`, and
/// `redfish-endpoints`. The API token is resolved once here and passed
/// down so each handler can issue HTTPS requests against `manta-server`.
///
/// # Errors
///
/// Returns an error if token acquisition fails, the selected
/// subcommand handler fails (network, server error, output formatting),
/// or no recognised subcommand was provided.
pub async fn handle_get(
  cli_get: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_get.subcommand() {
    Some(("groups", m)) => groups::exec(ctx, &token, m).await?,
    Some(("group-nodes", m)) => group_nodes::exec(ctx, &token, m).await?,
    Some(("hardware", m)) => match m.subcommand() {
      Some(("nodes", m)) => hardware_nodes::exec(ctx, &token, m).await?,
      Some(("group", m)) => hardware_group::exec(ctx, &token, m).await?,
      Some((other, _)) => bail!("Unknown 'get hardware' subcommand: {other}"),
      None => bail!("No 'get hardware' subcommand provided"),
    },
    Some(("configurations", m)) => {
      configurations::exec(ctx, &token, m).await?;
    }
    Some(("sessions", m)) => sessions::exec(ctx, &token, m).await?,
    Some(("templates", m)) => templates::exec(ctx, &token, m).await?,
    Some(("nodes", m)) => nodes::exec(ctx, &token, m).await?,
    Some(("images", m)) => images::exec(ctx, &token, m).await?,
    Some(("boot-parameters", m)) => {
      boot_parameters::exec(ctx, &token, m).await?;
    }
    Some(("kernel-parameters", m)) => {
      kernel_parameters::exec(ctx, &token, m).await?;
    }
    Some(("redfish-endpoints", m)) => {
      redfish_endpoints::exec(ctx, &token, m).await?;
    }
    Some((other, _)) => bail!("Unknown 'get' subcommand: {other}"),
    None => bail!("No 'get' subcommand provided"),
  }
  Ok(())
}
