use clap_complete::{generate, generate_to};
use std::{
    env,
    io::{self, IsTerminal},
    path::PathBuf,
};

use clap::Command;
use config::Config;
use k8s_openapi::chrono;
use mesa::{common::authentication, error::Error};

use crate::{cli::commands::validate_local_repo, common::kafka::Kafka};

use super::commands::{
    self, add_hw_component_cluster, add_kernel_parameters, add_nodes_to_hsm_groups,
    apply_boot_cluster, apply_boot_node, apply_configuration, apply_ephemeral_env,
    apply_hw_cluster_pin, apply_hw_cluster_unpin, apply_sat_file, apply_session, apply_template,
    config_set_hsm, config_set_log, config_set_parent_hsm, config_set_site, config_show,
    config_unset_auth, config_unset_hsm, config_unset_parent_hsm,
    console_cfs_session_image_target_ansible, console_node,
    delete_data_related_to_cfs_configuration::delete_data_related_cfs_configuration, delete_group,
    delete_hw_component_cluster, delete_image, delete_kernel_parameters, delete_sessions,
    get_cluster, get_configuration, get_hsm, get_hw_configuration_node, get_images,
    get_kernel_parameters, get_nodes, get_session, get_template, migrate_backup,
    migrate_nodes_between_hsm_groups, power_off_cluster, power_off_nodes, power_on_cluster,
    power_on_nodes, power_reset_cluster, power_reset_nodes, remove_nodes_from_hsm_groups,
};

pub async fn process_cli(
    mut cli: Command,
    keycloak_base_url: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    gitea_token: &str,
    gitea_base_url: &str,
    settings_hsm_group_name_opt: Option<&String>,
    // hsm_group_available_vec: &[String],
    // site_available_vec: &[String],
    // base_image_id: &str,
    k8s_api_url: &str,
    settings: &Config,
    kafka_audit: &Kafka,
    site_name: &str,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
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
            let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
                &site_name,
            )
            .await?;

            config_show::exec(shasta_token, shasta_base_url, shasta_root_cert, settings).await;
        } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
            if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
                let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?;

                config_set_hsm::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_config_set_hsm.get_one::<String>("HSM_GROUP_NAME"),
                    // hsm_available_vec,
                )
                .await;
            }
            if let Some(cli_config_set_parent_hsm) = cli_config_set.subcommand_matches("parent-hsm")
            {
                let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?;

                config_set_parent_hsm::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
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
                let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                    &site_name,
                )
                .await?;

                config_unset_parent_hsm::exec(shasta_token).await;
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
        let shasta_token = &authentication::get_api_token(
            shasta_base_url,
            shasta_root_cert,
            keycloak_base_url,
            &site_name,
        )
        .await?;

        /* let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await; */

        // println!("hsm_group: {:?}", hsm_group);
        // println!("hsm group available: {:?}", hsm_available_vec);

        // Validate hsm_vailable and hsm_group
        /* if hsm_group.is_none() && !hsm_available_vec.is_empty() {
            eprintln!("HSM group not defined. Please use 'manta config hsm <HSM group name>' to set the HSM group to use in your requests. Exit");
            std::process::exit(1);
        } */

        if let Some(cli_power) = cli_root.subcommand_matches("power") {
            if let Some(cli_power_on) = cli_power.subcommand_matches("on") {
                if let Some(cli_power_on_cluster) = cli_power_on.subcommand_matches("cluster") {
                    let hsm_group_name_arg = cli_power_on_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("The 'cluster name' argument must have a value");

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        Some(hsm_group_name_arg),
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    let target_hsm_group = target_hsm_group_vec
                        .first()
                        .expect("The 'cluster name' argument must have a value");

                    let assume_yes: bool = cli_power_on_cluster.get_flag("assume-yes");

                    let output: &str = cli_power_on_cluster.get_one::<String>("output").unwrap();

                    power_on_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group,
                        assume_yes,
                        output,
                        kafka_audit,
                    )
                    .await;
                } else if let Some(cli_power_on_node) = cli_power_on.subcommand_matches("nodes") {
                    let xname_requested: &str = cli_power_on_node
                        .get_one::<String>("VALUE")
                        .expect("The 'xnames' argument must have values");

                    let is_regex = *cli_power_on_node.get_one::<bool>("regex").unwrap_or(&true);

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
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_requested,
                        is_regex,
                        assume_yes,
                        output,
                        kafka_audit,
                    )
                    .await;
                }
            } else if let Some(cli_power_off) = cli_power.subcommand_matches("off") {
                if let Some(cli_power_off_cluster) = cli_power_off.subcommand_matches("cluster") {
                    let hsm_group_name_arg = cli_power_off_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("The 'cluster name' argument must have a value");

                    let force = cli_power_off_cluster
                        .get_one::<bool>("graceful")
                        .expect("The 'graceful' argument must have a value");

                    let output: &str = cli_power_off_cluster.get_one::<String>("output").unwrap();

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        Some(hsm_group_name_arg),
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    let target_hsm_group = target_hsm_group_vec
                        .first()
                        .expect("The 'cluster name' argument must have a value");

                    let assume_yes: bool = cli_power_off_cluster.get_flag("assume-yes");

                    power_off_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group,
                        *force,
                        assume_yes,
                        output,
                        kafka_audit,
                    )
                    .await;
                } else if let Some(cli_power_off_node) = cli_power_off.subcommand_matches("nodes") {
                    let xname_requested: &str = cli_power_off_node
                        .get_one::<String>("VALUE")
                        .expect("The 'xnames' argument must have values");

                    let force = cli_power_off_node
                        .get_one::<bool>("graceful")
                        .expect("The 'graceful' argument must have a value");

                    let is_regex = *cli_power_off_node.get_one::<bool>("regex").unwrap_or(&true);

                    let assume_yes: bool = cli_power_off_node.get_flag("assume-yes");

                    let output: &str = cli_power_off_node.get_one::<String>("output").unwrap();

                    power_off_nodes::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_requested,
                        is_regex,
                        *force,
                        assume_yes,
                        output,
                        kafka_audit,
                    )
                    .await;
                }
            } else if let Some(cli_power_reset) = cli_power.subcommand_matches("reset") {
                if let Some(cli_power_reset_cluster) = cli_power_reset.subcommand_matches("cluster")
                {
                    let hsm_group_name_arg = cli_power_reset_cluster
                        .get_one::<String>("CLUSTER_NAME")
                        .expect("The 'cluster name' argument must have a value");

                    let force = cli_power_reset_cluster
                        .get_one::<bool>("graceful")
                        .expect("The 'graceful' argument must have a value");

                    let output: &str = cli_power_reset_cluster.get_one::<String>("output").unwrap();

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        Some(hsm_group_name_arg),
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    let target_hsm_group = target_hsm_group_vec
                        .first()
                        .expect("Power off cluster must operate against a cluster");

                    let assume_yes: bool = cli_power_reset_cluster.get_flag("assume-yes");

                    power_reset_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group,
                        *force,
                        assume_yes,
                        output,
                        kafka_audit,
                    )
                    .await;
                } else if let Some(cli_power_reset_node) =
                    cli_power_reset.subcommand_matches("nodes")
                {
                    let xname_requested: &str = cli_power_reset_node
                        .get_one::<String>("VALUE")
                        .expect("The 'xnames' argument must have values");

                    let force = cli_power_reset_node
                        .get_one::<bool>("graceful")
                        .expect("The 'graceful' argument must have a value");

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
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_requested,
                        is_regex,
                        *force,
                        assume_yes,
                        output,
                        kafka_audit,
                    )
                    .await;
                }
            }
        /* } else if let Some(cli_set) = cli_root.subcommand_matches("set") {
        if let Some(cli_set_runtime_configuration) =
            cli_set.subcommand_matches("runtime-configuration")
        {
            let hsm_group_name_arg_opt = cli_set_runtime_configuration.get_one("hsm-group");
            let xnames_arg_opt = cli_set_runtime_configuration.get_one::<String>("xnames");

            let target_hsm_group_vec_opt = if hsm_group_name_arg_opt.is_some() {
                // Validate HSM group name
                Some(
                    get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await,
                )
            } else {
                None
            };

            let xname_vec_opt = if let Some(xnames_arg) = xnames_arg_opt {
                // Get list of xnames and validate
                let xname_vec = xnames_arg
                    .split(",")
                    .map(|elem| elem.to_string())
                    .collect::<Vec<String>>();

                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_vec.clone(),
                )
                .await;

                Some(xname_vec)
            } else {
                None
            };

            let configuration_name = cli_set_runtime_configuration
                .get_one::<String>("configuration")
                .unwrap(); // clap should validate the argument

            let result = set_runtime_configuration::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                configuration_name,
                target_hsm_group_vec_opt.as_ref(),
                xname_vec_opt.as_ref(),
            )
            .await;

            match result {
                Ok(_) => {}
                Err(error) => eprintln!("{}", error),
            }
        } else if let Some(cli_set_boot_image) = cli_set.subcommand_matches("boot-image") {
            let hsm_group_name_arg_opt = cli_set_boot_image.get_one("hsm-group");
            let xnames_arg_opt = cli_set_boot_image.get_one::<String>("xnames");

            let target_hsm_group_vec_opt = if hsm_group_name_arg_opt.is_some() {
                Some(
                    get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await,
                )
            } else {
                None
            };

            let xname_vec_opt = if let Some(xnames_arg) = xnames_arg_opt {
                // Get list of xnames and validate
                let xname_vec = xnames_arg
                    .split(",")
                    .map(|elem| elem.to_string())
                    .collect::<Vec<String>>();

                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_vec.clone(),
                )
                .await;

                Some(xname_vec)
            } else {
                None
            };

            let boot_image = cli_set_boot_image.get_one::<String>("image-id").unwrap(); // clap should validate the argument

            let assume_yes = cli_set_boot_image.get_flag("assume-yes");

            let output = cli_set_boot_image
                .get_one::<String>("output")
                .expect("ERROR - output argument in cli missing");

            let result = set_boot_image::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                boot_image,
                target_hsm_group_vec_opt.as_ref(),
                xname_vec_opt.as_ref(),
                assume_yes,
                output,
            )
            .await;

            match result {
                Ok(_) => {}
                Err(error) => eprintln!("{}", error),
            }
        } else if let Some(cli_set_boot_configuration) =
            cli_set.subcommand_matches("boot-configuration")
        {
            let hsm_group_name_arg_opt = cli_set_boot_configuration.get_one("hsm-group");
            let xnames_arg_opt = cli_set_boot_configuration.get_one::<String>("xnames");

            let target_hsm_group_vec_opt = if hsm_group_name_arg_opt.is_some() {
                Some(
                    get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await,
                )
            } else {
                None
            };

            let xname_vec_opt = if let Some(xnames_arg) = xnames_arg_opt {
                // Get list of xnames and validate
                let xname_vec = xnames_arg
                    .split(",")
                    .map(|elem| elem.to_string())
                    .collect::<Vec<String>>();

                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_vec.clone(),
                )
                .await;

                Some(xname_vec)
            } else {
                None
            };

            let configuration_name = cli_set_boot_configuration
                .get_one::<String>("configuration")
                .unwrap(); // clap should validate the argument

            let result = set_boot_configuration::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                configuration_name,
                target_hsm_group_vec_opt.as_ref(),
                xname_vec_opt.as_ref(),
            )
            .await;

            match result {
                Ok(_) => {}
                Err(error) => eprintln!("{}", error),
            }
        } else if let Some(cli_set_kernel_parameters) =
            cli_set.subcommand_matches("kernel-parameters")
        {
            let hsm_group_name_arg_opt = cli_set_kernel_parameters.get_one("hsm-group");
            let xnames_arg_opt = cli_set_kernel_parameters.get_one::<String>("xnames");

            let target_hsm_group_vec_opt = if hsm_group_name_arg_opt.is_some() {
                Some(
                    get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await,
                )
            } else {
                None
            };

            let xname_vec_opt = if let Some(xnames_arg) = xnames_arg_opt {
                Some(
                    xnames_arg
                        .split(",")
                        .map(|elem| elem.to_string())
                        .collect::<Vec<String>>(),
                )
            } else {
                None
            };

            let kernel_parameters = cli_set_kernel_parameters
                .get_one::<String>("kernel-parameters")
                .unwrap(); // clap should validate the argument

            let assume_yes: bool = cli_set_kernel_parameters.get_flag("assume-yes");

            let result = delete_kernel_parameters::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                kernel_parameters,
                target_hsm_group_vec_opt.as_ref(),
                xname_vec_opt.as_ref(),
                assume_yes,
            )
            .await;

            match result {
                Ok(_) => {}
                Err(error) => eprintln!("{}", error),
            }
        } */
        } else if let Some(cli_add) = cli_root.subcommand_matches("add") {
            if let Some(cli_add_group) = cli_add.subcommand_matches("group") {
                let label = cli_add_group
                    .get_one::<String>("label")
                    .expect("ERROR - 'label' argument is mandatory");

                let hosts_string = cli_add_group.get_one::<String>("nodes");

                let is_regex = cli_add_group.get_flag("regex");

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

                commands::add_group::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    label,
                    hosts_string,
                    assume_yes,
                    is_regex,
                    dryrun,
                    kafka_audit,
                )
                .await;
            } else if let Some(cli_add_hw_configuration) =
                cli_add.subcommand_matches("hw-component")
            {
                let target_hsm_group_name_arg_opt =
                    cli_add_hw_configuration.get_one::<String>("target-cluster");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                let parent_hsm_group_name_arg_opt =
                    cli_add_hw_configuration.get_one::<String>("parent-cluster");

                let parent_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    parent_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;
                let _ = cli_add_hw_configuration.get_one::<String>("target-cluster");

                let nodryrun = *cli_add_hw_configuration
                    .get_one::<bool>("no-dryrun")
                    .unwrap_or(&true);

                let create_hsm_group = *cli_add_hw_configuration
                    .get_one::<bool>("create-hsm-group")
                    .unwrap_or(&false);

                add_hw_component_cluster::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
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
                let hsm_group_name_arg_opt = cli_add_kernel_parameters.get_one("hsm-group");
                let xnames_arg_opt = cli_add_kernel_parameters.get_one::<String>("xnames");

                let target_hsm_group_vec_opt = if hsm_group_name_arg_opt.is_some() {
                    Some(
                        get_target_hsm_group_vec_or_all(
                            shasta_token,
                            shasta_base_url,
                            shasta_root_cert,
                            hsm_group_name_arg_opt,
                            settings_hsm_group_name_opt,
                        )
                        .await,
                    )
                } else {
                    None
                };

                let xname_vec_opt = if let Some(xnames_arg) = xnames_arg_opt {
                    Some(
                        xnames_arg
                            .split(",")
                            .map(|elem| elem.to_string())
                            .collect::<Vec<String>>(),
                    )
                } else {
                    None
                };

                let kernel_parameters = cli_add_kernel_parameters
                    .get_one::<String>("VALUE")
                    .unwrap(); // clap should validate the argument

                let assume_yes: bool = cli_add_kernel_parameters.get_flag("assume-yes");

                let result = add_kernel_parameters::command::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    kernel_parameters,
                    target_hsm_group_vec_opt.as_ref(),
                    xname_vec_opt.as_ref(),
                    assume_yes,
                    kafka_audit,
                )
                .await;

                match result {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                }
            }
        } else if let Some(cli_get) = cli_root.subcommand_matches("get") {
            if let Some(cli_get_hw_configuration) = cli_get.subcommand_matches("hw-component") {
                if let Some(cli_get_hw_configuration_cluster) =
                    cli_get_hw_configuration.subcommand_matches("cluster")
                {
                    let hsm_group_name_arg_opt =
                        cli_get_hw_configuration_cluster.get_one::<String>("CLUSTER_NAME");

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    commands::get_hw_configuration_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_vec.first().unwrap(),
                        cli_get_hw_configuration_cluster.get_one::<String>("output"),
                    )
                    .await;
                } else if let Some(cli_get_hw_configuration_node) =
                    cli_get_hw_configuration.subcommand_matches("node")
                {
                    let xnames = cli_get_hw_configuration_node
                        .get_one::<String>("XNAMES")
                        .expect("HSM group name is needed at this point");

                    let xname_vec: Vec<String> =
                        xnames.split(',').map(|xname| xname.to_string()).collect();

                    validate_target_hsm_members(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xname_vec,
                    )
                    .await;

                    get_hw_configuration_node::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        xnames,
                        cli_get_hw_configuration_node.get_one::<String>("type"),
                        cli_get_hw_configuration_node.get_one::<String>("output"),
                    )
                    .await;
                }
            } else if let Some(cli_get_configuration) = cli_get.subcommand_matches("configurations")
            {
                let hsm_group_name_arg_rslt = cli_get_configuration.try_get_one("hsm-group");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_rslt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await;

                let limit: Option<&u8> =
                    if let Some(true) = cli_get_configuration.get_one("most-recent") {
                        Some(&1)
                    } else {
                        cli_get_configuration.get_one::<u8>("limit")
                    };

                get_configuration::exec(
                    gitea_base_url,
                    gitea_token,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_get_configuration.get_one::<String>("name"),
                    cli_get_configuration.get_one::<String>("pattern"),
                    &target_hsm_group_vec,
                    limit,
                    cli_get_configuration.get_one("output"),
                    &site_name,
                )
                .await;
            } else if let Some(cli_get_session) = cli_get.subcommand_matches("sessions") {
                let hsm_group_name_arg_opt = cli_get_session.try_get_one("hsm-group");

                let target_hsm_group_vec: Vec<String> = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await;

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
                    shasta_token,
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
                let hsm_group_name_arg_opt = cli_get_template.try_get_one("hsm-group");

                let output: &String = cli_get_template
                    .get_one("output")
                    .expect("ERROR - output must be a valid value");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await;

                let hsm_member_vec = mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec.clone(),
                )
                .await;

                let limit_number_opt = if let Some(limit) = cli_get_template.get_one("limit") {
                    Some(limit)
                } else if let Some(true) = cli_get_template.get_one("most-recent") {
                    Some(&1)
                } else {
                    None
                };

                get_template::exec(
                    shasta_token,
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
                let hsm_group_name_arg_opt = cli_get_cluster.get_one::<String>("HSM_GROUP_NAME");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_cluster::exec(
                    shasta_token,
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
                // Get list of nodes from cli argument
                let xname_requested: &str = cli_get_nodes
                    .get_one::<String>("XNAMES")
                    .expect("The 'xnames' argument must have values");

                let is_regex = *cli_get_nodes.get_one::<bool>("regex").unwrap_or(&true);

                let is_silent_nids = cli_get_nodes
                    .get_one::<bool>("nids-only-one-line")
                    .unwrap_or(&false);

                let is_siblings = cli_get_nodes.get_flag("include-siblings");

                let output = cli_get_nodes.get_one::<String>("output");

                let status = cli_get_nodes.get_one::<bool>("status").unwrap_or(&false);

                /* let node_vec: Vec<String> = cli_get_nodes
                .get_one::<String>("XNAMES")
                .expect("ERROR - need list of xnames")
                .clone()
                .split(",")
                .map(|xname_str| xname_str.trim().to_string())
                .collect();

                // Validate user has access to list of xnames
                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    node_vec.clone(),
                )
                .await; */

                get_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_requested,
                    is_siblings,
                    *is_silent_nids,
                    false,
                    output,
                    *status,
                    is_regex,
                )
                .await;
            } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
                log::warn!("Deprecated - Do not use this command.");

                let hsm_group_name_arg_opt = cli_get_hsm_groups.get_one::<String>("HSM_GROUP_NAME");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_hsm::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec.first().unwrap(),
                )
                .await;
            } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
                let hsm_group_name_arg_opt = cli_get_images.try_get_one("hsm-group");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await;

                get_images::exec(
                    shasta_token,
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
                let hsm_group_name_arg_opt =
                    cli_get_kernel_parameters.get_one::<String>("hsm-group");

                let output: &String = cli_get_kernel_parameters
                    .get_one("output")
                    .expect("ERROR - output value missing");

                let filter_opt: Option<&String> = cli_get_kernel_parameters.get_one("filter");

                let xnames: Vec<String> = if hsm_group_name_arg_opt.is_some() {
                    let hsm_group_name_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_vec,
                    )
                    .await
                } else {
                    cli_get_kernel_parameters
                        .get_one::<String>("xnames")
                        .expect("Neither HSM group nor xnames defined")
                        .split(",")
                        .map(|value| value.to_string())
                        .collect()
                };

                let _ = get_kernel_parameters::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xnames,
                    filter_opt,
                    output,
                )
                .await;
            }
        } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
            if let Some(cli_apply_hw) = cli_apply.subcommand_matches("hw-configuration") {
                if let Some(cli_apply_hw_cluster) = cli_apply_hw.subcommand_matches("cluster") {
                    let target_hsm_group_name_arg_opt =
                        cli_apply_hw_cluster.get_one::<String>("target-cluster");

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    let parent_hsm_group_name_arg_opt =
                        cli_apply_hw_cluster.get_one::<String>("parent-cluster");

                    let parent_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        parent_hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;
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
                            shasta_token,
                            shasta_base_url,
                            shasta_root_cert,
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
                            shasta_token,
                            shasta_base_url,
                            shasta_root_cert,
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
            } else if let Some(cli_apply_configuration) =
                cli_apply.subcommand_matches("configuration")
            {
                log::warn!("Deprecated - Please use 'manta apply sat-file' command instead.");

                get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    settings_hsm_group_name_opt,
                )
                .await;

                let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

                let cli_value_vec_opt: Option<Vec<String>> =
                    cli_apply_configuration.get_many("values").map(|value_vec| {
                        value_vec
                            .map(|value: &String| value.replace("__DATE__", &timestamp))
                            .collect()
                    });

                let cli_values_file_content_opt: Option<String> = cli_apply_configuration
                    .get_one("values-file")
                    .and_then(|values_file_path: &PathBuf| {
                        std::fs::read_to_string(values_file_path).ok().map(
                            |cli_value_file: String| cli_value_file.replace("__DATE__", &timestamp),
                        )
                    });

                let sat_file_content: String = std::fs::read_to_string(
                    cli_apply_configuration
                        .get_one::<PathBuf>("sat-template-file")
                        .expect("ERROR: SAT file not found. Exit"),
                )
                .expect("ERROR: reading SAT file template. Exit");

                let _ = apply_configuration::exec(
                    sat_file_content,
                    cli_values_file_content_opt,
                    cli_value_vec_opt,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    k8s_api_url,
                    gitea_base_url,
                    gitea_token,
                    // &tag,
                    cli_apply_configuration.get_one::<String>("output"),
                    &site_name,
                )
                .await;
            } else if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
                let hsm_group_name_arg_opt: Option<&String> =
                    cli_apply_session.try_get_one("hsm-group").unwrap_or(None);

                let cfs_conf_sess_name_opt: Option<&String> = cli_apply_session.get_one("name");
                let playbook_file_name_opt: Option<&String> =
                    cli_apply_session.get_one("playbook-name");

                let hsm_group_members_opt = cli_apply_session.get_one::<String>("ansible-limit");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                target_hsm_group_vec.first().unwrap();

                if let Some(ansible_limit) = hsm_group_members_opt {
                    validate_target_hsm_members(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        ansible_limit
                            .split(',')
                            .map(|xname| xname.trim().to_string())
                            .collect::<Vec<String>>(),
                    )
                    .await;
                }

                apply_session::exec(
                    gitea_token,
                    gitea_base_url,
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    k8s_api_url,
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
                    kafka_audit,
                )
                .await;
            /* } else if let Some(cli_apply_image) = cli_apply.subcommand_matches("image") {
            log::warn!("Deprecated - Please use 'manta apply sat-file' command instead.");

            get_target_hsm_group_vec_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                None,
                settings_hsm_group_name_opt,
            )
            .await;

            let hsm_group_available_vec = get_hsm_name_available_from_jwt_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
            )
            .await;

            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

            let cli_value_vec_opt: Option<Vec<String>> =
                cli_apply_image.get_many("values").map(|value_vec| {
                    value_vec
                        .map(|value: &String| value.replace("__DATE__", &timestamp))
                        .collect()
                });

            let cli_values_file_content_opt: Option<String> = cli_apply_image
                .get_one("values-file")
                .and_then(|values_file_path: &PathBuf| {
                    std::fs::read_to_string(values_file_path).ok().map(
                        |cli_value_file: String| cli_value_file.replace("__DATE__", &timestamp),
                    )
                });

            let sat_file_content: String = std::fs::read_to_string(
                cli_apply_image
                    .get_one::<PathBuf>("sat-template-file")
                    .expect("ERROR: SAT file not found. Exit"),
            )
            .expect("ERROR: reading SAT file template. Exit");

            apply_image::exec(
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                sat_file_content,
                cli_values_file_content_opt,
                cli_value_vec_opt,
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                // base_image_id,
                cli_apply_image
                    .get_one::<String>("ansible-verbosity")
                    .cloned()
                    .map(|ansible_verbosity| ansible_verbosity.parse::<u8>().unwrap()),
                cli_apply_image.get_one::<String>("ansible-passthrough"),
                cli_apply_image.get_one::<bool>("watch-logs"),
                // &tag,
                &hsm_group_available_vec,
                k8s_api_url,
                gitea_base_url,
                gitea_token,
                cli_apply_image.get_one::<String>("output"),
            )
            .await; */
            /* } else if let Some(cli_apply_cluster) = cli_apply.subcommand_matches("cluster") {
            log::warn!("Deprecated - Please use 'manta apply sat-file' command instead.");

            let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                None,
                settings_hsm_group_name_opt,
            )
            .await;

            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

            let cli_value_vec_opt: Option<Vec<String>> =
                cli_apply_cluster.get_many("values").map(|value_vec| {
                    value_vec
                        .map(|value: &String| value.replace("__DATE__", &timestamp))
                        .collect()
                });

            let cli_values_file_content_opt: Option<String> = cli_apply_cluster
                .get_one("values-file")
                .and_then(|values_file_path: &PathBuf| {
                    std::fs::read_to_string(values_file_path).ok().map(
                        |cli_value_file: String| cli_value_file.replace("__DATE__", &timestamp),
                    )
                });

            let sat_file_content: String = std::fs::read_to_string(
                cli_apply_cluster
                    .get_one::<PathBuf>("sat-template-file")
                    .expect("ERROR: SAT file not found. Exit"),
            )
            .expect("ERROR: reading SAT file template. Exit");

            apply_cluster::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                k8s_api_url,
                sat_file_content,
                cli_values_file_content_opt,
                cli_value_vec_opt,
                settings_hsm_group_name_opt,
                &target_hsm_group_vec,
                cli_apply_cluster
                    .get_one::<String>("ansible-verbosity")
                    .cloned()
                    .map(|ansible_verbosity| ansible_verbosity.parse::<u8>().unwrap()),
                cli_apply_cluster.get_one::<String>("ansible-passthrough"),
                gitea_base_url,
                gitea_token,
                // &tag,
                *cli_apply_cluster
                    .get_one::<bool>("do-not-reboot")
                    .unwrap_or(&false),
            )
            .await; */
            } else if let Some(cli_apply_sat_file) = cli_apply.subcommand_matches("sat-file") {
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
                let target_hsm_group_vec =
                    config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                    )
                    .await;

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

                let ansible_passthrough_env = settings.get::<String>("ansible_passthrough").ok();
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

                apply_sat_file::command::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    k8s_api_url,
                    sat_file_content,
                    cli_values_file_content_opt,
                    cli_value_vec_opt,
                    settings_hsm_group_name_opt,
                    &target_hsm_group_vec,
                    ansible_verbosity,
                    ansible_passthrough.as_ref(),
                    gitea_base_url,
                    gitea_token,
                    do_not_reboot,
                    watch_logs,
                    prehook,
                    posthook,
                    cli_apply_sat_file.get_flag("image-only"),
                    cli_apply_sat_file.get_flag("sessiontemplate-only"),
                    true,
                    false,
                    assume_yes,
                    kafka_audit,
                    &site_name,
                )
                .await;
            } else if let Some(cli_apply_template) = cli_apply.subcommand_matches("template") {
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
                    shasta_token,
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
            /* } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                log::warn!("Deprecated - Please use 'manta power on' command instead.");

                let xname_vec: Vec<String> = cli_apply_node_on
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .split(',')
                    .map(|xname| xname.trim().to_string())
                    .collect();

                let output = "table"; // 'apply node on' subcommand does not have output
                                      // argument so we are forcing the value

                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_vec.clone(),
                )
                .await;

                power_on_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &xname_vec,
                    cli_apply_node_on.get_one::<String>("reason").cloned(),
                    output,
                )
                .await;
            } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
                log::warn!("Deprecated - Please use 'manta power off' command instead.");

                /* apply_node_off::exec(
                    settings_hsm_group_name_opt,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_apply_node_off
                        .get_one::<String>("XNAMES")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim())
                        .collect(),
                    cli_apply_node_off.get_one::<String>("reason").cloned(),
                    *cli_apply_node_off.get_one::<bool>("force").unwrap(),
                )
                .await; */

                let xname_vec: Vec<String> = cli_apply_node_off
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .split(',')
                    .map(|xname| xname.trim().to_string())
                    .collect();

                let output = "table"; // 'apply node off' subcommand does not have output
                                      // argument so we are forcing the value

                let _ = validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_vec.clone(),
                )
                .await;

                power_off_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &xname_vec,
                    cli_apply_node_off.get_one::<String>("reason").cloned(),
                    *cli_apply_node_off.get_one::<bool>("force").unwrap(),
                    output,
                )
                .await;
            } else if let Some(cli_apply_node_reset) =
                cli_apply_node.subcommand_matches("reset")
            {
                log::warn!("Deprecated - Please use 'manta power reset' command instead.");

                /* apply_node_reset::exec(
                    settings_hsm_group_name_opt,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_apply_node_reset
                        .get_one::<String>("XNAMES")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim())
                        .collect(),
                    cli_apply_node_reset.get_one::<String>("reason"),
                    *cli_apply_node_reset
                        .get_one::<bool>("force")
                        .unwrap_or(&false),
                )
                .await; */

                let xname_vec: Vec<String> = cli_apply_node_reset
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .split(',')
                    .map(|xname| xname.trim().to_string())
                    .collect();

                let output = "table"; // 'apply node off' subcommand does not have output
                                      // argument so we are forcing the value
                let _ = validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    xname_vec.clone(),
                )
                .await;

                power_reset_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &xname_vec,
                    cli_apply_node_reset.get_one::<String>("reason").cloned(),
                    *cli_apply_node_reset.get_one::<bool>("force").unwrap(),
                    output,
                )
                .await;
            } */
            } else if let Some(cli_apply_ephemeral_environment) =
                cli_apply.subcommand_matches("ephemeral-environment")
            {
                if !std::io::stdout().is_terminal() {
                    eprintln!("This command needs to run in interactive mode. Exit");
                    std::process::exit(1);
                }

                apply_ephemeral_env::exec(
                    shasta_token,
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
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        new_boot_image_id_opt,
                        new_boot_image_configuration_opt,
                        new_runtime_configuration_opt,
                        new_kernel_parameters_opt,
                        xname_vec,
                        assume_yes,
                        dry_run,
                        kafka_audit,
                    )
                    .await;
                } else if let Some(cli_apply_boot_cluster) =
                    cli_apply_boot.subcommand_matches("cluster")
                {
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
                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        Some(hsm_group_name_arg),
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    let target_hsm_group_name = target_hsm_group_vec
                        .first()
                        .expect("ERROR - Could not find valid HSM group name");

                    apply_boot_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        new_boot_image_id_opt,
                        new_boot_image_configuration_opt,
                        new_runtime_configuration_opt,
                        new_kernel_parameters_opt,
                        target_hsm_group_name,
                        assume_yes,
                        dry_run,
                        kafka_audit,
                    )
                    .await;
                }
            }
        } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
            // Get all HSM groups the user has access
            /* let target_hsm_group_vec = get_hsm_name_available_from_jwt_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
            )
            .await; */

            let hsm_group_name_arg_opt = cli_log.try_get_one::<String>("cluster").unwrap_or(None);

            let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                hsm_group_name_arg_opt,
                settings_hsm_group_name_opt,
            )
            .await;

            let session_name = cli_log.get_one::<String>("SESSION_NAME");

            let host_name = cli_log.get_one::<String>("node");

            commands::log::exec(
                // cli_log,
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                k8s_api_url,
                &target_hsm_group_vec,
                session_name,
                settings_hsm_group_name_opt,
                host_name,
            )
            .await;
        } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
            if let Some(cli_console_node) = cli_console.subcommand_matches("node") {
                if !std::io::stdout().is_terminal() {
                    eprintln!("This command needs to run in interactive mode. Exit");
                    std::process::exit(1);
                }

                console_node::exec(
                    settings_hsm_group_name_opt,
                    // cli_console,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    k8s_api_url,
                    cli_console_node.get_one::<String>("XNAME").unwrap(),
                )
                .await;
            } else if let Some(cli_console_target_ansible) =
                cli_console.subcommand_matches("target-ansible")
            {
                if !std::io::stdout().is_terminal() {
                    eprintln!("This command needs to run in interactive mode. Exit");
                    std::process::exit(1);
                }

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    settings_hsm_group_name_opt,
                )
                .await;

                console_cfs_session_image_target_ansible::exec(
                    &target_hsm_group_vec,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    k8s_api_url,
                    cli_console_target_ansible
                        .get_one::<String>("SESSION_NAME")
                        .unwrap(),
                )
                .await;
            }
        } else if let Some(cli_migrate) = cli_root.subcommand_matches("migrate") {
            if let Some(cli_migrate_nodes) = cli_migrate.subcommand_matches("nodes") {
                let dry_run: bool = *cli_migrate_nodes.get_one("dry-run").unwrap();

                let from_opt: Option<&String> = cli_migrate_nodes.get_one("from");
                let to: &String = cli_migrate_nodes
                    .get_one("to")
                    .expect("to value is mandatory");

                let xnames_string: &String = cli_migrate_nodes.get_one("XNAMES").unwrap();

                // Get target hsm group from either cli arguments or config and validate
                let from_rslt = get_target_hsm_name_group_vec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
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
                let to_rslt = get_target_hsm_name_group_vec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
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
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    to,
                    from,
                    xnames_string,
                    !dry_run,
                    false,
                    kafka_audit,
                )
                .await;
            } else if let Some(_cli_migrate_vcluster) = cli_migrate.subcommand_matches("vCluster") {
                if let Some(cli_migrate_vcluster_backup) = cli_migrate.subcommand_matches("backup")
                {
                    let bos = cli_migrate_vcluster_backup.get_one::<String>("bos");
                    let destination = cli_migrate_vcluster_backup.get_one::<String>("destination");
                    let prehook = cli_migrate_vcluster_backup.get_one::<String>("pre-hook");
                    let posthook = cli_migrate_vcluster_backup.get_one::<String>("post-hook");
                    migrate_backup::exec(
                        shasta_token,
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
                    let bos_file = cli_migrate_vcluster_restore.get_one::<String>("bos-file");
                    let cfs_file = cli_migrate_vcluster_restore.get_one::<String>("cfs-file");
                    let hsm_file = cli_migrate_vcluster_restore.get_one::<String>("hsm-file");
                    let ims_file = cli_migrate_vcluster_restore.get_one::<String>("ims-file");
                    let image_dir = cli_migrate_vcluster_restore.get_one::<String>("image-dir");
                    let prehook = cli_migrate_vcluster_restore.get_one::<String>("pre-hook");
                    let posthook = cli_migrate_vcluster_restore.get_one::<String>("post-hook");
                    commands::migrate_restore::exec(
                        shasta_token,
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
                let label = cli_delete_group
                    .get_one::<String>("VALUE")
                    .expect("ERROR - 'label' argument is mandatory")
                    .clone();

                let assume_yes: bool = cli_delete_group.get_flag("assume-yes");

                let dryrun = cli_delete_group.get_flag("dryrun");

                delete_group::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &label,
                    assume_yes,
                    dryrun,
                    kafka_audit,
                )
                .await;
            } else if let Some(cli_delete_hw_configuration) =
                cli_delete.subcommand_matches("hw-component")
            {
                let nodryrun = *cli_delete_hw_configuration
                    .get_one::<bool>("no-dryrun")
                    .unwrap_or(&true);

                let delete_hsm_group = *cli_delete_hw_configuration
                    .get_one::<bool>("delete-hsm-group")
                    .unwrap_or(&false);

                let target_hsm_group_name_arg_opt =
                    cli_delete_hw_configuration.get_one::<String>("target-cluster");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                // let parent_hsm_group_name_arg_opt =
                //     cli_remove_hw_configuration.get_one::<String>("PARENT_CLUSTER_NAME");

                let parent_hsm_group_name_arg_opt =
                    cli_delete_hw_configuration.get_one::<String>("parent-cluster");

                let parent_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    parent_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                delete_hw_component_cluster::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
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
                let hsm_group_name_arg_opt = cli_delete_kernel_parameters.get_one("hsm-group");
                let xnames_arg_opt = cli_delete_kernel_parameters.get_one::<String>("xnames");

                let target_hsm_group_vec_opt = if hsm_group_name_arg_opt.is_some() {
                    Some(
                        get_target_hsm_group_vec_or_all(
                            shasta_token,
                            shasta_base_url,
                            shasta_root_cert,
                            hsm_group_name_arg_opt,
                            settings_hsm_group_name_opt,
                        )
                        .await,
                    )
                } else {
                    None
                };

                let xname_vec_opt = if let Some(xnames_arg) = xnames_arg_opt {
                    Some(
                        xnames_arg
                            .split(",")
                            .map(|elem| elem.to_string())
                            .collect::<Vec<String>>(),
                    )
                } else {
                    None
                };

                let kernel_parameters = cli_delete_kernel_parameters
                    .get_one::<String>("VALUE")
                    .unwrap(); // clap should validate the argument

                let assume_yes: bool = cli_delete_kernel_parameters.get_flag("assume-yes");

                let result = delete_kernel_parameters::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    kernel_parameters,
                    target_hsm_group_vec_opt.as_ref(),
                    xname_vec_opt.as_ref(),
                    assume_yes,
                    kafka_audit,
                )
                .await;

                match result {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                }
            } else if let Some(cli_delete_session) = cli_root.subcommand_matches("session") {
                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    settings_hsm_group_name_opt,
                )
                .await;

                let session_name = cli_delete_session
                    .get_one::<String>("SESSION_NAME")
                    .expect("'session-name' argument must be provided");

                let dry_run: &bool = cli_delete_session
                    .get_one("dry-run")
                    .expect("'dry-run' argument must be provided");

                delete_sessions::command::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec,
                    session_name,
                    dry_run,
                    kafka_audit,
                )
                .await;
            } else if let Some(cli_delete_images) = cli_root.subcommand_matches("images") {
                let hsm_name_available_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    settings_hsm_group_name_opt,
                )
                .await;

                let image_id_vec: Vec<&str> = cli_delete_images
                    .get_one::<String>("IMAGE_LIST")
                    .expect("'IMAGE_LIST' argument must be provided")
                    .split(",")
                    .map(|image_id_str| image_id_str.trim())
                    .collect();

                let dry_run: bool = cli_delete_images.get_flag("dry-run");

                delete_image::command::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_name_available_vec,
                    image_id_vec.as_slice(),
                    dry_run,
                )
                .await;
            }
        // } else if let Some(cli_clean_system) = cli_root.subcommand_matches("clean-system") {
        //     let hsm_group_name_arg_opt = cli_clean_system.get_one::<String>("hsm-group"); // For now, we
        //                                                                                   // want to panic if this param is missing
        //
        //     let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
        //         shasta_token,
        //         shasta_base_url,
        //         shasta_root_cert,
        //         hsm_group_name_arg_opt,
        //         settings_hsm_group_name_opt,
        //     )
        //     .await;
        //
        //     let since_opt = if let Some(since) = cli_clean_system.get_one::<String>("since") {
        //         let date_time = chrono::NaiveDateTime::parse_from_str(
        //             &(since.to_string() + "T00:00:00"),
        //             "%Y-%m-%dT%H:%M:%S",
        //         )
        //         .unwrap();
        //         Some(date_time)
        //     } else {
        //         None
        //     };
        //
        //     let until_opt = if let Some(until) = cli_clean_system.get_one::<String>("until") {
        //         let date_time = chrono::NaiveDateTime::parse_from_str(
        //             &(until.to_string() + "T00:00:00"),
        //             "%Y-%m-%dT%H:%M:%S",
        //         )
        //         .unwrap();
        //         Some(date_time)
        //     } else {
        //         None
        //     };
        //
        //     let cfs_configuration_name_opt =
        //         cli_clean_system.get_one::<String>("configuration-name");
        //
        //     let cfs_configuration_name_pattern = cli_clean_system.get_one::<String>("pattern");
        //
        //     let yes = cli_clean_system
        //         .get_one::<bool>("assume-yes")
        //         .unwrap_or(&false);
        //
        //     let hsm_group_name_opt = if settings_hsm_group_name_opt.is_some() {
        //         settings_hsm_group_name_opt
        //     } else {
        //         cli_clean_system.get_one::<String>("hsm-group")
        //     };
        //
        //     // INPUT VALIDATION - Check since date is prior until date
        //     if since_opt.is_some() && until_opt.is_some() && since_opt.unwrap() > until_opt.unwrap()
        //     {
        //         eprintln!("ERROR - 'since' date can't be after 'until' date. Exit");
        //         std::process::exit(1);
        //     }
        //
        //     delete_data_related_cfs_configuration(
        //         shasta_token,
        //         shasta_base_url,
        //         shasta_root_cert,
        //         hsm_group_name_opt,
        //         target_hsm_group_vec,
        //         cfs_configuration_name_opt,
        //         cfs_configuration_name_pattern,
        //         since_opt,
        //         until_opt,
        //         yes,
        //     )
        //     .await;
        } else if let Some(cli_validate_local_repo) =
            cli_root.subcommand_matches("validate-local-repo")
        {
            let repo_path = cli_validate_local_repo
                .get_one::<String>("repo-path")
                .unwrap();

            validate_local_repo::exec(shasta_root_cert, gitea_base_url, gitea_token, repo_path)
                .await;
        } else if let Some(cli_add_nodes) = cli_root.subcommand_matches("add-nodes-to-groups") {
            let dryrun = *cli_add_nodes
                .get_one::<bool>("dry-run")
                .expect("'dry-run' argument must be provided");

            let nodes = cli_add_nodes.get_one::<String>("nodes").unwrap();

            let target_hsm_name: &String = cli_add_nodes
                .get_one::<String>("group")
                .expect("Error - target cluster is mandatory");

            let is_regex = *cli_add_nodes.get_one::<bool>("regex").unwrap_or(&true);

            add_nodes_to_hsm_groups::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                target_hsm_name,
                is_regex,
                nodes,
                dryrun,
                kafka_audit,
            )
            .await;
        } else if let Some(cli_remove_nodes) =
            cli_root.subcommand_matches("remove-nodes-from-groups")
        {
            let dryrun = *cli_remove_nodes
                .get_one::<bool>("dry-run")
                .expect("'dry-run' argument must be provided");

            let nodes = cli_remove_nodes.get_one::<String>("nodes").unwrap();

            let target_hsm_name: &String = cli_remove_nodes
                .get_one::<String>("group")
                .expect("Error - target cluster is mandatory");

            let is_regex = *cli_remove_nodes.get_one::<bool>("regex").unwrap_or(&true);

            remove_nodes_from_hsm_groups::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                target_hsm_name,
                is_regex,
                nodes,
                dryrun,
                kafka_audit,
            )
            .await;
        }
    }

    Ok(())
}

pub fn validate_hsm_groups(
    target_hsm_name_vec: &Vec<String>,
    hsm_name_available_vec: Vec<String>,
) -> Result<(), Error> {
    for target_hsm_name in target_hsm_name_vec {
        if !hsm_name_available_vec.contains(&target_hsm_name) {
            let err_msg = format!(
                "Can't access HSM group '{}'.\nPlease choose one from the list below:\n{}\nExit",
                target_hsm_name,
                hsm_name_available_vec.join(", ")
            );

            return Err(Error::Message(err_msg));
        }
    }

    Ok(())
}
/// Returns a list of HSM groups the user is expected to work with.
/// This method will exit if the user is asking for HSM group not allowed
/// If the user did not requested any HSM group, then it will return all HSM
/// groups he has access to
/// hsm_group_cli_arg_opt: may contain a comma separated list of HSM groups defined in CLI command
/// arguments
/// hsm_group_env_or_config_file_opt: may contain a comma separated list of HSM groups defined in
/// either environment variable or configuration file
pub async fn get_target_hsm_name_group_vec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_cli_arg_opt: Option<&String>,
    hsm_group_env_or_config_file_opt: Option<&String>,
) -> Result<Vec<String>, Error> {
    let hsm_name_available_vec =
        config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await;

    let target_hsm_name_vec = if let Some(hsm_group_cli_arg) = hsm_group_cli_arg_opt {
        hsm_group_cli_arg
            .split(",")
            .map(|hsm_group| hsm_group.trim().to_string())
            .collect()
    } else if let Some(hsm_group_env_or_config_file) = hsm_group_env_or_config_file_opt {
        hsm_group_env_or_config_file
            .split(",")
            .map(|hsm_group| hsm_group.trim().to_string())
            .collect()
    } else {
        hsm_name_available_vec.clone()
    };

    let _ = validate_hsm_groups(&target_hsm_name_vec, hsm_name_available_vec);

    Ok(target_hsm_name_vec.clone())
}

/// Returns a list of HSM groups the user is expected to work with.
/// This method will exit if the user is asking for HSM group not allowed
/// If the user did not requested any HSM group, then it will return all HSM
/// groups he has access to
pub async fn get_target_hsm_group_vec_or_all(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_cli_arg_opt: Option<&String>,
    hsm_group_env_or_config_file_opt: Option<&String>,
) -> Vec<String> {
    let hsm_name_available_vec =
        config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await;

    let target_hsm_group_opt = if hsm_group_cli_arg_opt.is_some() {
        hsm_group_cli_arg_opt
    } else {
        hsm_group_env_or_config_file_opt
    };

    /* let target_hsm_group_opt = if let Some(hsm_group_name) = hsm_group_env_or_config_file_opt {
        Some(hsm_group_name)
    } else {
        hsm_group_cli_arg_opt
    }; */

    if let Some(target_hsm_group) = target_hsm_group_opt {
        if !hsm_name_available_vec.contains(target_hsm_group) {
            println!(
                "Can't access HSM group '{}'.\nPlease choose one from the list below:\n{}\nExit",
                target_hsm_group,
                hsm_name_available_vec.join(", ")
            );

            std::process::exit(1);
        }

        vec![target_hsm_group.to_string()]
    } else {
        hsm_name_available_vec
    }
}

/* /// Returns a list of HSM groups the user is expected to work with or none (empty vec) if user is
/// admin role and has not selected a HSM group to work with.
/// This method will exit if the user is asking for HSM group not allowed
/// Thie method is used by 'get session' function because CFS sessions related to management nodes
/// are not linked to any HSM group
pub async fn get_target_hsm_group_vec(
    shasta_token: &str,
    cli_param_hsm_group: Option<&String>,
    config_file_or_env_hsm_group: Option<&String>,
) -> Vec<String> {
    let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt(shasta_token).await;

    let target_hsm_group_opt = if let Some(hsm_group_name) = config_file_or_env_hsm_group {
        Some(hsm_group_name)
    } else {
        cli_param_hsm_group
    };

    if let Some(target_hsm_group) = target_hsm_group_opt {
        if !hsm_name_available_vec.contains(target_hsm_group) {
            println!(
                "Can't access HSM group '{}'.\nPlease choose one from the list below:\n{}\nExit",
                target_hsm_group,
                hsm_name_available_vec.join(", ")
            );

            std::process::exit(1);
        }

        vec![target_hsm_group.to_string()]
    } else {
        hsm_name_available_vec
    }
} */

/// Validate user has access to a list of HSM group members provided.
/// HSM members user is asking for are taken from cli command
/// Exit if user does not have access to any of the members provided. By not having access to a HSM
/// members means, the node belongs to an HSM group which the user does not have access
pub async fn validate_target_hsm_members(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_members_opt: Vec<String>,
) -> Vec<String> {
    let hsm_groups_user_has_access =
        config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await;

    let all_xnames_user_has_access = mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_groups_user_has_access.clone(),
    )
    .await;

    // Check user has access to all xnames he is requesting
    if hsm_group_members_opt
        .iter()
        .all(|hsm_member| all_xnames_user_has_access.contains(hsm_member))
    {
        hsm_group_members_opt
    } else {
        println!("Can't access all or any of the HSM members '{}'.\nPlease choose members form the list of HSM groups below:\n{}\nExit", hsm_group_members_opt.join(", "), hsm_groups_user_has_access.join(", "));
        std::process::exit(1);
    }
}
