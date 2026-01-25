use anyhow::Error;
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait,
    hsm::{
      component::ComponentTrait, group::GroupTrait,
      redfish_endpoint::RedfishEndpointTrait,
    },
  },
  types::{
    HWInventoryByLocationList,
    bss::BootParameters,
    hsm::inventory::{RedfishEndpoint, RedfishEndpointArray},
  },
};
use std::{
  fs::File,
  io::{BufReader, IsTerminal},
  path::PathBuf,
};

use clap::Command;
use config::Config;

use crate::{
  cli::{
    commands::{add_node, validate_local_repo},
    parsers,
  },
  common::{
    authentication::get_api_token,
    authorization::{get_groups_names_available, validate_target_hsm_members},
    config::types::MantaConfiguration,
    kafka::Kafka,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use super::commands::{
  self, add_group, add_hw_component_cluster, add_kernel_parameters,
  add_nodes_to_hsm_groups, console_cfs_session_image_target_ansible,
  console_node, delete_and_cancel_session,
  delete_configurations_and_derivatives, delete_group,
  delete_hw_component_cluster, delete_images, delete_kernel_parameters,
  get_boot_parameters, get_cluster, get_configuration, get_hardware_node,
  get_images, get_kernel_parameters, get_nodes, get_session, get_template,
  migrate_backup, migrate_nodes_between_hsm_groups, power_off_cluster,
  power_off_nodes, power_on_cluster, power_on_nodes, power_reset_cluster,
  power_reset_nodes, remove_nodes_from_hsm_groups, update_boot_parameters,
};
use serde_json::Value;

pub async fn process_cli(
  cli: Command,
  backend: StaticBackendDispatcher,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  vault_base_url: Option<&String>,
  gitea_base_url: &str,
  settings_hsm_group_name_opt: Option<&String>,
  k8s_api_url: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
  settings: &Config,
  configuration: &MantaConfiguration,
) -> Result<(), Error> {
  let site_name: String = settings.get("site").map_err(|_| Error::msg("'site' value in configuration file is missing or does not have a value. Exit"))?;

  let cli_root = cli.clone().get_matches();

  if let Some(cli_config) = cli_root.subcommand_matches("config") {
    if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
      parsers::config::show::process_subcommand(&backend, &site_name, settings)
        .await?
    } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
      if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm")
      {
        parsers::config::set_hsm::process_subcommand(
          cli_config_set_hsm,
          &backend,
          &site_name,
        )
        .await?
      }
      if let Some(cli_config_set_parent_hsm) =
        cli_config_set.subcommand_matches("parent-hsm")
      {
        parsers::config::set_parent_hsm::process_subcommand(
          cli_config_set_parent_hsm,
          &backend,
          &site_name,
        )
        .await?;
      }
      if let Some(cli_config_set_site) =
        cli_config_set.subcommand_matches("site")
      {
        parsers::config::set_site::process_subcommand(cli_config_set_site)
          .await?;
      }
      if let Some(cli_config_set_log) = cli_config_set.subcommand_matches("log")
      {
        parsers::config::set_log::process_subcommand(cli_config_set_log)
          .await?;
      }
    } else if let Some(cli_config_unset) =
      cli_config.subcommand_matches("unset")
    {
      if let Some(_cli_config_unset_hsm) =
        cli_config_unset.subcommand_matches("hsm")
      {
        parsers::config::unset_hsm::process_subcommand().await?;
      }
      if let Some(_cli_config_unset_parent_hsm) =
        cli_config_unset.subcommand_matches("parent-hsm")
      {
        parsers::config::unset_parent_hsm::process_subcommand(
          &backend, &site_name,
        )
        .await?;
      }
      if let Some(_cli_config_unset_auth) =
        cli_config_unset.subcommand_matches("auth")
      {
        parsers::config::unset_auth::process_subcommand().await?;
      }
    } else if let Some(cli_config_generate_autocomplete) =
      cli_config.subcommand_matches("gen-autocomplete")
    {
      parsers::config::generate_shell_autocompletion::process_subcommand(
        cli,
        cli_config_generate_autocomplete,
      )
      .await?;
    }
  } else {
    if let Some(cli_power) = cli_root.subcommand_matches("power") {
      if let Some(cli_power_on) = cli_power.subcommand_matches("on") {
        if let Some(cli_power_on_cluster) =
          cli_power_on.subcommand_matches("cluster")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let hsm_group_name_arg = cli_power_on_cluster
            .get_one::<String>("CLUSTER_NAME")
            .expect("The 'cluster name' argument must have a value");

          let target_hsm_group_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            Some(hsm_group_name_arg),
            settings_hsm_group_name_opt,
          )
          .await?;

          let target_hsm_group = target_hsm_group_vec
            .first()
            .expect("The 'cluster name' argument must have a value");

          let assume_yes: bool = cli_power_on_cluster.get_flag("assume-yes");

          let output: &str =
            cli_power_on_cluster.get_one::<String>("output").unwrap();

          power_on_cluster::exec(
            backend,
            &shasta_token,
            target_hsm_group,
            assume_yes,
            output,
            kafka_audit_opt,
          )
          .await;
        } else if let Some(cli_power_on_node) =
          cli_power_on.subcommand_matches("nodes")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let xname_requested: &str = cli_power_on_node
            .get_one::<String>("VALUE")
            .expect("The 'xnames' argument must have values");

          let assume_yes: bool = cli_power_on_node.get_flag("assume-yes");

          let output: &str =
            cli_power_on_node.get_one::<String>("output").unwrap();

          power_on_nodes::exec(
            &backend,
            &shasta_token,
            xname_requested,
            assume_yes,
            output,
            kafka_audit_opt,
          )
          .await;
        }
      } else if let Some(cli_power_off) = cli_power.subcommand_matches("off") {
        if let Some(cli_power_off_cluster) =
          cli_power_off.subcommand_matches("cluster")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let hsm_group_name_arg = cli_power_off_cluster
            .get_one::<String>("CLUSTER_NAME")
            .expect("The 'cluster name' argument must have a value");

          let force = cli_power_off_cluster
            .get_one::<bool>("graceful")
            .expect("The 'graceful' argument must have a value");

          let output: &str =
            cli_power_off_cluster.get_one::<String>("output").unwrap();

          let target_hsm_group_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            Some(hsm_group_name_arg),
            settings_hsm_group_name_opt,
          )
          .await?;

          let target_hsm_group = target_hsm_group_vec
            .first()
            .expect("The 'cluster name' argument must have a value");

          let assume_yes: bool = cli_power_off_cluster.get_flag("assume-yes");

          power_off_cluster::exec(
            &backend,
            &shasta_token,
            target_hsm_group,
            *force,
            assume_yes,
            output,
            kafka_audit_opt,
          )
          .await;
        } else if let Some(cli_power_off_node) =
          cli_power_off.subcommand_matches("nodes")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let xname_requested: &str = cli_power_off_node
            .get_one::<String>("VALUE")
            .expect("The 'xnames' argument must have values");

          let force = cli_power_off_node
            .get_one::<bool>("graceful")
            .expect("The 'graceful' argument must have a value");

          let assume_yes: bool = cli_power_off_node.get_flag("assume-yes");

          let output: &str =
            cli_power_off_node.get_one::<String>("output").unwrap();

          power_off_nodes::exec(
            &backend,
            &shasta_token,
            xname_requested,
            *force,
            assume_yes,
            output,
            kafka_audit_opt,
          )
          .await;
        }
      } else if let Some(cli_power_reset) =
        cli_power.subcommand_matches("reset")
      {
        if let Some(cli_power_reset_cluster) =
          cli_power_reset.subcommand_matches("cluster")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let hsm_group_name_arg = cli_power_reset_cluster
            .get_one::<String>("CLUSTER_NAME")
            .expect("The 'cluster name' argument must have a value");

          let force = cli_power_reset_cluster
            .get_one::<bool>("graceful")
            .expect("The 'graceful' argument must have a value");

          let output: &str =
            cli_power_reset_cluster.get_one::<String>("output").unwrap();

          let target_hsm_group_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            Some(hsm_group_name_arg),
            settings_hsm_group_name_opt,
          )
          .await?;

          let target_hsm_group = target_hsm_group_vec
            .first()
            .expect("Power off cluster must operate against a cluster");

          let assume_yes: bool = cli_power_reset_cluster.get_flag("assume-yes");

          power_reset_cluster::exec(
            backend,
            &shasta_token,
            target_hsm_group,
            *force,
            assume_yes,
            output,
            kafka_audit_opt,
          )
          .await;
        } else if let Some(cli_power_reset_node) =
          cli_power_reset.subcommand_matches("nodes")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let xname_requested: &str = cli_power_reset_node
            .get_one::<String>("VALUE")
            .expect("The 'xnames' argument must have values");

          let force = cli_power_reset_node
            .get_one::<bool>("graceful")
            .expect("The 'graceful' argument must have a value");

          let assume_yes: bool = cli_power_reset_node.get_flag("assume-yes");

          let output: &str =
            cli_power_reset_node.get_one::<String>("output").unwrap();

          power_reset_nodes::exec(
            &backend,
            &shasta_token,
            xname_requested,
            *force,
            assume_yes,
            output,
            kafka_audit_opt,
          )
          .await;
        }
      }
    } else if let Some(cli_add) = cli_root.subcommand_matches("add") {
      if let Some(cli_add_node) = cli_add.subcommand_matches("node") {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id = cli_add_node
          .get_one::<String>("id")
          .expect("ERROR - 'id' argument is mandatory");

        let group = cli_add_node
          .get_one::<String>("group")
          .expect("ERROR - 'group' argument is mandatory");

        let hardware_file_opt = cli_add_node.get_one::<PathBuf>("hardware");

        let hw_inventory_opt: Option<HWInventoryByLocationList> =
          if let Some(hardware_file) = hardware_file_opt {
            let file = File::open(hardware_file)?;
            let reader = BufReader::new(file);

            let hw_inventory_value: serde_json::Value =
              serde_json::from_reader(reader).unwrap();

            Some(
              serde_json::from_value::<HWInventoryByLocationList>(
                hw_inventory_value,
              )
              .expect("ERROR - Could not parse hardware inventory file"),
            )
          } else {
            None
          };

        let arch_opt = cli_add_node.get_one::<String>("arch").cloned();

        let enabled = cli_add_node
          .get_one::<bool>("disabled")
          .cloned()
          .unwrap_or(true);

        let add_node_rslt = add_node::exec(
          &backend,
          &shasta_token,
          id,
          group,
          enabled,
          arch_opt,
          hw_inventory_opt,
          kafka_audit_opt,
        )
        .await;

        if let Err(error) = add_node_rslt {
          // Could not add xname to group. Reset operation by removing the node
          eprintln!(
            "ERROR - operation to add node '{id}' to group '{group}' failed. Reason:\n{error}\nRollback operation"
          );
          let delete_node_rslt = backend
            .delete_node(shasta_token.as_str(), id.clone().as_str())
            .await;

          eprintln!("Try to delete node '{}'", id);

          if delete_node_rslt.is_ok() {
            eprintln!("Node '{}' deleted", id);
          }

          std::process::exit(1);
        }

        log::info!("Node '{}' created", id);

        // Add node to group
        let new_group_members_rslt = backend
          .add_members_to_group(&shasta_token, group, &[id])
          .await;

        if let Err(error) = &new_group_members_rslt {
          eprintln!(
            "ERROR - Could not add node to group. Reason:\n{:#?}",
            error
          );
        }

        println!("Node '{}' created and added to group '{}'", id, group,);
      } else if let Some(cli_add_group) = cli_add.subcommand_matches("group") {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let label = cli_add_group
          .get_one::<String>("label")
          .expect("ERROR - 'label' argument is mandatory");

        let description: Option<&String> = cli_add_group.get_one("description");
        let node_expression: Option<&String> =
          cli_add_group.get_one::<String>("nodes");

        add_group::exec(
          backend,
          &shasta_token,
          label,
          description,
          node_expression,
          true,
          false,
          kafka_audit_opt,
        )
        .await;
      } else if let Some(cli_add_hw_configuration) =
        cli_add.subcommand_matches("hardware")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let target_hsm_group_name_arg_opt: Option<&String> =
          cli_add_hw_configuration.get_one("target-cluster");
        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          target_hsm_group_name_arg_opt,
          settings_hsm_group_name_opt,
        )
        .await?;

        let parent_hsm_group_name_arg_opt: Option<&String> =
          cli_add_hw_configuration.get_one("parent-cluster");
        let parent_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          parent_hsm_group_name_arg_opt,
          settings_hsm_group_name_opt,
        )
        .await?;
        let _: Option<String> =
          cli_add_hw_configuration.get_one("target-cluster").cloned();
        let dryrun = cli_add_hw_configuration.get_flag("dry-run");

        let create_hsm_group = *cli_add_hw_configuration
          .get_one::<bool>("create-hsm-group")
          .unwrap_or(&false);

        add_hw_component_cluster::exec(
          &backend,
          &shasta_token,
          target_hsm_group_vec.first().unwrap(),
          parent_hsm_group_vec.first().unwrap(),
          cli_add_hw_configuration
            .get_one::<String>("pattern")
            .unwrap(),
          dryrun,
          create_hsm_group,
        )
        .await?;
      } else if let Some(cli_add_boot_parameters) =
        cli_add.subcommand_matches("boot-parameters")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hosts = cli_add_boot_parameters
          .get_one::<String>("hosts")
          .expect("ERROR - 'hosts' argument is mandatory");
        let macs: Option<String> =
          cli_add_boot_parameters.get_one("macs").cloned();
        let nids: Option<String> =
          cli_add_boot_parameters.get_one("nids").cloned();
        let params = cli_add_boot_parameters
          .get_one::<String>("params")
          .expect("ERROR - 'params' argument is mandatory")
          .clone();
        let kernel = cli_add_boot_parameters
          .get_one::<String>("kernel")
          .expect("ERROR - 'kernel' argument is mandatory")
          .clone();
        let initrd = cli_add_boot_parameters
          .get_one::<String>("initrd")
          .expect("ERROR - 'initrd' argument is mandatory")
          .clone();
        let cloud_init = cli_add_boot_parameters
          .get_one::<Value>("cloud-init")
          .cloned();

        let _dry_run = cli_add_boot_parameters.get_flag("dry-run");
        let _assume_yes: bool = cli_add_boot_parameters.get_flag("assume-yes");

        let host_vec = hosts
          .split(",")
          .map(|value| value.trim().to_string())
          .collect::<Vec<String>>();

        let mac_vec = macs.map(|x| {
          x.split(",")
            .map(|value| value.trim().to_string())
            .collect::<Vec<String>>()
        });

        let nid_vec: Option<Vec<u32>> = nids.map(|x| {
          x.split(",")
            .map(|value| value.trim().parse().unwrap())
            .collect()
        });

        let boot_parameters = BootParameters {
          hosts: host_vec,
          macs: mac_vec,
          nids: nid_vec,
          params,
          kernel,
          initrd,
          cloud_init,
        };

        backend
          .add_bootparameters(&shasta_token, &boot_parameters)
          .await?;

        println!("Boot parameters created successfully");
      } else if let Some(cli_add_kernel_parameters) =
        cli_add.subcommand_matches("kernel-parameters")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_group_name_arg_opt =
          cli_add_kernel_parameters.get_one("hsm-group");

        let nodes: &String = if hsm_group_name_arg_opt.is_some() {
          let hsm_group_name_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
          )
          .await?;

          let hsm_members_rslt = backend
            .get_member_vec_from_group_name_vec(
              &shasta_token,
              &hsm_group_name_vec,
            )
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
          cli_add_kernel_parameters
            .get_one::<String>("nodes")
            .expect("Neither HSM group nor nodes defined")
        };

        let kernel_parameters = cli_add_kernel_parameters
          .get_one::<String>("VALUE")
          .unwrap(); // clap should validate the argument

        let overwrite: bool = cli_add_kernel_parameters.get_flag("overwrite");
        let assume_yes: bool = cli_add_kernel_parameters.get_flag("assume-yes");
        let do_not_reboot: bool =
          cli_add_kernel_parameters.get_flag("do-not-reboot");

        let dryrun = cli_add_kernel_parameters.get_flag("dry-run");

        let result = add_kernel_parameters::exec(
          backend,
          &shasta_token,
          kernel_parameters,
          nodes,
          overwrite,
          assume_yes,
          do_not_reboot,
          kafka_audit_opt,
          dryrun,
        )
        .await;

        match result {
          Ok(_) => {}
          Err(error) => eprintln!("{}", error),
        }
      } else if let Some(cli_add_redfish_endpoint) =
        cli_add.subcommand_matches("redfish-endpoint")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id = cli_add_redfish_endpoint
          .get_one::<String>("id")
          .expect("ERROR - 'id' argument is mandatory")
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

        let mac_required: bool =
          cli_add_redfish_endpoint.get_flag("mac-required");

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

        backend
          .add_redfish_endpoint(&shasta_token, &redfish_endpoint_array)
          .await?;

        println!("Redfish endpoint for node '{}' added", id);
      }
    } else if let Some(cli_update) = cli_root.subcommand_matches("update") {
      if let Some(cli_update_boot_parameters) =
        cli_update.subcommand_matches("boot-parameters")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hosts: &String = cli_update_boot_parameters
          .get_one("hosts")
          .expect("ERROR - 'hosts' argument is mandatory");

        // FIXME: Ignoring nids and macs to avoid checking if tenant has access to the nodes
        // using the nids or macs

        /* let nids: Option<&String> = cli_update_boot_parameters.get_one("nids");
        let macs: Option<&String> = cli_update_boot_parameters.get_one("macs"); */
        let params: Option<&String> =
          cli_update_boot_parameters.get_one("params");
        let kernel: Option<&String> =
          cli_update_boot_parameters.get_one("kernel");
        let initrd: Option<&String> =
          cli_update_boot_parameters.get_one("initrd");
        let xname_vec = hosts
          .split(",")
          .map(|value| value.trim().to_string())
          .collect();

        // Validate user has access to the list of xnames requested
        validate_target_hsm_members(&backend, &shasta_token, &xname_vec).await;

        let result = update_boot_parameters::exec(
          &backend,
          &shasta_token,
          hosts,
          /* nids,
          macs, */
          None,
          None,
          params,
          kernel,
          initrd,
          kafka_audit_opt,
        )
        .await;

        match result {
          Ok(_) => {}
          Err(error) => eprintln!("{}", error),
        }
      } else if let Some(cli_update_redfish_endpoint) =
        cli_update.subcommand_matches("redfish-endpoint")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id: String = cli_update_redfish_endpoint
          .get_one("id")
          .cloned()
          .expect("ERROR - 'id' argument is mandatory");

        let name: Option<String> =
          cli_update_redfish_endpoint.get_one("name").cloned();

        let hostname: Option<String> =
          cli_update_redfish_endpoint.get_one("hostname").cloned();

        let domain: Option<String> =
          cli_update_redfish_endpoint.get_one("domain").cloned();

        let fqdn: Option<String> =
          cli_update_redfish_endpoint.get_one("fqdn").cloned();

        let enabled: bool = cli_update_redfish_endpoint.get_flag("enabled");

        let user: Option<String> =
          cli_update_redfish_endpoint.get_one("user").cloned();

        let password: Option<String> =
          cli_update_redfish_endpoint.get_one("password").cloned();

        let use_ssdp: bool = cli_update_redfish_endpoint.get_flag("use-ssdp");

        let mac_required: bool =
          cli_update_redfish_endpoint.get_flag("mac-required");

        let mac_addr: Option<String> =
          cli_update_redfish_endpoint.get_one("macaddr").cloned();

        let ip_address: Option<String> =
          cli_update_redfish_endpoint.get_one("ipaddress").cloned();

        let rediscover_on_update: bool =
          cli_update_redfish_endpoint.get_flag("rediscover-on-update");

        let template_id: Option<String> =
          cli_update_redfish_endpoint.get_one("template-id").cloned();

        let redfish_endpoint = RedfishEndpoint {
          id,
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

        backend
          .update_redfish_endpoint(&shasta_token, &redfish_endpoint)
          .await?
      }
    } else if let Some(cli_get) = cli_root.subcommand_matches("get") {
      if let Some(cli_get_groups) = cli_get.subcommand_matches("groups") {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let group_name_arg_opt: Option<&String> =
          cli_get_groups.get_one("VALUE");

        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          group_name_arg_opt,
          settings_hsm_group_name_opt,
        )
        .await?;

        let output: String = cli_get_groups
          .get_one("output")
          .cloned()
          .expect("ERROR - 'output' argument is mandatory");

        let hsm_group_vec: Vec<String> = target_hsm_group_vec;

        commands::get_group::exec(
          &backend,
          &shasta_token,
          Some(&hsm_group_vec),
          &output,
        )
        .await?;
      } else if let Some(cli_get_hardware) =
        cli_get.subcommand_matches("hardware")
      {
        if let Some(cli_get_hardware_cluster) =
          cli_get_hardware.subcommand_matches("cluster")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let hsm_group_name_arg_opt: Option<&String> =
            cli_get_hardware_cluster.get_one("CLUSTER_NAME");
          let target_hsm_group_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
          )
          .await?;

          commands::get_hardware_cluster::exec(
            backend,
            &shasta_token,
            target_hsm_group_vec.first().unwrap(),
            cli_get_hardware_cluster.get_one::<String>("output"),
          )
          .await;
        } else if let Some(cli_get_hardware_node) =
          cli_get_hardware.subcommand_matches("node")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let xnames = cli_get_hardware_node
            .get_one::<String>("XNAMES")
            .expect("HSM group name is needed at this point");

          let xname_vec: Vec<String> =
            xnames.split(',').map(str::to_string).collect();

          validate_target_hsm_members(&backend, &shasta_token, &xname_vec)
            .await;

          get_hardware_node::exec(
            &backend,
            &shasta_token,
            xnames,
            cli_get_hardware_node.get_one::<String>("type"),
            cli_get_hardware_node.get_one::<String>("output"),
          )
          .await;
        }
      } else if let Some(cli_get_configuration) =
        cli_get.subcommand_matches("configurations")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        // FIXME: gitea auth token should be calculated before calling this function
        let gitea_token =
          crate::common::vault::http_client::fetch_shasta_vcs_token(
            &shasta_token,
            vault_base_url.expect("ERROR - vault base url is mandatory"),
            &site_name,
          )
          .await
          .unwrap();

        let name: Option<&String> =
          cli_get_configuration.get_one::<String>("name");

        let pattern: Option<&String> =
          cli_get_configuration.get_one::<String>("pattern");

        let hsm_group_name_arg_rslt =
          cli_get_configuration.try_get_one("hsm-group");

        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          hsm_group_name_arg_rslt.unwrap_or(None),
          settings_hsm_group_name_opt,
        )
        .await?;

        dbg!(&target_hsm_group_vec);

        let limit: Option<&u8> =
          if let Some(true) = cli_get_configuration.get_one("most-recent") {
            Some(&1)
          } else {
            cli_get_configuration.get_one::<u8>("limit")
          };

        let output: Option<&String> = cli_get_configuration.get_one("output");

        get_configuration::exec(
          &backend,
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
          &site_name,
        )
        .await;
      } else if let Some(cli_get_session) =
        cli_get.subcommand_matches("sessions")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

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
          .map(|xname_str| {
            xname_str.split(',').map(|xname| xname.trim()).collect()
          })
          .unwrap_or_default();

        let min_age_opt: Option<&String> =
          cli_get_session.get_one::<String>("min-age");

        let max_age_opt: Option<&String> =
          cli_get_session.get_one::<String>("max-age");

        let mut type_opt: Option<String> =
          cli_get_session.get_one("type").cloned();

        // Map runtime type to dynamic as runtime
        if type_opt == Some("runtime".to_string()) {
          type_opt = Some("dynamic".to_string())
        }

        let status_opt: Option<&String> =
          cli_get_session.get_one::<String>("status");

        let name_opt: Option<&String> =
          cli_get_session.get_one::<String>("name");

        let output_opt: Option<&String> = cli_get_session.get_one("output");

        get_session::exec(
          &backend,
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
        .await;
      } else if let Some(cli_get_template) =
        cli_get.subcommand_matches("templates")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let name: Option<&String> = cli_get_template.get_one::<String>("name");

        let hsm_group_name_arg_opt = cli_get_template.try_get_one("hsm-group");

        let output: &String = cli_get_template
          .get_one("output")
          .expect("ERROR - output must be a valid value");

        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          hsm_group_name_arg_opt.unwrap_or(None),
          settings_hsm_group_name_opt,
        )
        .await?;

        let hsm_member_vec = backend
          .get_member_vec_from_group_name_vec(
            &shasta_token,
            &target_hsm_group_vec,
          )
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
          &backend,
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &target_hsm_group_vec,
          &hsm_member_vec,
          name.map(String::as_str),
          limit_number_opt,
          output,
        )
        .await;
      } else if let Some(cli_get_cluster) =
        cli_get.subcommand_matches("cluster")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_group_name_arg_opt: Option<&String> =
          cli_get_cluster.get_one("HSM_GROUP_NAME");
        let target_hsm_group_vec = get_groups_names_available(
          &backend,
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
          &backend,
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
        .await;
      } else if let Some(cli_get_nodes) = cli_get.subcommand_matches("nodes") {
        // Get list of nodes from cli argument
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let xname_requested: &str = cli_get_nodes
          .get_one::<String>("VALUE")
          .expect("The 'xnames' argument must have values");

        let is_include_siblings = cli_get_nodes.get_flag("include-siblings");
        let nids_only = cli_get_nodes.get_flag("nids-only-one-line");
        let status: Option<&String> = cli_get_nodes.get_one("status");
        let output: Option<&String> = cli_get_nodes.get_one("output");
        let status_summary = cli_get_nodes.get_flag("summary-status");

        get_nodes::exec(
          &backend,
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
        .await;
      } else if let Some(cli_get_images) = cli_get.subcommand_matches("images")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id: Option<&String> = cli_get_images.get_one::<String>("id");

        let hsm_group_name_arg_opt = cli_get_images.try_get_one("hsm-group");

        let limit: Option<&u8> = cli_get_images.get_one::<u8>("limit");

        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          hsm_group_name_arg_opt.unwrap_or(None),
          settings_hsm_group_name_opt,
        )
        .await?;

        get_images::exec(
          &backend,
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &target_hsm_group_vec,
          id.map(String::as_str),
          limit,
        )
        .await;
      } else if let Some(cli_get_boot_parameters) =
        cli_get.subcommand_matches("boot-parameters")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_group_name_arg_opt: Option<&String> =
          cli_get_boot_parameters.get_one("hsm-group");
        let nodes: &String = if hsm_group_name_arg_opt.is_some() {
          let hsm_group_name_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
          )
          .await?;

          let hsm_members_rslt = backend
            .get_member_vec_from_group_name_vec(
              &shasta_token,
              &hsm_group_name_vec,
            )
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

        let boot_parameters_vec: Vec<BootParameters> =
          get_boot_parameters::exec(
            &backend,
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
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_group_name_arg_opt: Option<&String> =
          cli_get_kernel_parameters.get_one("hsm-group");
        let filter_opt: Option<&String> =
          cli_get_kernel_parameters.get_one("filter");

        let nodes: &String = if hsm_group_name_arg_opt.is_some() {
          let hsm_group_name_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
          )
          .await?;

          let hsm_members_rslt = backend
            .get_member_vec_from_group_name_vec(
              &shasta_token,
              &hsm_group_name_vec,
            )
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
          &backend,
          &shasta_token,
          nodes,
          filter_opt,
          output,
        )
        .await;
      } else if let Some(cli_get_redfish_endopints) =
        cli_get.subcommand_matches("redfish-endpoints")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id = cli_get_redfish_endopints
          .get_one::<String>("id")
          .map(|x| x.as_str());
        let fqdn = cli_get_redfish_endopints
          .get_one::<String>("fqdn")
          .map(|x| x.as_str());

        // FIXME: ignoring 'type' argument to ship faster and simplify tests

        let uuid = cli_get_redfish_endopints
          .get_one::<String>("uuid")
          .map(|x| x.as_str());
        let macaddr = cli_get_redfish_endopints
          .get_one::<String>("macaddr")
          .map(|x| x.as_str());
        let ipaddress = cli_get_redfish_endopints
          .get_one::<String>("ipaddress")
          .map(|x| x.as_str());

        // FIXME: ignoring 'last-status' argument to ship faster and simplify tests

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
    } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
      crate::cli::parsers::apply::parse_subcommand(
        cli_apply,
        backend,
        &site_name,
        shasta_base_url,
        shasta_root_cert,
        &vault_base_url.unwrap(),
        gitea_base_url,
        settings_hsm_group_name_opt,
        &k8s_api_url.unwrap(),
        kafka_audit_opt,
        settings,
        configuration,
      )
      .await?
    } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
      let shasta_token = get_api_token(&backend, &site_name).await?;

      // Get all HSM groups the user has access
      let user_input = cli_log
        .get_one::<String>("VALUE")
        .expect("ERROR - value is mandatory");

      let timestamps = cli_log.get_flag("timestamps");

      let group_available_vec =
        backend.get_group_available(&shasta_token).await?;

      let site = configuration
        .sites
        .get(&configuration.site.clone())
        .unwrap();

      let k8s_details = site
        .k8s
        .as_ref()
        .expect("ERROR - k8s section not found in configuration");

      match commands::log::exec(
        &backend,
        &site_name,
        &shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &group_available_vec,
        user_input,
        timestamps,
        k8s_details,
      )
      .await
      {
        Ok(_) => {
          println!("Log streaming ended");
        }
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
      if let Some(cli_console_node) = cli_console.subcommand_matches("node") {
        if !std::io::stdout().is_terminal() {
          eprintln!("This command needs to run in interactive mode. Exit");
          std::process::exit(1);
        }
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let site = configuration
          .sites
          .get(&configuration.site.clone())
          .unwrap();

        console_node::exec(
          &backend,
          &site_name,
          &shasta_token,
          cli_console_node.get_one::<String>("XNAME").unwrap(),
          &site
            .k8s
            .as_ref()
            .expect("ERROR - k8s section not found in configuration"), // FIXME:
        )
        .await;
      } else if let Some(cli_console_target_ansible) =
        cli_console.subcommand_matches("target-ansible")
      {
        if !std::io::stdout().is_terminal() {
          eprintln!("This command needs to run in interactive mode. Exit");
          std::process::exit(1);
        }
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          None,
          settings_hsm_group_name_opt,
        )
        .await?;

        let site = configuration
          .sites
          .get(&configuration.site.clone())
          .unwrap();

        console_cfs_session_image_target_ansible::exec(
          &backend,
          &site_name,
          &target_hsm_group_vec,
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          cli_console_target_ansible
            .get_one::<String>("SESSION_NAME")
            .unwrap(),
          &site
            .k8s
            .as_ref()
            .expect("ERROR - k8s section not found in configuration"), // FIXME:
        )
        .await;
      }
    } else if let Some(cli_migrate) = cli_root.subcommand_matches("migrate") {
      if let Some(cli_migrate_nodes) = cli_migrate.subcommand_matches("nodes") {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let dry_run: bool = cli_migrate_nodes.get_flag("dry-run");

        let from_opt: Option<&String> = cli_migrate_nodes.get_one("from");
        let to: &String = cli_migrate_nodes
          .get_one("to")
          .expect("to value is mandatory");

        let xnames_string: &String =
          cli_migrate_nodes.get_one("XNAMES").unwrap();

        // Get target hsm group from either cli arguments or config and validate
        let from_rslt = get_groups_names_available(
          &backend,
          &shasta_token,
          from_opt,
          settings_hsm_group_name_opt,
        )
        .await;

        let from = match from_rslt {
          Ok(from) => from,
          Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
          }
        };

        // Validate 'to' hsm groups
        let to_rslt = get_groups_names_available(
          &backend,
          &shasta_token,
          /* shasta_base_url,
          shasta_root_cert, */
          Some(to),
          settings_hsm_group_name_opt,
        )
        .await;

        let to = match to_rslt {
          Ok(to) => to,
          Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
          }
        };

        // Migrate nodes
        migrate_nodes_between_hsm_groups::exec(
          &backend,
          &shasta_token,
          &to,
          &from,
          xnames_string,
          !dry_run,
          false,
          kafka_audit_opt,
        )
        .await;
      } else if let Some(_cli_migrate_vcluster) =
        cli_migrate.subcommand_matches("vCluster")
      {
        if let Some(cli_migrate_vcluster_backup) =
          cli_migrate.subcommand_matches("backup")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let bos: Option<&String> = cli_migrate_vcluster_backup.get_one("bos");
          let destination: Option<&String> =
            cli_migrate_vcluster_backup.get_one("destination");
          let prehook: Option<&String> =
            cli_migrate_vcluster_backup.get_one("pre-hook");
          let posthook: Option<&String> =
            cli_migrate_vcluster_backup.get_one("post-hook");

          migrate_backup::exec(
            &backend,
            &shasta_token,
            shasta_base_url,
            shasta_root_cert,
            bos.map(String::as_str),
            destination.map(String::as_str),
            prehook.map(String::as_str),
            posthook.map(String::as_str),
          )
          .await;
        } else if let Some(cli_migrate_vcluster_restore) =
          cli_migrate.subcommand_matches("restore")
        {
          let shasta_token = get_api_token(&backend, &site_name).await?;

          let bos_file: Option<&String> =
            cli_migrate_vcluster_restore.get_one("bos-file");
          let cfs_file: Option<&String> =
            cli_migrate_vcluster_restore.get_one("cfs-file");
          let hsm_file: Option<&String> =
            cli_migrate_vcluster_restore.get_one("hsm-file");
          let ims_file: Option<&String> =
            cli_migrate_vcluster_restore.get_one("ims-file");
          let image_dir: Option<&String> =
            cli_migrate_vcluster_restore.get_one("image-dir");
          let prehook: Option<&String> =
            cli_migrate_vcluster_restore.get_one("pre-hook");
          let posthook: Option<&String> =
            cli_migrate_vcluster_restore.get_one("post-hook");
          let overwrite: bool =
            cli_migrate_vcluster_restore.get_flag("overwrite");

          commands::migrate_restore::exec(
            &backend,
            &shasta_token,
            shasta_base_url,
            shasta_root_cert,
            bos_file.map(String::as_str),
            cfs_file.map(String::as_str),
            hsm_file.map(String::as_str),
            ims_file.map(String::as_str),
            image_dir.map(String::as_str),
            prehook.map(String::as_str),
            posthook.map(String::as_str),
            overwrite,
          )
          .await;
        }
      }
    } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
      if let Some(cli_delete_group) = cli_delete.subcommand_matches("group") {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let label: &String = cli_delete_group
          .get_one("VALUE")
          .expect("ERROR - group name argument is mandatory");

        let force: bool = *cli_delete_group
          .get_one("force")
          .expect("The 'force' argument must have a value");

        delete_group::exec(
          &backend,
          &shasta_token,
          label,
          force,
          kafka_audit_opt,
        )
        .await;
      } else if let Some(cli_delete_node) =
        cli_delete.subcommand_matches("node")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id: &String = cli_delete_node
          .get_one("VALUE")
          .expect("ERROR - group name argument is mandatory");

        backend.delete_node(&shasta_token, id).await?;

        println!("Node '{}' deleted", id);
      } else if let Some(cli_delete_hw_configuration) =
        cli_delete.subcommand_matches("hardware")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let dryrun = cli_delete_hw_configuration.get_flag("dry-run");

        let delete_hsm_group =
          cli_delete_hw_configuration.get_flag("delete-hsm-group");

        let target_hsm_group_name_arg_opt: Option<&String> =
          cli_delete_hw_configuration.get_one("target-cluster");
        let target_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          target_hsm_group_name_arg_opt,
          settings_hsm_group_name_opt,
        )
        .await?;

        // let parent_hsm_group_name_arg_opt =
        //     cli_remove_hw_configuration.get_one::<String>("PARENT_CLUSTER_NAME");

        let parent_hsm_group_name_arg_opt: Option<&String> =
          cli_delete_hw_configuration.get_one("parent-cluster");
        let parent_hsm_group_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          parent_hsm_group_name_arg_opt,
          settings_hsm_group_name_opt,
        )
        .await?;

        delete_hw_component_cluster::exec(
          &backend,
          &shasta_token,
          target_hsm_group_vec.first().unwrap(),
          parent_hsm_group_vec.first().unwrap(),
          cli_delete_hw_configuration
            .get_one::<String>("pattern")
            .unwrap(),
          dryrun,
          delete_hsm_group,
        )
        .await?;
      } else if let Some(cli_delete_boot_parameters) =
        cli_delete.subcommand_matches("boot-parameters")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let xnames: Option<&String> =
          cli_delete_boot_parameters.get_one("hosts");

        let hosts: Vec<String> = xnames
          .cloned()
          .unwrap_or_default()
          .split(',')
          .map(String::from)
          .collect();

        let boot_parameters = BootParameters {
          hosts,
          macs: None,
          nids: None,
          params: "".to_string(),
          kernel: "".to_string(),
          initrd: "".to_string(),
          cloud_init: None,
        };

        let result = backend
          .delete_bootparameters(&shasta_token, &boot_parameters)
          .await;

        match result {
          Ok(_) => println!("Boot parameters deleted successfully"),
          Err(error) => eprintln!("{}", error),
        }
      } else if let Some(cli_delete_redfish_endpoint) =
        cli_delete.subcommand_matches("redfish-endpoint")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let id: &String = cli_delete_redfish_endpoint.get_one("id").expect(
                    "ERROR - host argument is mandatory. Please provide the host to delete",
                );

        let result = backend.delete_redfish_endpoint(&shasta_token, &id).await;

        match result {
          Ok(_) => {
            println!("Redfish endpoint for id '{}' deleted successfully", id)
          }
          Err(error) => eprintln!("{}", error),
        }
      } else if let Some(cli_delete_kernel_parameters) =
        cli_delete.subcommand_matches("kernel-parameters")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_group_name_arg_opt =
          cli_delete_kernel_parameters.get_one("hsm-group");

        let node_expression: &String = if hsm_group_name_arg_opt.is_some() {
          let hsm_group_name_vec = get_groups_names_available(
            &backend,
            &shasta_token,
            hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
          )
          .await?;

          let hsm_members_rslt = backend
            .get_member_vec_from_group_name_vec(
              &shasta_token,
              &hsm_group_name_vec,
            )
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
          cli_delete_kernel_parameters
            .get_one::<String>("nodes")
            .expect("Neither HSM group nor nodes defined")
        };

        let kernel_parameters = cli_delete_kernel_parameters
          .get_one::<String>("VALUE")
          .unwrap(); // clap should validate the argument

        let assume_yes: bool =
          cli_delete_kernel_parameters.get_flag("assume-yes");
        let do_not_reboot: bool =
          cli_delete_kernel_parameters.get_flag("do-not-reboot");

        let dryrun = cli_delete_kernel_parameters.get_flag("dry-run");

        let result = delete_kernel_parameters::exec(
          backend,
          &shasta_token,
          kernel_parameters,
          node_expression,
          assume_yes,
          do_not_reboot,
          kafka_audit_opt,
          dryrun,
        )
        .await;

        match result {
          Ok(_) => {}
          Err(error) => eprintln!("{}", error),
        }
      } else if let Some(cli_delete_session) =
        cli_delete.subcommand_matches("session")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_group_available_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          None,
          settings_hsm_group_name_opt,
        )
        .await?;

        let session_name = cli_delete_session
          .get_one::<String>("SESSION_NAME")
          .expect("'session-name' argument must be provided");

        let assume_yes: bool = cli_delete_session.get_flag("assume-yes");

        let dry_run: bool = cli_delete_session.get_flag("dry-run");

        let result = delete_and_cancel_session::exec(
          backend,
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_group_available_vec,
          session_name,
          dry_run,
          assume_yes,
        )
        .await;

        if let Err(e) = result {
          eprintln!("{}", e.to_string());
          std::process::exit(1);
        }
      } else if let Some(cli_delete_configurations) =
        cli_delete.subcommand_matches("configurations")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let since_opt = if let Some(since) =
          cli_delete_configurations.get_one::<String>("since")
        {
          let date_time = chrono::NaiveDateTime::parse_from_str(
            &(since.to_string() + "T00:00:00"),
            "%Y-%m-%dT%H:%M:%S",
          )
          .unwrap();
          Some(date_time)
        } else {
          None
        };

        let until_opt = if let Some(until) =
          cli_delete_configurations.get_one::<String>("until")
        {
          let date_time = chrono::NaiveDateTime::parse_from_str(
            &(until.to_string() + "T00:00:00"),
            "%Y-%m-%dT%H:%M:%S",
          )
          .unwrap();
          Some(date_time)
        } else {
          None
        };

        /* let cfs_configuration_name_opt =
        cli_delete_configurations.get_one::<String>("configuration-name"); */

        let cfs_configuration_name_pattern: Option<&String> =
          cli_delete_configurations.get_one("configuration-name");
        let assume_yes = cli_delete_configurations.get_flag("assume-yes");

        // INPUT VALIDATION - Check since date is prior until date
        if since_opt.is_some()
          && until_opt.is_some()
          && since_opt.unwrap() > until_opt.unwrap()
        {
          eprintln!("ERROR - 'since' date can't be after 'until' date. Exit");
          std::process::exit(1);
        }

        let target_hsm_group_vec =
          if let Some(settings_hsm_group_name) = settings_hsm_group_name_opt {
            vec![settings_hsm_group_name.clone()]
          } else {
            get_groups_names_available(
              &backend,
              &shasta_token,
              None,
              settings_hsm_group_name_opt,
            )
            .await?
          };

        let result = delete_configurations_and_derivatives::exec(
          backend,
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &target_hsm_group_vec,
          cfs_configuration_name_pattern.map(String::as_str),
          since_opt,
          until_opt,
          assume_yes,
        )
        .await;

        if let Err(e) = result {
          eprintln!("{}", e.to_string());
          std::process::exit(1);
        }
      } else if let Some(cli_delete_images) =
        cli_delete.subcommand_matches("images")
      {
        let shasta_token = get_api_token(&backend, &site_name).await?;

        let hsm_name_available_vec = get_groups_names_available(
          &backend,
          &shasta_token,
          None,
          settings_hsm_group_name_opt,
        )
        .await?;

        let image_id_vec: Vec<&str> = cli_delete_images
          .get_one::<String>("IMAGE_LIST")
          .expect("'IMAGE_LIST' argument must be provided")
          .split(",")
          .map(|image_id_str| image_id_str.trim())
          .collect();

        let dry_run: bool = cli_delete_images.get_flag("dry-run");

        match delete_images::command::exec(
          &backend,
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          hsm_name_available_vec,
          image_id_vec.as_slice(),
          dry_run,
        )
        .await
        {
          Ok(_) => {
            println!("Images deleted successfully");
          }
          Err(e) => {
            eprintln!("{}", e.to_string());
            std::process::exit(1);
          }
        }
      }
    } else if let Some(cli_validate_local_repo) =
      cli_root.subcommand_matches("validate-local-repo")
    {
      let shasta_token = get_api_token(&backend, &site_name).await?;

      // FIXME: gitea auth token should be calculated before colling this function
      let gitea_token =
        crate::common::vault::http_client::fetch_shasta_vcs_token(
          &shasta_token,
          vault_base_url.expect("ERROR - vault base url is mandatory"),
          &site_name,
        )
        .await
        .unwrap();

      let repo_path = cli_validate_local_repo
        .get_one::<String>("repo-path")
        .unwrap();

      validate_local_repo::exec(
        shasta_root_cert,
        gitea_base_url,
        &gitea_token,
        repo_path,
      )
      .await;
    } else if let Some(cli_add_nodes) =
      cli_root.subcommand_matches("add-nodes-to-groups")
    {
      let shasta_token = get_api_token(&backend, &site_name).await?;

      let dryrun = cli_add_nodes.get_flag("dry-run");

      let hosts_expression = cli_add_nodes.get_one::<String>("nodes").unwrap();

      let target_hsm_name: &String = cli_add_nodes
        .get_one::<String>("group")
        .expect("Error - target cluster is mandatory");

      add_nodes_to_hsm_groups::exec(
        &backend,
        &shasta_token,
        target_hsm_name,
        hosts_expression,
        dryrun,
        kafka_audit_opt,
      )
      .await;
    } else if let Some(cli_remove_nodes) =
      cli_root.subcommand_matches("remove-nodes-from-groups")
    {
      let shasta_token = get_api_token(&backend, &site_name).await?;

      let dryrun = cli_remove_nodes.get_flag("dry-run");

      let nodes = cli_remove_nodes.get_one::<String>("nodes").unwrap();

      let target_hsm_name: &String = cli_remove_nodes
        .get_one::<String>("group")
        .expect("Error - target cluster is mandatory");

      remove_nodes_from_hsm_groups::exec(
        &backend,
        &shasta_token,
        target_hsm_name,
        nodes,
        dryrun,
        kafka_audit_opt,
      )
      .await;
    } else if let Some(_) = cli_root.subcommand_matches("download-boot-image") {
      println!("Download boot image");
    } else if let Some(_) = cli_root.subcommand_matches("upload-boot-image") {
      println!("Upload boot image");
    }
  }

  Ok(())
}
