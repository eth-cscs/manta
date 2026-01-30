use crate::cli::commands::{
  get_boot_parameters, get_cluster, get_configuration, get_group,
  get_hardware_cluster, get_hardware_node, get_images, get_kernel_parameters,
  get_nodes, get_session, get_template,
};
use crate::common::{
  authentication::get_api_token,
  authorization::{get_groups_names_available, validate_target_hsm_members},
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;
use manta_backend_dispatcher::interfaces::hsm::{
  group::GroupTrait, redfish_endpoint::RedfishEndpointTrait,
};
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
    let shasta_token = get_api_token(backend, site_name).await?;
    let group_name_arg_opt: Option<&String> = cli_get_groups.get_one("VALUE");
    let target_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await?;
    let output: String = cli_get_groups
      .get_one("output")
      .cloned()
      .expect("ERROR - 'output' argument is mandatory");
    get_group::exec(
      backend,
      &shasta_token,
      Some(&target_hsm_group_vec),
      &output,
    )
    .await?;
  } else if let Some(cli_get_hardware) = cli_get.subcommand_matches("hardware")
  {
    if let Some(cli_get_hardware_cluster) =
      cli_get_hardware.subcommand_matches("cluster")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let hsm_group_name_arg_opt: Option<&String> =
        cli_get_hardware_cluster.get_one("CLUSTER_NAME");
      let target_hsm_group_vec = get_groups_names_available(
        backend,
        &shasta_token,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
      )
      .await?;
      get_hardware_cluster::exec(
        backend.clone(),
        &shasta_token,
        target_hsm_group_vec.first().unwrap(),
        cli_get_hardware_cluster.get_one::<String>("output"),
      )
      .await;
    } else if let Some(cli_get_hardware_node) =
      cli_get_hardware.subcommand_matches("node")
    {
      let shasta_token = get_api_token(backend, site_name).await?;
      let xnames = cli_get_hardware_node
        .get_one::<String>("XNAMES")
        .expect("HSM group name is needed at this point");
      let xname_vec: Vec<String> =
        xnames.split(',').map(str::to_string).collect();
      validate_target_hsm_members(backend, &shasta_token, &xname_vec).await?;
      get_hardware_node::exec(
        backend,
        &shasta_token,
        xnames,
        cli_get_hardware_node.get_one::<String>("type"),
        cli_get_hardware_node.get_one::<String>("output"),
      )
      .await?;
    }
  } else if let Some(cli_get_configuration) =
    cli_get.subcommand_matches("configurations")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let gitea_token =
      crate::common::vault::http_client::fetch_shasta_vcs_token(
        &shasta_token,
        vault_base_url.expect("ERROR - vault base url is mandatory"),
        site_name,
      )
      .await
      .unwrap();
    let name: Option<&String> = cli_get_configuration.get_one::<String>("name");
    let pattern: Option<&String> =
      cli_get_configuration.get_one::<String>("pattern");
    let hsm_group_name_arg_rslt =
      cli_get_configuration.try_get_one("hsm-group");
    let target_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_rslt.unwrap_or(None),
      settings_hsm_group_name_opt,
    )
    .await?;
    let limit: Option<&u8> =
      if let Some(true) = cli_get_configuration.get_one("most-recent") {
        Some(&1)
      } else {
        cli_get_configuration.get_one::<u8>("limit")
      };
    let output: Option<&String> = cli_get_configuration.get_one("output");
    get_configuration::exec(
      backend,
      gitea_base_url,
      &gitea_token,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      name.map(String::as_str),
      pattern.map(String::as_str),
      &target_hsm_group_vec,
      None,
      None,
      limit,
      output.map(String::as_str),
      site_name,
    )
    .await?;
  } else if let Some(cli_get_session) = cli_get.subcommand_matches("sessions") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_group_name_arg_opt: Option<&String> =
      cli_get_session.get_one("hsm-group");
    let limit: Option<&u8> =
      if let Some(true) = cli_get_session.get_one("most-recent") {
        Some(&1)
      } else {
        cli_get_session.get_one::<u8>("limit")
      };
    let xname_vec_arg: Vec<&str> = cli_get_session
      .get_one::<String>("xnames")
      .map(|xname_str| xname_str.split(',').map(|xname| xname.trim()).collect())
      .unwrap_or_default();
    let min_age_opt: Option<&String> =
      cli_get_session.get_one::<String>("min-age");
    let max_age_opt: Option<&String> =
      cli_get_session.get_one::<String>("max-age");
    let mut type_opt: Option<String> = cli_get_session.get_one("type").cloned();
    if type_opt == Some("runtime".to_string()) {
      type_opt = Some("dynamic".to_string())
    }
    let status_opt: Option<&String> =
      cli_get_session.get_one::<String>("status");
    let name_opt: Option<&String> = cli_get_session.get_one::<String>("name");
    let output_opt: Option<&String> = cli_get_session.get_one("output");
    get_session::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_arg_opt.map(|v| vec![v.clone()]),
      Some(xname_vec_arg),
      min_age_opt,
      max_age_opt,
      type_opt.as_ref(),
      status_opt,
      name_opt,
      limit,
      output_opt,
    )
    .await?;
  } else if let Some(cli_get_template) = cli_get.subcommand_matches("templates")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let name: Option<&String> = cli_get_template.get_one::<String>("name");
    let hsm_group_name_arg_opt = cli_get_template.try_get_one("hsm-group");
    let output: &String = cli_get_template
      .get_one("output")
      .expect("ERROR - output must be a valid value");
    let target_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt.unwrap_or(None),
      settings_hsm_group_name_opt,
    )
    .await?;
    let hsm_member_vec = backend
      .get_member_vec_from_group_name_vec(&shasta_token, &target_hsm_group_vec)
      .await?;
    let limit_number_opt =
      if let Some(limit) = cli_get_template.get_one("limit") {
        Some(limit)
      } else if let Some(true) = cli_get_template.get_one("most-recent") {
        Some(&1)
      } else {
        None
      };
    get_template::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      &hsm_member_vec,
      name.map(String::as_str),
      limit_number_opt,
      output,
    )
    .await?;
  } else if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_group_name_arg_opt: Option<&String> =
      cli_get_cluster.get_one("HSM_GROUP_NAME");
    let target_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await?;
    let status: Option<&String> = cli_get_cluster.get_one("status");
    let nids_only = cli_get_cluster.get_flag("nids-only-one-line");
    let xnames_only = cli_get_cluster.get_flag("xnames-only-one-line");
    let output: Option<&String> = cli_get_cluster.get_one("output");
    let summary_status = cli_get_cluster.get_flag("summary-status");
    get_cluster::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      status.map(String::as_str),
      nids_only,
      xnames_only,
      output.map(String::as_str),
      summary_status,
    )
    .await?;
  } else if let Some(cli_get_nodes) = cli_get.subcommand_matches("nodes") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let xname_requested: &str = cli_get_nodes
      .get_one::<String>("VALUE")
      .expect("The 'xnames' argument must have values");
    let is_include_siblings = cli_get_nodes.get_flag("include-siblings");
    let nids_only = cli_get_nodes.get_flag("nids-only-one-line");
    let status: Option<&String> = cli_get_nodes.get_one("status");
    let output: Option<&String> = cli_get_nodes.get_one("output");
    let status_summary = cli_get_nodes.get_flag("summary-status");
    get_nodes::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      xname_requested,
      status,
      is_include_siblings,
      nids_only,
      false,
      output,
      status_summary,
    )
    .await?;
  } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let id: Option<&String> = cli_get_images.get_one::<String>("id");
    let hsm_group_name_arg_opt = cli_get_images.try_get_one("hsm-group");
    let limit: Option<&u8> = cli_get_images.get_one::<u8>("limit");
    let target_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt.unwrap_or(None),
      settings_hsm_group_name_opt,
    )
    .await?;
    get_images::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      id.map(String::as_str),
      limit,
    )
    .await?;
  } else if let Some(cli_get_boot_parameters) =
    cli_get.subcommand_matches("boot-parameters")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_group_name_arg_opt: Option<&String> =
      cli_get_boot_parameters.get_one("hsm-group");
    let nodes: &String = if hsm_group_name_arg_opt.is_some() {
      let hsm_group_name_vec = get_groups_names_available(
        backend,
        &shasta_token,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
      )
      .await?;
      let hsm_members_rslt = backend
        .get_member_vec_from_group_name_vec(&shasta_token, &hsm_group_name_vec)
        .await;
      match hsm_members_rslt {
        Ok(hsm_members) => &hsm_members.join(","),
        Err(e) => {
          eprintln!(
            "ERROR - could not fetch HSM groups members. Reason:\n{}",
            e.to_string()
          );
          std::process::exit(1);
        }
      }
    } else {
      cli_get_boot_parameters
        .get_one::<String>("nodes")
        .expect("Neither HSM group nor nodes defined")
    };
    let boot_parameters_vec: Vec<BootParameters> = get_boot_parameters::exec(
      backend,
      &shasta_token,
      nodes,
      None,
      None,
      None,
      None,
      None,
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&boot_parameters_vec)?);
  } else if let Some(cli_get_kernel_parameters) =
    cli_get.subcommand_matches("kernel-parameters")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_group_name_arg_opt: Option<&String> =
      cli_get_kernel_parameters.get_one("hsm-group");
    let filter_opt: Option<&String> =
      cli_get_kernel_parameters.get_one("filter");
    let nodes: &String = if hsm_group_name_arg_opt.is_some() {
      let hsm_group_name_vec = get_groups_names_available(
        backend,
        &shasta_token,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
      )
      .await?;
      let hsm_members_rslt = backend
        .get_member_vec_from_group_name_vec(&shasta_token, &hsm_group_name_vec)
        .await;
      match hsm_members_rslt {
        Ok(hsm_members) => &hsm_members.join(","),
        Err(e) => {
          eprintln!(
            "ERROR - could not fetch HSM groups members. Reason:\n{}",
            e.to_string()
          );
          std::process::exit(1);
        }
      }
    } else {
      cli_get_kernel_parameters
        .get_one::<String>("nodes")
        .expect("Neither HSM group nor nodes defined")
    };
    let output: &String = cli_get_kernel_parameters
      .get_one("output")
      .expect("ERROR - output value missing");
    let _ = get_kernel_parameters::exec(
      backend,
      &shasta_token,
      nodes,
      filter_opt,
      output,
    )
    .await;
  } else if let Some(cli_get_redfish_endopints) =
    cli_get.subcommand_matches("redfish-endpoints")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let id = cli_get_redfish_endopints
      .get_one::<String>("id")
      .map(|x| x.as_str());
    let fqdn = cli_get_redfish_endopints
      .get_one::<String>("fqdn")
      .map(|x| x.as_str());
    let uuid = cli_get_redfish_endopints
      .get_one::<String>("uuid")
      .map(|x| x.as_str());
    let macaddr = cli_get_redfish_endopints
      .get_one::<String>("macaddr")
      .map(|x| x.as_str());
    let ipaddress = cli_get_redfish_endopints
      .get_one::<String>("ipaddress")
      .map(|x| x.as_str());
    let redfish_endpoints = backend
      .get_redfish_endpoints(
        &shasta_token,
        id,
        fqdn,
        None,
        uuid,
        macaddr,
        ipaddress,
        None,
      )
      .await?;
    println!("{}", serde_json::to_string_pretty(&redfish_endpoints)?);
  }
  Ok(())
}
