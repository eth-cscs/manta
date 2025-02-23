use backend_dispatcher::{
    contracts::BackendTrait,
    interfaces::hsm::{
        component::ComponentTrait, group::GroupTrait, hardware_inventory::HardwareInventory,
    },
    types::{ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList},
};
use clap_complete::{generate, generate_to};
use std::{
    env,
    fs::File,
    io::{self, BufReader, IsTerminal},
    path::PathBuf,
};

use clap::Command;
use config::Config;
use k8s_openapi::chrono;

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    cli::commands::validate_local_repo,
    common::{
        authorization::{get_groups_available, validate_target_hsm_members},
        config::types::MantaConfiguration,
        kafka::Kafka,
    },
};

use super::commands::{
    self, add_group, add_hw_component_cluster, add_kernel_parameters, add_nodes_to_hsm_groups,
    apply_boot_cluster, apply_boot_node, apply_ephemeral_env, apply_hw_cluster_pin,
    apply_hw_cluster_unpin, apply_sat_file, apply_session, apply_template, config_set_hsm,
    config_set_log, config_set_parent_hsm, config_set_site, config_show, config_unset_auth,
    config_unset_hsm, config_unset_parent_hsm, console_cfs_session_image_target_ansible,
    console_node, delete_data_related_to_cfs_configuration::delete_data_related_cfs_configuration,
    delete_group, delete_hw_component_cluster, delete_image, delete_kernel_parameters,
    delete_sessions, get_cluster, get_configuration, get_hw_configuration_node, get_images,
    get_kernel_parameters, get_nodes, get_session, get_template, migrate_backup,
    migrate_nodes_between_hsm_groups, power_off_cluster, power_off_nodes, power_on_cluster,
    power_on_nodes, power_reset_cluster, power_reset_nodes, remove_nodes_from_hsm_groups,
};

pub async fn process_cli(
    mut cli: Command,
    backend: StaticBackendDispatcher,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: Option<&String>,
    vault_secret_path: Option<&String>,
    vault_role_id: Option<&String>,
    gitea_base_url: &str,
    settings_hsm_group_name_opt: Option<&String>,
    k8s_api_url: Option<&String>,
    kafka_audit_opt: Option<&Kafka>,
    settings: &Config,
    configuration: &MantaConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let site_name: String = match settings.get("site") {
        Ok(site_name) => site_name,
        Err(_) => {
            eprintln!(
                "'site' value in configuration file is missing or does not have a value. Exit"
            );
            std::process::exit(1);
        }
    };

    let cli_root = cli.clone().get_matches();

    if let Some(cli_config) = cli_root.subcommand_matches("config") {
        if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
            /* let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?; */
            let shasta_token_rslt = backend.get_api_token(&site_name).await;

            config_show::exec(
                &backend,
                shasta_token_rslt.ok(),
                /* shasta_base_url,
                shasta_root_cert, */
                settings,
            )
            .await;
        } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
            if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                config_set_hsm::exec(
                    &backend,
                    &shasta_token,
                    cli_config_set_hsm.get_one::<String>("HSM_GROUP_NAME"),
                    // hsm_available_vec,
                )
                .await;
            }
            if let Some(cli_config_set_parent_hsm) = cli_config_set.subcommand_matches("parent-hsm")
            {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                config_set_parent_hsm::exec(
                    &backend,
                    &shasta_token,
                    cli_config_set_parent_hsm.get_one::<String>("HSM_GROUP_NAME"),
                    // hsm_available_vec,
                )
                .await;
            }
            if let Some(cli_config_set_site) = cli_config_set.subcommand_matches("site") {
                config_set_site::exec(cli_config_set_site.get_one::<String>("SITE_NAME")).await;
            }
            if let Some(cli_config_set_log) = cli_config_set.subcommand_matches("log") {
                config_set_log::exec(cli_config_set_log.get_one::<String>("LOG_LEVEL")).await;
            }
        } else if let Some(cli_config_unset) = cli_config.subcommand_matches("unset") {
            if let Some(_cli_config_unset_hsm) = cli_config_unset.subcommand_matches("hsm") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */

                config_unset_hsm::exec().await;
            }
            if let Some(_cli_config_unset_parent_hsm) =
                cli_config_unset.subcommand_matches("parent-hsm")
            {
                let shasta_token = backend
                    .get_api_token(
                        /* shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url, */
                        &site_name,
                    )
                    .await?;
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */

                config_unset_parent_hsm::exec(&backend, &shasta_token).await;
            }
            if let Some(_cli_config_unset_auth) = cli_config_unset.subcommand_matches("auth") {
                config_unset_auth::exec().await;
            }
        } else if let Some(cli_config_generate_autocomplete) =
            cli_config.subcommand_matches("gen-autocomplete")
        {
            let shell_opt: Option<String> =
                cli_config_generate_autocomplete.get_one("shell").cloned();

            let path_opt: Option<PathBuf> =
                cli_config_generate_autocomplete.get_one("path").cloned();

            let shell = if let Some(shell) = shell_opt {
                shell.to_ascii_uppercase()
            } else {
                let shell_ostring =
                    PathBuf::from(env::var_os("SHELL").expect("$SHELL env missing"))
                        .file_name()
                        .unwrap()
                        .to_ascii_uppercase();

                shell_ostring
                    .into_string()
                    .expect("Could not convert shell name to string")
            };

            let shell_gen = match shell.as_str() {
                "BASH" => clap_complete::Shell::Bash,
                "ZSH" => clap_complete::Shell::Zsh,
                "FISH" => clap_complete::Shell::Fish,
                _ => {
                    eprintln!("ERROR - Shell '{shell}' not supported",);
                    std::process::exit(1);
                }
            };

            if let Some(path) = path_opt {
                // Destination path defined
                log::info!(
                    "Generating shell autocomplete for '{}' to '{}'",
                    shell,
                    path.display()
                );
                generate_to(shell_gen, &mut cli, env!("CARGO_PKG_NAME"), path)?;
            } else {
                // Destination path not defined - print to stdout
                log::info!("Generating shell autocomplete for '{}'", shell);
                generate(
                    shell_gen,
                    &mut cli,
                    env!("CARGO_PKG_NAME"),
                    &mut io::stdout(),
                );
            }
        }
    } else {
        if let Some(cli_power) = cli_root.subcommand_matches("power") {
            if let Some(cli_power_on) = cli_power.subcommand_matches("on") {
                if let Some(cli_power_on_cluster) = cli_power_on.subcommand_matches("cluster") {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let hsm_group_name_arg = cli_power_on_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("The 'cluster name' argument must have a value");

                    let target_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        Some(hsm_group_name_arg),
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    let target_hsm_group = target_hsm_group_vec
                        .first()
                        .expect("The 'cluster name' argument must have a value");

                    let assume_yes: bool = cli_power_on_cluster.get_flag("assume-yes");

                    let output: &str = cli_power_on_cluster.get_one::<String>("output").unwrap();

                    power_on_cluster::exec(
                        backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        target_hsm_group,
                        assume_yes,
                        output,
                        kafka_audit_opt,
                    )
                    .await;
                } else if let Some(cli_power_on_node) = cli_power_on.subcommand_matches("nodes") {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let xname_requested: &str = cli_power_on_node
                        .get_one::<String>("VALUE")
                        .expect("The 'xnames' argument must have values");

                    let assume_yes: bool = cli_power_on_node.get_flag("assume-yes");

                    let output: &str = cli_power_on_node.get_one::<String>("output").unwrap();

                    /* let _ = validate_target_hsm_members(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_vec.clone(),
                    )
                    .await; */

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
                if let Some(cli_power_off_cluster) = cli_power_off.subcommand_matches("cluster") {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let hsm_group_name_arg = cli_power_off_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("The 'cluster name' argument must have a value");

                    let force = cli_power_off_cluster
                        .get_one::<bool>("force")
                        .expect("The 'force' argument must have a value");

                    let output: &str = cli_power_off_cluster.get_one::<String>("output").unwrap();

                    let target_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
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
                        /* shasta_base_url,
                        shasta_root_cert, */
                        target_hsm_group,
                        *force,
                        assume_yes,
                        output,
                        kafka_audit_opt,
                    )
                    .await;
                } else if let Some(cli_power_off_node) = cli_power_off.subcommand_matches("nodes") {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let xname_requested: &str = cli_power_off_node
                        .get_one::<String>("VALUE")
                        .expect("The 'xnames' argument must have values");

                    let force = cli_power_off_node
                        .get_one::<bool>("force")
                        .expect("The 'force' argument must have a value");

                    let is_regex = *cli_power_off_node.get_one::<bool>("regex").unwrap_or(&true);

                    let assume_yes: bool = cli_power_off_node.get_flag("assume-yes");

                    let output: &str = cli_power_off_node.get_one::<String>("output").unwrap();

                    power_off_nodes::exec(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        xname_requested,
                        is_regex,
                        *force,
                        assume_yes,
                        output,
                        kafka_audit_opt,
                    )
                    .await;
                }
            } else if let Some(cli_power_reset) = cli_power.subcommand_matches("reset") {
                if let Some(cli_power_reset_cluster) = cli_power_reset.subcommand_matches("cluster")
                {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let hsm_group_name_arg = cli_power_reset_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("The 'cluster name' argument must have a value");

                    let force = cli_power_reset_cluster
                        .get_one::<bool>("force")
                        .expect("The 'force' argument must have a value");

                    let output: &str = cli_power_reset_cluster.get_one::<String>("output").unwrap();

                    let target_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
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
                        /* shasta_base_url,
                        shasta_root_cert, */
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
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let xname_requested: &str = cli_power_reset_node
                        .get_one::<String>("VALUE")
                        .expect("The 'xnames' argument must have values");

                    let force = cli_power_reset_node
                        .get_one::<bool>("force")
                        .expect("The 'force' argument must have a value");

                    let is_regex = *cli_power_reset_node
                        .get_one::<bool>("regex")
                        .unwrap_or(&true);

                    let assume_yes: bool = cli_power_reset_node.get_flag("assume-yes");

                    let output: &str = cli_power_reset_node.get_one::<String>("output").unwrap();

                    /* let _ = validate_target_hsm_members(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_requested,
                    )
                    .await; */

                    power_reset_nodes::exec(
                        &backend,
                        &shasta_token,
                        xname_requested,
                        is_regex,
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
                let shasta_token = backend.get_api_token(&site_name).await?;

                let id = cli_add_node
                    .get_one::<String>("id")
                    .expect("ERROR - 'id' argument is mandatory");

                let group = cli_add_node
                    .get_one::<String>("group")
                    .expect("ERROR - 'group' argument is mandatory");

                let hardware_file = cli_add_node
                    .get_one::<PathBuf>("hardware")
                    .expect("ERROR - 'hardware' argument is mandatory");

                let file = File::open(hardware_file)?;
                let reader = BufReader::new(file);

                let hw_inventory_value: serde_json::Value =
                    serde_json::from_reader(reader).unwrap();

                // let hw_inventory: HWInventoryList = serde_json::from_reader(reader).unwrap();
                let hw_inventory: HWInventoryByLocationList =
                    serde_json::from_value(hw_inventory_value)
                        .expect("ERROR - Could not parse hardware inventory file");

                println!(
                    "DEBUG - hw inventory:\n{}",
                    serde_json::to_string_pretty(&hw_inventory).unwrap()
                );

                let arch_opt = cli_add_node.get_one::<String>("arch").cloned();

                let enabled = cli_add_node
                    .get_one::<bool>("disabled")
                    .cloned()
                    .unwrap_or(true);

                // Create node api payload
                let component: ComponentCreate = ComponentCreate {
                    id: id.to_string(),
                    state: "Unknown".to_string(),
                    flag: None,
                    enabled: Some(enabled),
                    software_status: None,
                    role: None,
                    sub_role: None,
                    nid: None,
                    subtype: None,
                    net_type: None,
                    arch: arch_opt,
                    class: None,
                };

                let components = ComponentArrayPostArray {
                    components: vec![component],
                    force: Some(true),
                };

                // Add node to backend
                let add_node_rslt = backend.post_nodes(shasta_token.as_str(), components).await;

                if let Err(e) = add_node_rslt {
                    eprintln!("ERROR - Could not create node '{}'. Reason:\n{:#?}", id, e);
                    std::process::exit(1);
                };

                log::info!("Node saved '{}'. Try to add hardware", id);

                // Add hardware inventory
                let add_hardware_rslt = backend
                    .post_inventory_hardware(&shasta_token, hw_inventory)
                    .await;

                if let Err(e) = add_hardware_rslt {
                    eprintln!("ERROR - Could not save node's hardware. Reason:\n{:#?}", e);
                    std::process::exit(1);
                };

                log::info!(
                    "Node's hardware saved '{}'. Try to join node '{}' to group '{}'",
                    id,
                    id,
                    group
                );

                // Add node to group
                let new_group_members_rslt = backend
                    .add_members_to_group(&shasta_token, group, vec![id.clone().as_str()])
                    .await;

                if new_group_members_rslt.is_err() {
                    eprintln!(
                        "ERROR - Could not add node to group. Reason:\n{:#?}",
                        new_group_members_rslt
                    );
                    eprintln!("Try to delete node '{}'", id);
                    // Could not add xname to group. Reset operation by removing the node
                    eprintln!("ERROR - operation to add node '{id}' to group '{group}' failed, delete node '{id}' to reset state");
                    let delete_node_rslt = backend
                        .delete_node(shasta_token.as_str(), id.clone().as_str())
                        .await;

                    if delete_node_rslt.is_ok() {
                        eprintln!("Node '{}' deleted", id);
                    }

                    std::process::exit(1);
                }

                println!(
                    "Node '{}' created and added to group '{}'.\nGroup '{}' members:\n{:#?}",
                    id,
                    group,
                    group,
                    new_group_members_rslt.unwrap()
                );
            } else if let Some(cli_add_group) = cli_add.subcommand_matches("group") {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let label = cli_add_group
                    .get_one::<String>("label")
                    .expect("ERROR - 'label' argument is mandatory");

                let description = cli_add_group.get_one::<String>("description");

                let node_expression: Option<&String> = cli_add_group.get_one::<String>("nodes");

                let assume_yes: bool = cli_add_group.get_flag("assume-yes");

                let dryrun = cli_add_group.get_flag("dryrun");

                /* // Validate user has access to the list of xnames requested
                if let Some(xname_vec) = &xname_vec_opt {
                    validate_target_hsm_members(
                        &shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_vec.iter().map(|xname| xname.to_string()).collect(),
                    )
                    .await;
                }

                // Create Group instance for http payload
                let group = HsmGroup::new(label, xname_vec_opt);

                // Call backend to create group
                let result = backend.add_hsm_group(&shasta_token, group).await; */

                add_group::exec(
                    backend,
                    &shasta_token,
                    label,
                    description,
                    node_expression,
                    assume_yes,
                    dryrun,
                    kafka_audit_opt,
                )
                .await;
            } else if let Some(cli_add_hw_configuration) = cli_add.subcommand_matches("hardware") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let target_hsm_group_name_arg_opt =
                    cli_add_hw_configuration.get_one::<String>("target-cluster");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    target_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await?;

                let parent_hsm_group_name_arg_opt =
                    cli_add_hw_configuration.get_one::<String>("parent-cluster");

                let parent_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    parent_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await?;
                let _ = cli_add_hw_configuration.get_one::<String>("target-cluster");

                let nodryrun = *cli_add_hw_configuration
                    .get_one::<bool>("no-dryrun")
                    .unwrap_or(&true);

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
                    nodryrun,
                    create_hsm_group,
                )
                .await;
            } else if let Some(cli_add_kernel_parameters) =
                cli_add.subcommand_matches("kernel-parameters")
            {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt = cli_add_kernel_parameters.get_one("hsm-group");

                let xname_vec: Vec<String> = if hsm_group_name_arg_opt.is_some() {
                    let hsm_group_name_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    let hsm_members_rslt = backend
                        .get_member_vec_from_group_name_vec(&shasta_token, hsm_group_name_vec)
                        .await;

                    match hsm_members_rslt {
                        Ok(hsm_members) => hsm_members,
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
                        .get_one::<String>("xnames")
                        .expect("Neither HSM group nor xnames defined")
                        .split(",")
                        .map(|value| value.to_string())
                        .collect()
                };

                let kernel_parameters = cli_add_kernel_parameters
                    .get_one::<String>("VALUE")
                    .unwrap(); // clap should validate the argument

                let assume_yes: bool = cli_add_kernel_parameters.get_flag("assume-yes");

                // Validate user has access to the list of xnames requested
                validate_target_hsm_members(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    &xname_vec,
                )
                .await;

                let result = add_kernel_parameters::exec(
                    backend,
                    &shasta_token,
                    kernel_parameters,
                    xname_vec,
                    assume_yes,
                    kafka_audit_opt,
                )
                .await;

                match result {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                }
            }
        } else if let Some(cli_get) = cli_root.subcommand_matches("get") {
            if let Some(cli_get_group) = cli_get.subcommand_matches("groups") {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let group_name_arg_opt = cli_get_group.get_one::<String>("VALUE");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await?;

                let output = cli_get_group
                    .get_one::<String>("output")
                    .expect("ERROR - 'output' argument is mandatory");

                let hsm_group_vec: Vec<&str> = target_hsm_group_vec
                    .iter()
                    .map(|x| &**x)
                    .collect::<Vec<&str>>();

                commands::get_group::exec(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    Some(hsm_group_vec.as_slice()),
                    output,
                )
                .await?;
            } else if let Some(cli_get_hw_configuration) = cli_get.subcommand_matches("hardware") {
                if let Some(cli_get_hw_configuration_cluster) =
                    cli_get_hw_configuration.subcommand_matches("cluster")
                {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let hsm_group_name_arg_opt =
                        cli_get_hw_configuration_cluster.get_one::<String>("CLUSTER_NAME");

                    let target_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    commands::get_hw_configuration_cluster::exec(
                        backend,
                        &shasta_token,
                        target_hsm_group_vec.first().unwrap(),
                        cli_get_hw_configuration_cluster.get_one::<String>("output"),
                    )
                    .await;
                } else if let Some(cli_get_hw_configuration_node) =
                    cli_get_hw_configuration.subcommand_matches("node")
                {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let xnames = cli_get_hw_configuration_node
                        .get_one::<String>("XNAMES")
                        .expect("HSM group name is needed at this point");

                    let xname_vec: Vec<String> =
                        xnames.split(',').map(|xname| xname.to_string()).collect();

                    validate_target_hsm_members(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        &xname_vec,
                    )
                    .await;

                    get_hw_configuration_node::exec(
                        &backend,
                        &shasta_token,
                        xnames,
                        cli_get_hw_configuration_node.get_one::<String>("type"),
                        cli_get_hw_configuration_node.get_one::<String>("output"),
                    )
                    .await;
                }
            } else if let Some(cli_get_configuration) = cli_get.subcommand_matches("configurations")
            {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                // FIXME: gitea auth token should be calculated before calling this function
                let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
                    vault_base_url.expect("ERROR - vault base url is mandatory"),
                    vault_secret_path.expect("ERROR - vault secret path is mandatory"),
                    vault_role_id.expect("ERROR - vault role id is mandatory"),
                )
                .await
                .unwrap();

                let hsm_group_name_arg_rslt = cli_get_configuration.try_get_one("hsm-group");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
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

                get_configuration::exec(
                    &backend,
                    gitea_base_url,
                    &gitea_token,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_get_configuration.get_one::<String>("name"),
                    cli_get_configuration.get_one::<String>("pattern"),
                    &target_hsm_group_vec,
                    limit,
                    cli_get_configuration.get_one("output"),
                )
                .await;
            } else if let Some(cli_get_session) = cli_get.subcommand_matches("sessions") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt = cli_get_session.try_get_one("hsm-group");

                let target_hsm_group_vec: Vec<String> = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await?;

                let limit: Option<&u8> = if let Some(true) = cli_get_session.get_one("most-recent")
                {
                    Some(&1)
                } else {
                    cli_get_session.get_one::<u8>("limit")
                };

                let xname_vec_opt: Option<Vec<&str>> = cli_get_session
                    .get_one::<String>("xnames")
                    .map(|xname_str| xname_str.split(',').map(|xname| xname.trim()).collect());

                get_session::exec(
                    &backend,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(target_hsm_group_vec),
                    xname_vec_opt,
                    cli_get_session.get_one::<String>("min-age"),
                    cli_get_session.get_one::<String>("max-age"),
                    cli_get_session.get_one::<String>("status"),
                    cli_get_session.get_one::<String>("name"),
                    limit,
                    cli_get_session.get_one("output"),
                )
                .await;
            } else if let Some(cli_get_template) = cli_get.subcommand_matches("templates") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt = cli_get_template.try_get_one("hsm-group");

                let output: &String = cli_get_template
                    .get_one("output")
                    .expect("ERROR - output must be a valid value");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await?;

                let hsm_member_vec = backend
                    .get_member_vec_from_group_name_vec(
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        target_hsm_group_vec.clone(),
                    )
                    .await?;
                /* let hsm_member_vec = hsm::group::utils::get_member_vec_from_hsm_name_vec(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec.clone(),
                )
                .await; */

                let limit_number_opt = if let Some(limit) = cli_get_template.get_one("limit") {
                    Some(limit)
                } else if let Some(true) = cli_get_template.get_one("most-recent") {
                    Some(&1)
                } else {
                    None
                };

                get_template::exec(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &target_hsm_group_vec,
                    &hsm_member_vec,
                    cli_get_template.get_one::<String>("name"),
                    limit_number_opt,
                    output,
                )
                .await;
            } else if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt = cli_get_cluster.get_one::<String>("HSM_GROUP_NAME");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await?;

                get_cluster::exec(
                    &backend,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &target_hsm_group_vec,
                    *cli_get_cluster
                        .get_one::<bool>("nids-only-one-line")
                        .unwrap_or(&false),
                    *cli_get_cluster
                        .get_one::<bool>("xnames-only-one-line")
                        .unwrap_or(&false),
                    cli_get_cluster.get_one::<String>("output"),
                    *cli_get_cluster.get_one::<bool>("status").unwrap_or(&false),
                )
                .await;
            } else if let Some(cli_get_nodes) = cli_get.subcommand_matches("nodes") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */

                // Get list of nodes from cli argument
                let shasta_token = backend.get_api_token(&site_name).await?;

                let xname_requested: &str = cli_get_nodes
                    .get_one::<String>("VALUE")
                    .expect("The 'xnames' argument must have values");

                let is_include_siblings = cli_get_nodes.get_flag("include-siblings");

                let is_regex = *cli_get_nodes.get_one::<bool>("regex").unwrap_or(&true);

                let nids_only = cli_get_nodes
                    .get_one::<bool>("nids-only-one-line")
                    .unwrap_or(&false);

                let output = cli_get_nodes.get_one::<String>("output");

                let status = *cli_get_nodes.get_one::<bool>("status").unwrap_or(&false);

                /* // Get list of nodes from cli argument
                let node_vec: Vec<String> = cli_get_nodes
                    .get_one::<String>("XNAMES")
                    .expect("ERROR - need list of xnames")
                    .clone()
                    .split(",")
                    .map(|xname_str| xname_str.trim().to_string())
                    .collect();

                // Validate user has access to list of xnames
                validate_target_hsm_members(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    node_vec.clone(),
                )
                .await; */

                get_nodes::exec(
                    &backend,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_requested,
                    is_include_siblings,
                    *nids_only,
                    false,
                    output,
                    status,
                    is_regex,
                )
                .await;
            /* } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
            log::warn!("Deprecated - Do not use this command.");
            /* let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?; */
            let shasta_token = backend.get_api_token(&site_name).await?;

            let hsm_group_name_arg_opt = cli_get_hsm_groups.get_one::<String>("HSM_GROUP_NAME");

            let target_hsm_group_vec = get_groups_available(
                &backend,
                &shasta_token,
                /* shasta_base_url,
                shasta_root_cert, */
                hsm_group_name_arg_opt,
                settings_hsm_group_name_opt,
            )
            .await?;

            get_hsm::exec(
                &backend,
                &shasta_token,
                shasta_base_url,
                shasta_root_cert,
                target_hsm_group_vec.first().unwrap(),
            )
            .await; */
            } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt = cli_get_images.try_get_one("hsm-group");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await?;

                get_images::exec(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &target_hsm_group_vec,
                    cli_get_images.get_one::<String>("id"),
                    cli_get_images.get_one::<u8>("limit"),
                )
                .await;
            } else if let Some(cli_get_kernel_parameters) =
                cli_get.subcommand_matches("kernel-parameters")
            {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt =
                    cli_get_kernel_parameters.get_one::<String>("hsm-group");

                let output: &String = cli_get_kernel_parameters
                    .get_one("output")
                    .expect("ERROR - output value missing");

                let filter_opt: Option<&String> = cli_get_kernel_parameters.get_one("filter");

                let xnames: Vec<String> = if hsm_group_name_arg_opt.is_some() {
                    let hsm_group_name_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    /* hsm::group::utils::get_member_vec_from_hsm_name_vec(
                        &shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_vec,
                    )
                    .await */

                    let hsm_members_rslt = backend
                        .get_member_vec_from_group_name_vec(&shasta_token, hsm_group_name_vec)
                        .await;

                    match hsm_members_rslt {
                        Ok(hsm_members) => hsm_members,
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
                        .get_one::<String>("xnames")
                        .expect("Neither HSM group nor xnames defined")
                        .split(",")
                        .map(|value| value.to_string())
                        .collect()
                };

                let _ = get_kernel_parameters::exec(
                    &backend,
                    &shasta_token,
                    xnames,
                    filter_opt,
                    output,
                )
                .await;
            }
        } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
            if let Some(cli_apply_hw) = cli_apply.subcommand_matches("hardware") {
                if let Some(cli_apply_hw_cluster) = cli_apply_hw.subcommand_matches("cluster") {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let target_hsm_group_name_arg_opt =
                        cli_apply_hw_cluster.get_one::<String>("target-cluster");

                    let target_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        target_hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    let parent_hsm_group_name_arg_opt =
                        cli_apply_hw_cluster.get_one::<String>("parent-cluster");

                    let parent_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        parent_hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    let nodryrun = *cli_apply_hw_cluster
                        .get_one::<bool>("no-dryrun")
                        .unwrap_or(&true);

                    let create_target_hsm_group = *cli_apply_hw_cluster
                        .get_one::<bool>("create-target-hsm-group")
                        .unwrap_or(&true);

                    let delete_empty_parent_hsm_group = *cli_apply_hw_cluster
                        .get_one::<bool>("delete-empty-parent-hsm-group")
                        .unwrap_or(&true);

                    let is_unpin = cli_apply_hw_cluster
                        .get_one::<bool>("unpin-nodes")
                        .unwrap_or(&false);

                    if *is_unpin {
                        apply_hw_cluster_unpin::command::exec(
                            &backend,
                            &shasta_token,
                            target_hsm_group_vec.first().unwrap(),
                            parent_hsm_group_vec.first().unwrap(),
                            cli_apply_hw_cluster.get_one::<String>("pattern").unwrap(),
                            nodryrun,
                            create_target_hsm_group,
                            delete_empty_parent_hsm_group,
                        )
                        .await;
                    } else {
                        apply_hw_cluster_pin::command::exec(
                            &backend,
                            &shasta_token,
                            /* shasta_base_url,
                            shasta_root_cert, */
                            target_hsm_group_vec.first().unwrap(),
                            parent_hsm_group_vec.first().unwrap(),
                            cli_apply_hw_cluster.get_one::<String>("pattern").unwrap(),
                            nodryrun,
                            create_target_hsm_group,
                            delete_empty_parent_hsm_group,
                        )
                        .await;
                    }
                }
            } else if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                // FIXME: gitea auth token should be calculated before colling this function
                let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
                    vault_base_url.expect("ERROR - vault base url is mandatory"),
                    vault_secret_path.expect("ERROR - vault secret path is mandatory"),
                    vault_role_id.expect("ERROR - vault role id is mandatory"),
                )
                .await
                .unwrap();

                let hsm_group_name_arg_opt: Option<&String> =
                    cli_apply_session.try_get_one("hsm-group").unwrap_or(None);

                let cfs_conf_sess_name_opt: Option<&String> = cli_apply_session.get_one("name");
                let playbook_file_name_opt: Option<&String> =
                    cli_apply_session.get_one("playbook-name");

                let hsm_group_members_opt = cli_apply_session.get_one::<String>("ansible-limit");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await?;

                target_hsm_group_vec.first().unwrap();

                if let Some(ansible_limit) = hsm_group_members_opt {
                    validate_target_hsm_members(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        &ansible_limit
                            .split(',')
                            .map(|xname| xname.trim().to_string())
                            .collect::<Vec<String>>(),
                    )
                    .await;
                }

                let site = configuration
                    .sites
                    .get(&configuration.site.clone())
                    .unwrap();

                let apply_session_rslt = apply_session::exec(
                    backend,
                    &gitea_token,
                    gitea_base_url,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    k8s_api_url.expect("ERROR - k8s api url is mandatory"),
                    cfs_conf_sess_name_opt,
                    playbook_file_name_opt,
                    hsm_group_name_arg_opt,
                    cli_apply_session
                        .get_many("repo-path")
                        .unwrap()
                        .cloned()
                        .collect(),
                    hsm_group_members_opt.cloned(),
                    cli_apply_session
                        .get_one::<String>("ansible-verbosity")
                        .cloned(),
                    cli_apply_session
                        .get_one::<String>("ansible-passthrough")
                        .cloned(),
                    *cli_apply_session
                        .get_one::<bool>("watch-logs")
                        .unwrap_or(&false),
                    kafka_audit_opt,
                    &site
                        .k8s
                        .as_ref()
                        .expect("ERROR - k8s section not found in configuration"), // FIXME:
                                                                                   // refactor this, we can't check configuration here and should be done ealier
                )
                .await;

                if let Err(e) = apply_session_rslt {
                    eprintln!("ERROR - Could not apply session. Reason:\n{:#?}", e);
                    std::process::exit(1);
                }
            } else if let Some(cli_apply_sat_file) = cli_apply.subcommand_matches("sat-file") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                // FIXME: gitea auth token should be calculated before colling this function
                let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
                    vault_base_url.expect("ERROR - vault base url is mandatory"),
                    vault_secret_path.expect("ERROR - vault secret path is mandatory"),
                    vault_role_id.expect("ERROR - vault role id is mandatory"),
                )
                .await
                .unwrap();

                /* let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    settings_hsm_group_name_opt,
                )
                .await; */

                // IMPORTANT: FOR SAT FILE, THERE IS NO POINT TO CONSIDER LOCKED HSM GROUP NAME IN
                // CONFIG FILE SINCE SAT FILES MAY USE MULTIPLE HSM GROUPS. THEREFORE HSM GROUP
                // VALIDATION CAN'T BE DONE AGAINST CONFIG FILE OR CLI HSM GROUP ARGUMENT AGAINST
                // HSM GROUPS AVAILABLE ACCORDING TO KEYCLOAK ROLES BUT HSM GROUPS IN SAT FILE VS
                // KEYCLOAK ROLES. BECAUASE OF THIS, THERE IS NO VALUE IN CALLING
                // 'get_target_hsm_group_vec_or_all' FUNCTION
                let target_hsm_group_vec = backend.get_group_name_available(&shasta_token).await?;
                /* let target_hsm_group_vec = config_show::get_hsm_name_available_from_jwt_or_all(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                )
                .await; */

                let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

                let cli_value_vec_opt: Option<Vec<String>> =
                    cli_apply_sat_file.get_many("values").map(|value_vec| {
                        value_vec
                            .map(|value: &String| value.replace("__DATE__", &timestamp))
                            .collect()
                    });

                let cli_values_file_content_opt: Option<String> = cli_apply_sat_file
                    .get_one("values-file")
                    .and_then(|values_file_path: &PathBuf| {
                        std::fs::read_to_string(values_file_path).ok().map(
                            |cli_value_file: String| cli_value_file.replace("__DATE__", &timestamp),
                        )
                    });

                let sat_file_content: String = std::fs::read_to_string(
                    cli_apply_sat_file
                        .get_one::<PathBuf>("sat-template-file")
                        .expect("ERROR: SAT file not found. Exit"),
                )
                .expect("ERROR: reading SAT file template. Exit");

                let ansible_passthrough_env = settings.get::<String>("ansible-passthrough").ok();
                let ansible_passthrough_cli_arg = cli_apply_sat_file
                    .get_one::<String>("ansible-passthrough")
                    .cloned();
                let ansible_passthrough = ansible_passthrough_env.or(ansible_passthrough_cli_arg);
                let ansible_verbosity: Option<u8> = cli_apply_sat_file
                    .get_one::<String>("ansible-verbosity")
                    .map(|ansible_verbosity| ansible_verbosity.parse::<u8>().unwrap());

                let prehook = cli_apply_sat_file.get_one::<String>("pre-hook");
                let posthook = cli_apply_sat_file.get_one::<String>("post-hook");

                let do_not_reboot: bool = cli_apply_sat_file.get_flag("do-not-reboot");
                let watch_logs: bool = cli_apply_sat_file.get_flag("watch-logs");
                let assume_yes: bool = cli_apply_sat_file.get_flag("assume-yes");

                /* let dry_run: bool = *cli_apply_sat_file.get_one("dry-run").unwrap();
                // If dry_run is true, then no need to reboot because no changes to the system are
                // applied, otherwise, depends on user input
                let do_not_reboot: bool = if dry_run {
                    false
                } else {
                    cli_apply_sat_file.get_flag("do-not-reboot")
                }; */

                let site = configuration
                    .sites
                    .get(&configuration.site.clone())
                    .unwrap();

                apply_sat_file::command::exec(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    vault_base_url.expect("ERROR - vault_base_url is mandatory"),
                    vault_secret_path.expect("ERROR - vault_secret_path is mandatory"),
                    vault_role_id.expect("ERROR - vault_role_id is mandatory"),
                    k8s_api_url.expect("ERROR - k8s_api_url is mandatory"),
                    sat_file_content,
                    cli_values_file_content_opt,
                    cli_value_vec_opt,
                    settings_hsm_group_name_opt,
                    &target_hsm_group_vec,
                    ansible_verbosity,
                    ansible_passthrough.as_ref(),
                    gitea_base_url,
                    &gitea_token,
                    do_not_reboot,
                    watch_logs,
                    prehook,
                    posthook,
                    cli_apply_sat_file.get_flag("image-only"),
                    cli_apply_sat_file.get_flag("sessiontemplate-only"),
                    true,
                    false,
                    assume_yes,
                    &site
                        .k8s
                        .as_ref()
                        .expect("ERROR - k8s section not found in configuration"), // FIXME:
                )
                .await;
            } else if let Some(cli_apply_template) = cli_apply.subcommand_matches("template") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let bos_session_name_opt: Option<&String> = cli_apply_template.get_one("name");
                let bos_sessiontemplate_name: &String = cli_apply_template
                    .get_one("template")
                    .expect("ERROR - template name is mandatory");
                let limit_opt: Option<&String> = cli_apply_template.get_one("limit");
                let bos_session_operation: &String = cli_apply_template
                    .get_one("operation")
                    .expect("ERROR - operation is mandatory");

                let include_disabled: bool = *cli_apply_template
                    .get_one("include-disabled")
                    .expect("ERROR - include disabled must have a value");

                let dry_run: bool = *cli_apply_template
                    .get_one("dry-run")
                    .expect("ERROR - dry-run must have a value");

                apply_template::exec(
                    &backend,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    bos_session_name_opt,
                    &bos_sessiontemplate_name,
                    &bos_session_operation,
                    limit_opt,
                    include_disabled,
                    dry_run,
                )
                .await;
            } else if let Some(cli_apply_ephemeral_environment) =
                cli_apply.subcommand_matches("ephemeral-environment")
            {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                if !std::io::stdout().is_terminal() {
                    eprintln!("This command needs to run in interactive mode. Exit");
                    std::process::exit(1);
                }

                apply_ephemeral_env::exec(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    // cli_apply_ephemeral_environment
                    //     .get_one::<bool>("block")
                    //     .copied(),
                    cli_apply_ephemeral_environment
                        .get_one::<String>("image-id")
                        .unwrap(),
                )
                .await;
            } else if let Some(cli_apply_boot) = cli_apply.subcommand_matches("boot") {
                if let Some(cli_apply_boot_nodes) = cli_apply_boot.subcommand_matches("nodes") {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let xname_vec: Vec<&str> = cli_apply_boot_nodes
                        .get_one::<String>("XNAMES")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim())
                        .collect();

                    let new_boot_image_id_opt: Option<&String> =
                        cli_apply_boot_nodes.get_one("boot-image");

                    if let Some(new_boot_image_id) = new_boot_image_id_opt {
                        if uuid::Uuid::parse_str(new_boot_image_id).is_err() {
                            eprintln!("ERROR - image id is not an UUID");
                            std::process::exit(1);
                        }
                    }

                    let new_boot_image_configuration_opt: Option<&String> =
                        cli_apply_boot_nodes.get_one("boot-image-configuration");

                    let new_runtime_configuration_opt: Option<&String> =
                        cli_apply_boot_nodes.get_one("runtime-configuration");

                    let new_kernel_parameters_opt: Option<&String> =
                        cli_apply_boot_nodes.get_one::<String>("kernel-parameters");

                    let dry_run = cli_apply_boot_nodes.get_flag("dry-run");

                    let assume_yes: bool = cli_apply_boot_nodes.get_flag("assume-yes");

                    apply_boot_node::exec(
                        &backend,
                        &shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        new_boot_image_id_opt,
                        new_boot_image_configuration_opt,
                        new_runtime_configuration_opt,
                        new_kernel_parameters_opt,
                        xname_vec,
                        assume_yes,
                        dry_run,
                        kafka_audit_opt,
                    )
                    .await;
                } else if let Some(cli_apply_boot_cluster) =
                    cli_apply_boot.subcommand_matches("cluster")
                {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let hsm_group_name_arg = cli_apply_boot_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("ERROR - cluster name must be provided");

                    let new_boot_image_id_opt: Option<&String> =
                        cli_apply_boot_cluster.get_one("boot-image");

                    let new_boot_image_configuration_opt: Option<&String> =
                        cli_apply_boot_cluster.get_one("boot-image-configuration");

                    let new_runtime_configuration_opt: Option<&String> =
                        cli_apply_boot_cluster.get_one("runtime-configuration");

                    let new_kernel_parameters_opt: Option<&String> =
                        cli_apply_boot_cluster.get_one::<String>("kernel-parameters");

                    let assume_yes = cli_apply_boot_cluster.get_flag("assume-yes");

                    let dry_run = cli_apply_boot_cluster.get_flag("dry-run");

                    // Validate
                    //
                    // Check user has provided valid HSM group name
                    let target_hsm_group_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        /* shasta_base_url,
                        shasta_root_cert, */
                        Some(hsm_group_name_arg),
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    let target_hsm_group_name = target_hsm_group_vec
                        .first()
                        .expect("ERROR - Could not find valid HSM group name");

                    apply_boot_cluster::exec(
                        &backend,
                        &shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        new_boot_image_id_opt,
                        new_boot_image_configuration_opt,
                        new_runtime_configuration_opt,
                        new_kernel_parameters_opt,
                        target_hsm_group_name,
                        assume_yes,
                        dry_run,
                        kafka_audit_opt,
                    )
                    .await;
                }
            }
        } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
            /* let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?; */

            let shasta_token = backend.get_api_token(&site_name).await?;

            // Get all HSM groups the user has access
            /* let target_hsm_group_vec = get_hsm_name_available_from_jwt_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
            )
            .await; */

            let user_input = cli_log
                .get_one::<String>("VALUE")
                .expect("ERROR - value is mandatory");

            let group_available_vec = backend.get_group_available(&shasta_token).await?;

            let site = configuration
                .sites
                .get(&configuration.site.clone())
                .unwrap();

            commands::log::exec(
                &backend,
                &shasta_token,
                shasta_base_url,
                shasta_root_cert,
                k8s_api_url.expect("ERROR - k8s api url is mandatory"),
                &group_available_vec,
                user_input,
                &site
                    .k8s
                    .as_ref()
                    .expect("ERROR - k8s section not found in configuration"), // FIXME:
            )
            .await;
        } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
            if let Some(cli_console_node) = cli_console.subcommand_matches("node") {
                if !std::io::stdout().is_terminal() {
                    eprintln!("This command needs to run in interactive mode. Exit");
                    std::process::exit(1);
                }
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let site = configuration
                    .sites
                    .get(&configuration.site.clone())
                    .unwrap();

                console_node::exec(
                    &backend,
                    settings_hsm_group_name_opt,
                    // cli_console,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    // k8s_api_url,
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
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    None,
                    settings_hsm_group_name_opt,
                )
                .await?;

                let site = configuration
                    .sites
                    .get(&configuration.site.clone())
                    .unwrap();

                console_cfs_session_image_target_ansible::exec(
                    &target_hsm_group_vec,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    /* vault_base_url,
                    vault_secret_path,
                    vault_role_id, */
                    k8s_api_url.expect("ERROR - k8s api url is mandatory"),
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
                let shasta_token = backend
                    .get_api_token(
                        /* shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url, */
                        &site_name,
                    )
                    .await?;
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */

                let dry_run: bool = *cli_migrate_nodes.get_one("dry-run").unwrap();

                let from_opt: Option<&String> = cli_migrate_nodes.get_one("from");
                let to: &String = cli_migrate_nodes
                    .get_one("to")
                    .expect("to value is mandatory");

                let xnames_string: &String = cli_migrate_nodes.get_one("XNAMES").unwrap();

                // Get target hsm group from either cli arguments or config and validate
                let from_rslt = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
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
                /* let from_rslt = get_target_hsm_name_group_vec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    from_opt,
                    settings_hsm_group_name_opt,
                )
                .await; */

                // Validate 'to' hsm groups
                let to_rslt = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    Some(to),
                    settings_hsm_group_name_opt,
                )
                .await;
                /* let to_rslt = get_target_hsm_name_group_vec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(to),
                    settings_hsm_group_name_opt,
                )
                .await; */

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
            } else if let Some(_cli_migrate_vcluster) = cli_migrate.subcommand_matches("vCluster") {
                if let Some(cli_migrate_vcluster_backup) = cli_migrate.subcommand_matches("backup")
                {
                    /* let shasta_token = &authentication::get_api_token(
                        shasta_base_url,
                        shasta_root_cert,
                        keycloak_base_url,
                        &site_name,
                    )
                    .await?; */
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let bos = cli_migrate_vcluster_backup.get_one::<String>("bos");
                    let destination = cli_migrate_vcluster_backup.get_one::<String>("destination");
                    let prehook = cli_migrate_vcluster_backup.get_one::<String>("pre-hook");
                    let posthook = cli_migrate_vcluster_backup.get_one::<String>("post-hook");
                    migrate_backup::exec(
                        &backend,
                        &shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        bos,
                        destination,
                        prehook,
                        posthook,
                    )
                    .await;
                } else if let Some(cli_migrate_vcluster_restore) =
                    cli_migrate.subcommand_matches("restore")
                {
                    let shasta_token = backend.get_api_token(&site_name).await?;

                    let bos_file = cli_migrate_vcluster_restore.get_one::<String>("bos-file");
                    let cfs_file = cli_migrate_vcluster_restore.get_one::<String>("cfs-file");
                    let hsm_file = cli_migrate_vcluster_restore.get_one::<String>("hsm-file");
                    let ims_file = cli_migrate_vcluster_restore.get_one::<String>("ims-file");
                    let image_dir = cli_migrate_vcluster_restore.get_one::<String>("image-dir");
                    let prehook = cli_migrate_vcluster_restore.get_one::<String>("pre-hook");
                    let posthook = cli_migrate_vcluster_restore.get_one::<String>("post-hook");

                    commands::migrate_restore::exec(
                        &backend,
                        &shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        bos_file,
                        cfs_file,
                        hsm_file,
                        ims_file,
                        image_dir,
                        prehook,
                        posthook,
                    )
                    .await;
                }
            }
        } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
            if let Some(cli_delete_group) = cli_delete.subcommand_matches("group") {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let label: &String = cli_delete_group
                    .get_one("VALUE")
                    .expect("ERROR - group name argument is mandatory");

                let force: bool = *cli_delete_group
                    .get_one("force")
                    .expect("The 'force' argument must have a value");

                delete_group::exec(&backend, &shasta_token, label, force).await;
            } else if let Some(cli_delete_hw_configuration) =
                cli_delete.subcommand_matches("hardware")
            {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let nodryrun = *cli_delete_hw_configuration
                    .get_one::<bool>("no-dryrun")
                    .unwrap_or(&true);

                let delete_hsm_group = *cli_delete_hw_configuration
                    .get_one::<bool>("delete-hsm-group")
                    .unwrap_or(&false);

                let target_hsm_group_name_arg_opt =
                    cli_delete_hw_configuration.get_one::<String>("target-cluster");

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    target_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await?;

                // let parent_hsm_group_name_arg_opt =
                //     cli_remove_hw_configuration.get_one::<String>("PARENT_CLUSTER_NAME");

                let parent_hsm_group_name_arg_opt =
                    cli_delete_hw_configuration.get_one::<String>("parent-cluster");

                let parent_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
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
                    nodryrun,
                    delete_hsm_group,
                )
                .await;
            } else if let Some(cli_delete_kernel_parameters) =
                cli_delete.subcommand_matches("kernel-parameters")
            {
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_group_name_arg_opt = cli_delete_kernel_parameters.get_one("hsm-group");

                let xname_vec: Vec<String> = if hsm_group_name_arg_opt.is_some() {
                    let hsm_group_name_vec = get_groups_available(
                        &backend,
                        &shasta_token,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await?;

                    let hsm_members_rslt = backend
                        .get_member_vec_from_group_name_vec(&shasta_token, hsm_group_name_vec)
                        .await;

                    match hsm_members_rslt {
                        Ok(hsm_members) => hsm_members,
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
                        .get_one::<String>("xnames")
                        .expect("Neither HSM group nor xnames defined")
                        .split(",")
                        .map(|value| value.to_string())
                        .collect()
                };

                // Validate user has access to the list of xnames requested
                validate_target_hsm_members(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    &xname_vec,
                )
                .await;

                let kernel_parameters = cli_delete_kernel_parameters
                    .get_one::<String>("VALUE")
                    .unwrap(); // clap should validate the argument

                let assume_yes: bool = cli_delete_kernel_parameters.get_flag("assume-yes");

                let result = delete_kernel_parameters::exec(
                    backend,
                    &shasta_token,
                    kernel_parameters,
                    xname_vec,
                    assume_yes,
                    kafka_audit_opt,
                )
                .await;

                match result {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                }
            } else if let Some(cli_delete_session) = cli_delete.subcommand_matches("session") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let target_hsm_group_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
                    None,
                    settings_hsm_group_name_opt,
                )
                .await?;

                let session_name = cli_delete_session
                    .get_one::<String>("SESSION_NAME")
                    .expect("'session-name' argument must be provided");

                let dry_run: &bool = cli_delete_session
                    .get_one("dry-run")
                    .expect("'dry-run' argument must be provided");

                delete_sessions::command::exec(
                    &backend,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec,
                    session_name,
                    dry_run,
                    kafka_audit_opt,
                )
                .await;
            } else if let Some(cli_delete_configurations) =
                cli_delete.subcommand_matches("configurations")
            {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let target_hsm_group_vec =
                    if let Some(settings_hsm_group_name) = settings_hsm_group_name_opt {
                        vec![settings_hsm_group_name.clone()]
                    } else {
                        get_groups_available(
                            &backend,
                            &shasta_token,
                            /* shasta_base_url,
                            shasta_root_cert, */
                            None,
                            settings_hsm_group_name_opt,
                        )
                        .await?
                    };

                let since_opt =
                    if let Some(since) = cli_delete_configurations.get_one::<String>("since") {
                        let date_time = chrono::NaiveDateTime::parse_from_str(
                            &(since.to_string() + "T00:00:00"),
                            "%Y-%m-%dT%H:%M:%S",
                        )
                        .unwrap();
                        Some(date_time)
                    } else {
                        None
                    };

                let until_opt =
                    if let Some(until) = cli_delete_configurations.get_one::<String>("until") {
                        let date_time = chrono::NaiveDateTime::parse_from_str(
                            &(until.to_string() + "T00:00:00"),
                            "%Y-%m-%dT%H:%M:%S",
                        )
                        .unwrap();
                        Some(date_time)
                    } else {
                        None
                    };

                let cfs_configuration_name_opt =
                    cli_delete_configurations.get_one::<String>("configuration-name");

                let cfs_configuration_name_pattern =
                    cli_delete_configurations.get_one::<String>("pattern");

                let yes = cli_delete_configurations
                    .get_one::<bool>("assume-yes")
                    .unwrap_or(&false);

                // INPUT VALIDATION - Check since date is prior until date
                if since_opt.is_some()
                    && until_opt.is_some()
                    && since_opt.unwrap() > until_opt.unwrap()
                {
                    eprintln!("ERROR - 'since' date can't be after 'until' date. Exit");
                    std::process::exit(1);
                }

                delete_data_related_cfs_configuration(
                    &backend,
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec,
                    cfs_configuration_name_opt,
                    cfs_configuration_name_pattern,
                    since_opt,
                    until_opt,
                    yes,
                )
                .await;
            } else if let Some(cli_delete_images) = cli_delete.subcommand_matches("images") {
                /* let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?; */
                let shasta_token = backend.get_api_token(&site_name).await?;

                let hsm_name_available_vec = get_groups_available(
                    &backend,
                    &shasta_token,
                    /* shasta_base_url,
                    shasta_root_cert, */
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

                delete_image::command::exec(
                    &shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_name_available_vec,
                    image_id_vec.as_slice(),
                    dry_run,
                )
                .await;
            }
        } else if let Some(cli_validate_local_repo) =
            cli_root.subcommand_matches("validate-local-repo")
        {
            /* let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?; */
            // let shasta_token = backend.get_api_token(&site_name).await?;

            // FIXME: gitea auth token should be calculated before colling this function
            let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
                vault_base_url.expect("ERROR - vault base url is mandatory"),
                vault_secret_path.expect("ERROR - vault secret path is mandatory"),
                vault_role_id.expect("ERROR - vault role id is mandatory"),
            )
            .await
            .unwrap();

            let repo_path = cli_validate_local_repo
                .get_one::<String>("repo-path")
                .unwrap();

            validate_local_repo::exec(shasta_root_cert, gitea_base_url, &gitea_token, repo_path)
                .await;
        } else if let Some(cli_add_nodes) = cli_root.subcommand_matches("add-nodes-to-groups") {
            /* let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?; */
            let shasta_token = backend.get_api_token(&site_name).await?;

            let dryrun = *cli_add_nodes
                .get_one::<bool>("dry-run")
                .expect("'dry-run' argument must be provided");

            let nodes = cli_add_nodes.get_one::<String>("nodes").unwrap();

            let target_hsm_name: &String = cli_add_nodes
                .get_one::<String>("group")
                .expect("Error - target cluster is mandatory");

            let is_regex = *cli_add_nodes.get_one::<bool>("regex").unwrap_or(&true);

            add_nodes_to_hsm_groups::exec(
                &backend,
                &shasta_token,
                target_hsm_name,
                is_regex,
                nodes,
                dryrun,
                kafka_audit_opt,
            )
            .await;
        } else if let Some(cli_remove_nodes) =
            cli_root.subcommand_matches("remove-nodes-from-groups")
        {
            /* let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?; */
            let shasta_token = backend.get_api_token(&site_name).await?;

            let dryrun = *cli_remove_nodes
                .get_one::<bool>("dry-run")
                .expect("'dry-run' argument must be provided");

            let nodes = cli_remove_nodes.get_one::<String>("nodes").unwrap();

            let target_hsm_name: &String = cli_remove_nodes
                .get_one::<String>("group")
                .expect("Error - target cluster is mandatory");

            let is_regex = *cli_remove_nodes.get_one::<bool>("regex").unwrap_or(&true);

            remove_nodes_from_hsm_groups::exec(
                &backend,
                &shasta_token,
                target_hsm_name,
                is_regex,
                nodes,
                dryrun,
                kafka_audit_opt,
            )
            .await;
        }
    }

    Ok(())
}
