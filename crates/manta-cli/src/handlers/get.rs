//! Routes `manta get *` subcommands to their exec functions.

use crate::dispatch::get::{
  boot_parameters as get_boot_parameters, configurations as get_configurations,
  group_hardware as get_group_hardware, group_nodes as get_group_nodes,
  groups as get_groups, hardware_nodes as get_hardware_nodes,
  images as get_images, kernel_parameters as get_kernel_parameters,
  nodes as get_nodes, sessions as get_sessions, templates as get_templates,
};
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta get` subcommands (groups, session,
/// configuration, template, images, cluster, hardware, nodes,
/// boot-parameters, kernel-parameters, redfish-endpoint).
pub async fn handle_get(
  cli_get: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_get.subcommand() {
    Some(("groups", m)) => get_groups::exec(ctx, &token, m).await?,
    Some(("group-nodes", m)) => get_group_nodes::exec(ctx, &token, m).await?,
    Some(("group-hardware", m)) => {
      get_group_hardware::exec(ctx, &token, m).await?
    }
    Some(("hardware", m)) => match m.subcommand() {
      Some(("nodes", m)) => get_hardware_nodes::exec(ctx, &token, m).await?,
      Some((other, _)) => bail!("Unknown 'get hardware' subcommand: {other}"),
      None => bail!("No 'get hardware' subcommand provided"),
    },
    Some(("configurations", m)) => {
      get_configurations::exec(ctx, &token, m).await?
    }
    Some(("sessions", m)) => get_sessions::exec(ctx, &token, m).await?,
    Some(("templates", m)) => get_templates::exec(ctx, &token, m).await?,
    Some(("nodes", m)) => get_nodes::exec(ctx, &token, m).await?,
    Some(("images", m)) => get_images::exec(ctx, &token, m).await?,
    Some(("boot-parameters", m)) => {
      get_boot_parameters::exec(ctx, &token, m).await?
    }
    Some(("kernel-parameters", m)) => {
      get_kernel_parameters::exec(ctx, &token, m).await?
    }
    Some(("redfish-endpoints", m)) => {
      crate::dispatch::get::redfish_endpoints::exec(ctx, &token, m).await?
    }
    Some((other, _)) => bail!("Unknown 'get' subcommand: {other}"),
    None => bail!("No 'get' subcommand provided"),
  }
  Ok(())
}
