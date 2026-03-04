use crate::cli::commands::{
  get_boot_parameters, get_cluster, get_configuration, get_group,
  get_hardware_cluster, get_hardware_node, get_images, get_kernel_parameters,
  get_nodes, get_session, get_template,
};
use crate::common::app_context::AppContext;
use anyhow::{Context, Error};
use clap::ArgMatches;
use manta_backend_dispatcher::types::bss::BootParameters;

pub async fn handle_get(
  cli_get: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(cli_get_groups) = cli_get.subcommand_matches("groups") {
    let group_name_arg_opt: Option<&String> = cli_get_groups.get_one("VALUE");
    let output: String = cli_get_groups
      .get_one("output")
      .cloned()
      .context("The 'output' argument is mandatory")?;
    get_group::exec(
      ctx.backend,
      ctx.site_name,
      group_name_arg_opt,
      ctx.settings_hsm_group_name_opt,
      &output,
    )
    .await?;
  } else if let Some(cli_get_hardware) = cli_get.subcommand_matches("hardware")
  {
    if let Some(cli_get_hardware_cluster) =
      cli_get_hardware.subcommand_matches("cluster")
    {
      let hsm_group_name_arg_opt: Option<&String> =
        cli_get_hardware_cluster.get_one("CLUSTER_NAME");
      get_hardware_cluster::exec(
        ctx.backend.clone(),
        ctx.site_name,
        hsm_group_name_arg_opt,
        ctx.settings_hsm_group_name_opt,
        cli_get_hardware_cluster.get_one::<String>("output"),
      )
      .await?;
    } else if let Some(cli_get_hardware_node) =
      cli_get_hardware.subcommand_matches("node")
    {
      let xnames = cli_get_hardware_node
        .get_one::<String>("XNAMES")
        .context("The 'XNAMES' argument must have a value")?;
      get_hardware_node::exec(
        ctx.backend,
        ctx.site_name,
        xnames,
        cli_get_hardware_node.get_one::<String>("type"),
        cli_get_hardware_node.get_one::<String>("output"),
      )
      .await?;
    }
  } else if let Some(cli_get_configuration) =
    cli_get.subcommand_matches("configurations")
  {
    let name: Option<&String> = cli_get_configuration.get_one::<String>("name");
    let pattern: Option<&String> =
      cli_get_configuration.get_one::<String>("pattern");
    let hsm_group_name_arg_rslt =
      cli_get_configuration.try_get_one("hsm-group");
    let limit: Option<&u8> =
      if let Some(true) = cli_get_configuration.get_one("most-recent") {
        Some(&1)
      } else {
        cli_get_configuration.get_one::<u8>("limit")
      };
    let output: Option<&String> = cli_get_configuration.get_one("output");
    get_configuration::exec(
      ctx,
      name.map(String::as_str),
      pattern.map(String::as_str),
      hsm_group_name_arg_rslt.unwrap_or(None),
      None,
      None,
      limit,
      output.map(String::as_str),
    )
    .await?;
  } else if let Some(cli_get_session) = cli_get.subcommand_matches("sessions") {
    get_session::exec(
      ctx.backend,
      ctx.site_name,
      ctx.shasta_base_url,
      ctx.shasta_root_cert,
      cli_get_session,
    )
    .await?;
  } else if let Some(cli_get_template) = cli_get.subcommand_matches("templates")
  {
    get_template::exec(
      ctx.backend,
      ctx.site_name,
      ctx.shasta_base_url,
      ctx.shasta_root_cert,
      cli_get_template,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;
  } else if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
    get_cluster::exec(
      ctx.backend,
      ctx.site_name,
      ctx.shasta_base_url,
      ctx.shasta_root_cert,
      cli_get_cluster,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;
  } else if let Some(cli_get_nodes) = cli_get.subcommand_matches("nodes") {
    get_nodes::exec(
      ctx.backend,
      ctx.site_name,
      ctx.shasta_base_url,
      ctx.shasta_root_cert,
      cli_get_nodes,
    )
    .await?;
  } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
    get_images::exec(
      ctx.backend,
      ctx.site_name,
      ctx.shasta_base_url,
      ctx.shasta_root_cert,
      cli_get_images,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;
  } else if let Some(cli_get_boot_parameters) =
    cli_get.subcommand_matches("boot-parameters")
  {
    let boot_parameters_vec: Vec<BootParameters> = get_boot_parameters::exec(
      ctx.backend,
      ctx.site_name,
      cli_get_boot_parameters,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&boot_parameters_vec)?);
  } else if let Some(cli_get_kernel_parameters) =
    cli_get.subcommand_matches("kernel-parameters")
  {
    let _ = get_kernel_parameters::exec(
      ctx.backend,
      ctx.site_name,
      cli_get_kernel_parameters,
      ctx.settings_hsm_group_name_opt,
    )
    .await;
  } else if let Some(cli_get_redfish_endopints) =
    cli_get.subcommand_matches("redfish-endpoints")
  {
    crate::cli::commands::get_redfish_endpoints::exec(
      ctx.backend,
      ctx.site_name,
      cli_get_redfish_endopints,
    )
    .await?;
  }
  Ok(())
}
