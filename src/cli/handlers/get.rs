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
  // Resolve auth token once for all subcommands.
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  if let Some(cli_get_groups) = cli_get.subcommand_matches("groups") {
    get_group::exec(ctx, &token, cli_get_groups).await?;
  } else if let Some(cli_get_hardware) = cli_get.subcommand_matches("hardware")
  {
    if let Some(cli_get_hardware_cluster) =
      cli_get_hardware.subcommand_matches("cluster")
    {
      get_hardware_cluster::exec(ctx, &token, cli_get_hardware_cluster)
        .await?;
    } else if let Some(cli_get_hardware_node) =
      cli_get_hardware.subcommand_matches("node")
    {
      get_hardware_node::exec(ctx, &token, cli_get_hardware_node).await?;
    } else {
      bail!("Unknown 'get hardware' subcommand");
    }
  } else if let Some(cli_get_configuration) =
    cli_get.subcommand_matches("configurations")
  {
    get_configuration::exec(ctx, &token, cli_get_configuration).await?;
  } else if let Some(cli_get_session) = cli_get.subcommand_matches("sessions") {
    get_session::exec(ctx, &token, cli_get_session).await?;
  } else if let Some(cli_get_template) = cli_get.subcommand_matches("templates")
  {
    get_template::exec(ctx, &token, cli_get_template).await?;
  } else if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
    get_cluster::exec(ctx, &token, cli_get_cluster).await?;
  } else if let Some(cli_get_nodes) = cli_get.subcommand_matches("nodes") {
    get_nodes::exec(ctx, &token, cli_get_nodes).await?;
  } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
    get_images::exec(ctx, &token, cli_get_images).await?;
  } else if let Some(cli_get_boot_parameters) =
    cli_get.subcommand_matches("boot-parameters")
  {
    get_boot_parameters::exec(ctx, &token, cli_get_boot_parameters).await?;
  } else if let Some(cli_get_kernel_parameters) =
    cli_get.subcommand_matches("kernel-parameters")
  {
    get_kernel_parameters::exec(ctx, &token, cli_get_kernel_parameters)
      .await?;
  } else if let Some(cli_get_redfish_endpoints) =
    cli_get.subcommand_matches("redfish-endpoints")
  {
    crate::cli::commands::get_redfish_endpoints::exec(
      ctx,
      &token,
      cli_get_redfish_endpoints,
    )
    .await?;
  } else {
    bail!("Unknown 'get' subcommand");
  }
  Ok(())
}
