use crate::cli::commands::{
  get_boot_parameters, get_cluster, get_configuration, get_group,
  get_hardware_cluster, get_hardware_node, get_images, get_kernel_parameters,
  get_nodes, get_session, get_template,
};
use crate::common::authentication::get_api_token;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;
use manta_backend_dispatcher::types::bss::BootParameters;

pub async fn handle_get(
  cli_get: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  vault_base_url: Option<&String>,
  gitea_base_url: &str,
  settings_hsm_group_name_opt: Option<&String>,
) -> Result<(), Error> {
  if let Some(cli_get_groups) = cli_get.subcommand_matches("groups") {
    let group_name_arg_opt: Option<&String> = cli_get_groups.get_one("VALUE");
    let output: String = cli_get_groups
      .get_one("output")
      .cloned()
      .expect("ERROR - 'output' argument is mandatory");
    get_group::exec(
      backend,
      site_name,
      group_name_arg_opt,
      settings_hsm_group_name_opt,
      &output,
    )
    .await?;
  } else if let Some(cli_get_hardware) = cli_get.subcommand_matches("hardware") {
    if let Some(cli_get_hardware_cluster) =
      cli_get_hardware.subcommand_matches("cluster")
    {
      let hsm_group_name_arg_opt: Option<&String> =
        cli_get_hardware_cluster.get_one("CLUSTER_NAME");
      get_hardware_cluster::exec(
        backend.clone(),
        site_name,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
        cli_get_hardware_cluster.get_one::<String>("output"),
      )
      .await;
    } else if let Some(cli_get_hardware_node) =
      cli_get_hardware.subcommand_matches("node")
    {
      let xnames = cli_get_hardware_node
        .get_one::<String>("XNAMES")
        .expect("HSM group name is needed at this point");
      get_hardware_node::exec(
        backend,
        site_name,
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
      backend,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      gitea_base_url,
      name.map(String::as_str),
      pattern.map(String::as_str),
      hsm_group_name_arg_rslt.unwrap_or(None),
      settings_hsm_group_name_opt,
      None,
      None,
      limit,
      output.map(String::as_str),
      site_name,
    )
    .await?;
  } else if let Some(cli_get_session) = cli_get.subcommand_matches("sessions") {
    let shasta_token = get_api_token(backend, site_name).await?;
    get_session::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      cli_get_session,
    )
    .await?;
  } else if let Some(cli_get_template) = cli_get.subcommand_matches("templates")
  {
    get_template::exec(
      backend,
      site_name,
      shasta_base_url,
      shasta_root_cert,
      cli_get_template,
      settings_hsm_group_name_opt,
    )
    .await?;
  } else if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
    get_cluster::exec(
      backend,
      site_name,
      shasta_base_url,
      shasta_root_cert,
      cli_get_cluster,
      settings_hsm_group_name_opt,
    )
    .await?;
  } else if let Some(cli_get_nodes) = cli_get.subcommand_matches("nodes") {
    let shasta_token = get_api_token(backend, site_name).await?;
    get_nodes::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      cli_get_nodes,
    )
    .await?;
  } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
    let shasta_token = get_api_token(backend, site_name).await?;
    get_images::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      cli_get_images,
      settings_hsm_group_name_opt,
    )
    .await?;
  } else if let Some(cli_get_boot_parameters) =
    cli_get.subcommand_matches("boot-parameters")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let boot_parameters_vec: Vec<BootParameters> = get_boot_parameters::exec(
      backend,
      &shasta_token,
      cli_get_boot_parameters,
      settings_hsm_group_name_opt,
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&boot_parameters_vec)?);
  } else if let Some(cli_get_kernel_parameters) =
    cli_get.subcommand_matches("kernel-parameters")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let _ = get_kernel_parameters::exec(
      backend,
      &shasta_token,
      cli_get_kernel_parameters,
      settings_hsm_group_name_opt,
    )
    .await;
  } else if let Some(cli_get_redfish_endopints) =
    cli_get.subcommand_matches("redfish-endpoints")
  {
    crate::cli::commands::get_redfish_endpoints::exec(
      backend,
      site_name,
      cli_get_redfish_endopints,
    )
    .await?;
  }
  Ok(())
}
