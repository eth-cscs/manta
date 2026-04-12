use crate::cli::commands::{
  add_group, add_hw_component_cluster, add_kernel_parameters, add_node,
};
use crate::common::app_context::AppContext;
use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
};
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::hsm::{
  redfish_endpoint::RedfishEndpointTrait,
};
use manta_backend_dispatcher::types::{
  bss::BootParameters,
  hsm::inventory::{RedfishEndpoint, RedfishEndpointArray},
};
use serde_json::Value;
use std::path::PathBuf;

/// Dispatch `manta add` subcommands (node, group,
/// kernel-parameters, hardware cluster).
pub async fn handle_add(
  cli_add: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(cli_add_node) = cli_add.subcommand_matches("node") {
    let shasta_token = get_api_token(ctx.backend, ctx.site_name).await?;
    let id = cli_add_node
      .get_one::<String>("id")
      .context("'id' argument is mandatory")?;
    let group = cli_add_node
      .get_one::<String>("group")
      .context("'group' argument is mandatory")?;
    let hardware_file_opt = cli_add_node.get_one::<PathBuf>("hardware");

    let arch_opt = cli_add_node.get_one::<String>("arch").cloned();
    let enabled = cli_add_node
      .get_one::<bool>("disabled")
      .cloned()
      .unwrap_or(true);

    add_node::exec(
      ctx,
      &shasta_token,
      id,
      group,
      enabled,
      arch_opt,
      hardware_file_opt,
    )
    .await?;
  } else if let Some(cli_add_group) = cli_add.subcommand_matches("group") {
    let shasta_token = get_api_token(ctx.backend, ctx.site_name).await?;
    let label = cli_add_group
      .get_one::<String>("label")
      .context("'label' argument is mandatory")?;
    let description = cli_add_group
      .get_one::<String>("description")
      .map(String::as_str);
    let node_expression =
      cli_add_group.get_one::<String>("nodes").map(String::as_str);
    add_group::exec(
      ctx,
      &shasta_token,
      label,
      description,
      node_expression,
      true,
      false,
    )
    .await?;
  } else if let Some(cli_add_hw_configuration) =
    cli_add.subcommand_matches("hardware")
  {
    let shasta_token = get_api_token(ctx.backend, ctx.site_name).await?;
    let target_hsm_group_name_arg_opt = cli_add_hw_configuration
      .get_one::<String>("target-cluster")
      .map(String::as_str);
    let target_hsm_group_vec = get_groups_names_available(
      ctx.backend,
      &shasta_token,
      target_hsm_group_name_arg_opt,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;
    let parent_hsm_group_name_arg_opt = cli_add_hw_configuration
      .get_one::<String>("parent-cluster")
      .map(String::as_str);
    let parent_hsm_group_vec = get_groups_names_available(
      ctx.backend,
      &shasta_token,
      parent_hsm_group_name_arg_opt,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;
    let dryrun = cli_add_hw_configuration.get_flag("dry-run");
    let create_hsm_group = *cli_add_hw_configuration
      .get_one::<bool>("create-hsm-group")
      .unwrap_or(&false);
    add_hw_component_cluster::exec(
      ctx.backend,
      &shasta_token,
      target_hsm_group_vec
        .first()
        .context("No target HSM groups available")?,
      parent_hsm_group_vec
        .first()
        .context("No parent HSM groups available")?,
      cli_add_hw_configuration
        .get_one::<String>("pattern")
        .context("'pattern' argument is mandatory")?,
      dryrun,
      create_hsm_group,
    )
    .await?;
  } else if let Some(cli_add_boot_parameters) =
    cli_add.subcommand_matches("boot-parameters")
  {
    let shasta_token = get_api_token(ctx.backend, ctx.site_name).await?;
    let hosts = cli_add_boot_parameters
      .get_one::<String>("hosts")
      .context("'hosts' argument is mandatory")?;
    let macs: Option<String> = cli_add_boot_parameters.get_one("macs").cloned();
    let nids: Option<String> = cli_add_boot_parameters.get_one("nids").cloned();
    let params = cli_add_boot_parameters
      .get_one::<String>("params")
      .context("'params' argument is mandatory")?
      .clone();
    let kernel = cli_add_boot_parameters
      .get_one::<String>("kernel")
      .context("'kernel' argument is mandatory")?
      .clone();
    let initrd = cli_add_boot_parameters
      .get_one::<String>("initrd")
      .context("'initrd' argument is mandatory")?
      .clone();
    let cloud_init = cli_add_boot_parameters
      .get_one::<Value>("cloud-init")
      .cloned();
    let host_vec = hosts
      .split(',')
      .map(|value| value.trim().to_string())
      .collect::<Vec<String>>();
    let mac_vec = macs.map(|x| {
      x.split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<String>>()
    });
    let nid_vec: Option<Vec<u32>> = nids
      .map(|x| {
        x.split(',')
          .map(|value| {
            value.trim().parse().with_context(|| {
              format!(
                "Could not parse NID value '{}' as a number",
                value.trim()
              )
            })
          })
          .collect::<Result<Vec<u32>, _>>()
      })
      .transpose()?;
    let boot_parameters = BootParameters {
      hosts: host_vec,
      macs: mac_vec,
      nids: nid_vec,
      params,
      kernel,
      initrd,
      cloud_init,
    };
    ctx
      .backend
      .add_bootparameters(&shasta_token, &boot_parameters)
      .await?;
    println!("Boot parameters created successfully");
  } else if let Some(cli_add_kernel_parameters) =
    cli_add.subcommand_matches("kernel-parameters")
  {
    let hsm_group_name_arg_opt = cli_add_kernel_parameters
      .get_one::<String>("hsm-group")
      .map(String::as_str);
    let nodes_opt: Option<&str> = if hsm_group_name_arg_opt.is_none() {
      cli_add_kernel_parameters
        .get_one::<String>("nodes")
        .map(String::as_str)
    } else {
      None
    };
    let kernel_parameters = cli_add_kernel_parameters
      .get_one::<String>("VALUE")
      .context("'VALUE' argument is mandatory")?;
    let overwrite: bool = cli_add_kernel_parameters.get_flag("overwrite");
    let assume_yes: bool = cli_add_kernel_parameters.get_flag("assume-yes");
    let do_not_reboot: bool =
      cli_add_kernel_parameters.get_flag("do-not-reboot");
    let dryrun = cli_add_kernel_parameters.get_flag("dry-run");
    add_kernel_parameters::exec(
      ctx,
      kernel_parameters,
      nodes_opt,
      hsm_group_name_arg_opt,
      overwrite,
      assume_yes,
      do_not_reboot,
      dryrun,
    )
    .await?;
  } else if let Some(cli_add_redfish_endpoint) =
    cli_add.subcommand_matches("redfish-endpoint")
  {
    let shasta_token = get_api_token(ctx.backend, ctx.site_name).await?;
    let id = cli_add_redfish_endpoint
      .get_one::<String>("id")
      .context("'id' argument is mandatory")?
      .to_string();
    let name: Option<String> =
      cli_add_redfish_endpoint.get_one("name").cloned();
    let hostname: Option<String> = cli_add_redfish_endpoint
      .get_one::<String>("hostname")
      .cloned();
    let domain: Option<String> = cli_add_redfish_endpoint
      .get_one::<String>("domain")
      .cloned();
    let fqdn: Option<String> =
      cli_add_redfish_endpoint.get_one::<String>("fqdn").cloned();
    let enabled: bool = cli_add_redfish_endpoint.get_flag("enabled");
    let user: Option<String> =
      cli_add_redfish_endpoint.get_one::<String>("user").cloned();
    let password: Option<String> = cli_add_redfish_endpoint
      .get_one::<String>("password")
      .cloned();
    let use_ssdp: bool = cli_add_redfish_endpoint.get_flag("use-ssdp");
    let mac_required: bool = cli_add_redfish_endpoint.get_flag("mac-required");
    let mac_addr: Option<String> = cli_add_redfish_endpoint
      .get_one::<String>("macaddr")
      .cloned();
    let ip_address: Option<String> = cli_add_redfish_endpoint
      .get_one::<String>("ipaddress")
      .cloned();
    let rediscover_on_update: bool =
      cli_add_redfish_endpoint.get_flag("rediscover-on-update");
    let template_id: Option<String> = cli_add_redfish_endpoint
      .get_one::<String>("template-id")
      .cloned();
    let redfish_endpoint = RedfishEndpoint {
      id: id.clone(),
      name,
      hostname,
      domain,
      fqdn,
      enabled: Some(enabled),
      user,
      password,
      use_ssdp: Some(use_ssdp),
      mac_required: Some(mac_required),
      mac_addr,
      ip_address,
      rediscover_on_update: Some(rediscover_on_update),
      template_id,
      r#type: None,
      uuid: None,
      discovery_info: None,
    };
    let redfish_endpoint_array = RedfishEndpointArray {
      redfish_endpoints: Some(vec![redfish_endpoint]),
    };
    ctx
      .backend
      .add_redfish_endpoint(&shasta_token, &redfish_endpoint_array)
      .await?;
    println!("Redfish endpoint for node '{}' added", id);
  } else {
    bail!("Unknown 'add' subcommand");
  }
  Ok(())
}
