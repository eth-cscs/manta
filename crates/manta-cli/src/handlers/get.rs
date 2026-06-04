//! Routes `manta get *` subcommands to their exec functions.

use crate::commands::get::{
  boot_parameters as get_boot_parameters, configuration as get_configuration,
  group as get_group, group_hardware as get_group_hardware,
  group_nodes as get_group_nodes, hardware_nodes as get_hardware_nodes,
  images as get_images, kernel_parameters as get_kernel_parameters,
  nodes as get_nodes, session as get_session, template as get_template,
};
use crate::common::authentication::get_api_token;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

/// Dispatch `manta get` subcommands (groups, session,
/// configuration, template, images, cluster, hardware, nodes,
/// boot-parameters, kernel-parameters, redfish-endpoint).
pub async fn handle_get(
  cli_get: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_get.subcommand() {
    Some(("groups", m)) => get_group::exec(ctx, &token, m).await?,
    Some(("group-nodes", m)) => get_group_nodes::exec(ctx, &token, m).await?,
    Some(("group-hardware", m)) => {
      get_group_hardware::exec(ctx, &token, m).await?
    }
    Some(("hardware", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        eprintln!(
          "warning: 'manta get hardware cluster' is deprecated; \
           use 'manta get group-hardware' instead.",
        );
        get_group_hardware::exec(ctx, &token, m).await?
      }
      Some(("nodes", m)) => get_hardware_nodes::exec(ctx, &token, m).await?,
      Some((other, _)) => bail!("Unknown 'get hardware' subcommand: {other}"),
      None => bail!("No 'get hardware' subcommand provided"),
    },
    Some(("configurations", m)) => {
      get_configuration::exec(ctx, &token, m).await?
    }
    Some(("sessions", m)) => get_session::exec(ctx, &token, m).await?,
    Some(("templates", m)) => get_template::exec(ctx, &token, m).await?,
    Some(("cluster", m)) => {
      eprintln!(
        "warning: 'manta get cluster' is deprecated; \
         use 'manta get group-nodes' instead.",
      );
      get_group_nodes::exec(ctx, &token, m).await?
    }
    Some(("nodes", m)) => get_nodes::exec(ctx, &token, m).await?,
    Some(("images", m)) => get_images::exec(ctx, &token, m).await?,
    Some(("boot-parameters", m)) => {
      get_boot_parameters::exec(ctx, &token, m).await?
    }
    Some(("kernel-parameters", m)) => {
      get_kernel_parameters::exec(ctx, &token, m).await?
    }
    Some(("redfish-endpoints", m)) => {
      crate::commands::get::redfish_endpoints::exec(ctx, &token, m).await?
    }
    Some((other, _)) => bail!("Unknown 'get' subcommand: {other}"),
    None => bail!("No 'get' subcommand provided"),
  }
  Ok(())
}
