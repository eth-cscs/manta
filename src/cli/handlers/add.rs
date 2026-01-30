use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use manta_backend_dispatcher::interfaces::hsm::{
    component::ComponentTrait, group::GroupTrait, redfish_endpoint::RedfishEndpointTrait,
};
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use crate::common::{authorization::get_groups_names_available, authentication::get_api_token, kafka::Kafka};
use std::{fs::File, io::BufReader, path::PathBuf};
use manta_backend_dispatcher::types::{HWInventoryByLocationList, hsm::inventory::{RedfishEndpoint, RedfishEndpointArray}, bss::BootParameters};
use serde_json::Value;
use crate::cli::commands::{add_node, add_group, add_hw_component_cluster, add_kernel_parameters};

pub async fn handle_add(
    cli_add: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    settings_hsm_group_name_opt: Option<&String>,
    kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
    if let Some(cli_add_node) = cli_add.subcommand_matches("node") {
        let shasta_token = get_api_token(backend, site_name).await?;
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
                    serde_json::from_value::<HWInventoryByLocationList>(hw_inventory_value)
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
            backend,
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
        let new_group_members_rslt = backend
            .add_members_to_group(&shasta_token, group, &[id])
            .await;
        if let Err(error) = &new_group_members_rslt {
            eprintln!("ERROR - Could not add node to group. Reason:\n{:#?}", error);
        }
        println!("Node '{}' created and added to group '{}'", id, group,);
    } else if let Some(cli_add_group) = cli_add.subcommand_matches("group") {
        let shasta_token = get_api_token(backend, site_name).await?;
        let label = cli_add_group
            .get_one::<String>("label")
            .expect("ERROR - 'label' argument is mandatory");
        let description: Option<&String> = cli_add_group.get_one("description");
        let node_expression: Option<&String> = cli_add_group.get_one::<String>("nodes");
        add_group::exec(
            backend.clone(),
            &shasta_token,
            label,
            description,
            node_expression,
            true,
            false,
            kafka_audit_opt,
        )
        .await?;
    } else if let Some(cli_add_hw_configuration) = cli_add.subcommand_matches("hardware") {
        let shasta_token = get_api_token(backend, site_name).await?;
        let target_hsm_group_name_arg_opt: Option<&String> =
            cli_add_hw_configuration.get_one("target-cluster");
        let target_hsm_group_vec = get_groups_names_available(
            backend,
            &shasta_token,
            target_hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
        )
        .await?;
        let parent_hsm_group_name_arg_opt: Option<&String> =
            cli_add_hw_configuration.get_one("parent-cluster");
        let parent_hsm_group_vec = get_groups_names_available(
            backend,
            &shasta_token,
            parent_hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
        )
        .await?;
        let dryrun = cli_add_hw_configuration.get_flag("dry-run");
        let create_hsm_group = *cli_add_hw_configuration
            .get_one::<bool>("create-hsm-group")
            .unwrap_or(&false);
        add_hw_component_cluster::exec(
            backend,
            &shasta_token,
            target_hsm_group_vec.first().unwrap(),
            parent_hsm_group_vec.first().unwrap(),
            cli_add_hw_configuration.get_one::<String>("pattern").unwrap(),
            dryrun,
            create_hsm_group,
        )
        .await?;
    } else if let Some(cli_add_boot_parameters) = cli_add.subcommand_matches("boot-parameters") {
        let shasta_token = get_api_token(backend, site_name).await?;
        let hosts = cli_add_boot_parameters
            .get_one::<String>("hosts")
            .expect("ERROR - 'hosts' argument is mandatory");
        let macs: Option<String> = cli_add_boot_parameters.get_one("macs").cloned();
        let nids: Option<String> = cli_add_boot_parameters.get_one("nids").cloned();
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
    } else if let Some(cli_add_kernel_parameters) = cli_add.subcommand_matches("kernel-parameters") {
        let shasta_token = get_api_token(backend, site_name).await?;
        let hsm_group_name_arg_opt = cli_add_kernel_parameters.get_one("hsm-group");
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
            cli_add_kernel_parameters
                .get_one::<String>("nodes")
                .expect("Neither HSM group nor nodes defined")
        };
        let kernel_parameters = cli_add_kernel_parameters
            .get_one::<String>("VALUE")
            .unwrap();
        let overwrite: bool = cli_add_kernel_parameters.get_flag("overwrite");
        let assume_yes: bool = cli_add_kernel_parameters.get_flag("assume-yes");
        let do_not_reboot: bool = cli_add_kernel_parameters.get_flag("do-not-reboot");
        let dryrun = cli_add_kernel_parameters.get_flag("dry-run");
        let result = add_kernel_parameters::exec(
            backend.clone(),
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
    } else if let Some(cli_add_redfish_endpoint) = cli_add.subcommand_matches("redfish-endpoint") {
        let shasta_token = get_api_token(backend, site_name).await?;
        let id = cli_add_redfish_endpoint
            .get_one::<String>("id")
            .expect("ERROR - 'id' argument is mandatory")
            .to_string();
        let name: Option<String> = cli_add_redfish_endpoint.get_one("name").cloned();
        let hostname: Option<String> = cli_add_redfish_endpoint.get_one::<String>("hostname").cloned();
        let domain: Option<String> = cli_add_redfish_endpoint.get_one::<String>("domain").cloned();
        let fqdn: Option<String> = cli_add_redfish_endpoint.get_one::<String>("fqdn").cloned();
        let enabled: bool = cli_add_redfish_endpoint.get_flag("enabled");
        let user: Option<String> = cli_add_redfish_endpoint.get_one::<String>("user").cloned();
        let password: Option<String> = cli_add_redfish_endpoint.get_one::<String>("password").cloned();
        let use_ssdp: bool = cli_add_redfish_endpoint.get_flag("use-ssdp");
        let mac_required: bool = cli_add_redfish_endpoint.get_flag("mac-required");
        let mac_addr: Option<String> = cli_add_redfish_endpoint.get_one::<String>("macaddr").cloned();
        let ip_address: Option<String> = cli_add_redfish_endpoint.get_one::<String>("ipaddress").cloned();
        let rediscover_on_update: bool = cli_add_redfish_endpoint.get_flag("rediscover-on-update");
        let template_id: Option<String> = cli_add_redfish_endpoint.get_one::<String>("template-id").cloned();
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
    Ok(())
}
