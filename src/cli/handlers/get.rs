use crate::cli::commands::{
  get_boot_parameters, get_cluster, get_configuration, get_group,
  get_hardware_cluster, get_hardware_node, get_images, get_kernel_parameters,
  get_nodes, get_session, get_template,
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
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  match cli_get.subcommand() {
    Some(("groups", m)) => get_group::exec(ctx, &token, m).await?,
    Some(("hardware", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        get_hardware_cluster::exec(ctx, &token, m).await?
      }
      Some(("node", m)) => get_hardware_node::exec(ctx, &token, m).await?,
      Some((other, _)) => bail!("Unknown 'get hardware' subcommand: {other}"),
      None => bail!("No 'get hardware' subcommand provided"),
    },
    Some(("configurations", m)) => {
      get_configuration::exec(ctx, &token, m).await?
    }
    Some(("sessions", m)) => get_session::exec(ctx, &token, m).await?,
    Some(("templates", m)) => get_template::exec(ctx, &token, m).await?,
    Some(("cluster", m)) => get_cluster::exec(ctx, &token, m).await?,
    Some(("nodes", m)) => get_nodes::exec(ctx, &token, m).await?,
    Some(("images", m)) => get_images::exec(ctx, &token, m).await?,
    Some(("boot-parameters", m)) => {
      get_boot_parameters::exec(ctx, &token, m).await?
    }
    Some(("kernel-parameters", m)) => {
      get_kernel_parameters::exec(ctx, &token, m).await?
    }
    Some(("redfish-endpoints", m)) => {
      crate::cli::commands::get_redfish_endpoints::exec(ctx, &token, m).await?
    }
    Some((other, _)) => bail!("Unknown 'get' subcommand: {other}"),
    None => bail!("No 'get' subcommand provided"),
  }
  Ok(())
}
