use std::{io::IsTerminal, path::PathBuf};

use clap::ArgMatches;
use config::Config;
use k8s_openapi::chrono;
use mesa::common::authentication;

use super::commands::{
    self, add_hw_component_cluster, add_nodes, apply_cluster, apply_configuration,
    apply_ephemeral_env, apply_hw_cluster, apply_image, apply_sat_file, apply_session,
    config_set_hsm, config_set_log, config_set_parent_hsm, config_set_site,
    config_show::{self, get_hsm_name_available_from_jwt_or_all},
    config_unset_auth, config_unset_hsm, config_unset_parent_hsm,
    console_cfs_session_image_target_ansible, console_node,
    delete_data_related_to_cfs_configuration::delete_data_related_cfs_configuration,
    get_configuration, get_hsm, get_hw_configuration_node, get_images, get_nodes, get_session,
    get_template, migrate_backup, power_off_cluster, power_off_nodes, power_on_cluster,
    power_on_nodes, power_reset_cluster, power_reset_nodes, remove_hw_component_cluster,
    remove_nodes, update_hsm_group, update_node,
};

pub async fn process_cli(
    cli_root: ArgMatches,
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
                    let hsm_group_name_arg_opt =
                        cli_power_on_cluster.get_one::<String>("CLUSTER_NAME");

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    power_on_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_vec.first(),
                    )
                    .await;
                } else if let Some(cli_power_on_node) = cli_power_on.subcommand_matches("node") {
                    let xname_vec: Vec<String> = cli_power_on_node
                        .get_one::<String>("NODE_NAME")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim().to_string())
                        .collect();

                    let _ = validate_target_hsm_members(
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
                        xname_vec,
                        cli_power_on_node.get_one::<String>("reason").cloned(),
                    )
                    .await;
                }
            } else if let Some(cli_power_off) = cli_power.subcommand_matches("off") {
                if let Some(cli_power_off_cluster) = cli_power_off.subcommand_matches("cluster") {
                    let hsm_group_name_arg_opt =
                        cli_power_off_cluster.get_one::<String>("CLUSTER_NAME");

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    power_off_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_vec.first(),
                    )
                    .await;
                } else if let Some(cli_power_off_node) = cli_power_off.subcommand_matches("node") {
                    let xname_vec: Vec<String> = cli_power_off_node
                        .get_one::<String>("NODE_NAME")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim().to_string())
                        .collect();

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
                        xname_vec,
                        cli_power_off_node.get_one::<String>("reason").cloned(),
                        *cli_power_off_node.get_one::<bool>("force").unwrap(),
                    )
                    .await;
                }
            } else if let Some(cli_power_reset) = cli_power.subcommand_matches("reset") {
                if let Some(cli_power_reset_cluster) = cli_power_reset.subcommand_matches("cluster")
                {
                    let hsm_group_name_arg_opt =
                        cli_power_reset_cluster.get_one::<String>("CLUSTER_NAME");

                    let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;

                    power_reset_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_vec.first(),
                    )
                    .await;
                } else if let Some(cli_power_reset_node) =
                    cli_power_reset.subcommand_matches("node")
                {
                    let xname_vec: Vec<String> = cli_power_reset_node
                        .get_one::<String>("NODE_NAME")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim().to_string())
                        .collect();

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
                        xname_vec,
                        cli_power_reset_node.get_one::<String>("reason").cloned(),
                        *cli_power_reset_node.get_one::<bool>("force").unwrap(),
                    )
                    .await;
                }
            }
        } else if let Some(cli_add) = cli_root.subcommand_matches("add") {
            if let Some(cli_add_hw_configuration) = cli_add.subcommand_matches("hw-component") {
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

                add_hw_component_cluster::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec.first().unwrap(),
                    parent_hsm_group_vec.first().unwrap(),
                    cli_add_hw_configuration
                        .get_one::<String>("pattern")
                        .unwrap(),
                )
                .await;
            } else if let Some(cli_add_nodes) = cli_add.subcommand_matches("nodes") {
                add_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_add_nodes.get_one::<String>("target-cluster").unwrap(),
                    cli_add_nodes.get_one::<String>("parent-cluster").unwrap(),
                    cli_add_nodes.get_one::<String>("XNAMES").unwrap(),
                )
                .await;
            }
        } else if let Some(cli_remove) = cli_root.subcommand_matches("remove") {
            if let Some(cli_remove_hw_configuration) = cli_remove.subcommand_matches("hw-component")
            {
                let target_hsm_group_name_arg_opt =
                    cli_remove_hw_configuration.get_one::<String>("target-cluster");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                let parent_hsm_group_name_arg_opt =
                    cli_remove_hw_configuration.get_one::<String>("PARENT_CLUSTER_NAME");

                let parent_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    parent_hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                remove_hw_component_cluster::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_vec.first().unwrap(),
                    parent_hsm_group_vec.first().unwrap(),
                    cli_remove_hw_configuration
                        .get_one::<String>("pattern")
                        .unwrap(),
                )
                .await;
            } else if let Some(cli_remove_nodes) = cli_remove.subcommand_matches("nodes") {
                remove_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_remove_nodes
                        .get_one::<String>("target-cluster")
                        .unwrap(),
                    cli_remove_nodes
                        .get_one::<String>("parent-cluster")
                        .unwrap(),
                    cli_remove_nodes.get_one::<String>("XNAMES").unwrap(),
                )
                .await;
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
            } else if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration")
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
                    gitea_token,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_get_configuration.get_one::<String>("name"),
                    &target_hsm_group_vec,
                    limit,
                    cli_get_configuration.get_one("output"),
                )
                .await;
            } else if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
                let hsm_group_name_arg_opt = cli_get_session.try_get_one("hsm-group");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
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

                get_session::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &target_hsm_group_vec,
                    cli_get_session.get_one::<String>("name"),
                    limit,
                    cli_get_session.get_one("output"),
                )
                .await;
            } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
                let hsm_group_name_arg_opt = cli_get_template.try_get_one("hsm-group");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt.unwrap_or(None),
                    settings_hsm_group_name_opt,
                )
                .await;

                let hsm_member_vec =
                    mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_name_vec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        &target_hsm_group_vec,
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
                )
                .await;
            } else if let Some(cli_get_node) = cli_get.subcommand_matches("cluster") {
                let hsm_group_name_arg_opt = cli_get_node.get_one::<String>("HSM_GROUP_NAME");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &target_hsm_group_vec,
                    *cli_get_node
                        .get_one::<bool>("nids-only-one-line")
                        .unwrap_or(&false),
                    *cli_get_node
                        .get_one::<bool>("xnames-only-one-line")
                        .unwrap_or(&false),
                    cli_get_node.get_one::<String>("output"),
                    *cli_get_node.get_one::<bool>("status").unwrap_or(&false),
                )
                .await;
            } else if let Some(cli_get_node) = cli_get.subcommand_matches("nodes") {
                let hsm_group_name_arg_opt = cli_get_node.get_one::<String>("HSM_GROUP_NAME");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &target_hsm_group_vec,
                    *cli_get_node
                        .get_one::<bool>("nids-only-one-line")
                        .unwrap_or(&false),
                    *cli_get_node
                        .get_one::<bool>("xnames-only-one-line")
                        .unwrap_or(&false),
                    cli_get_node.get_one::<String>("output"),
                    false,
                )
                .await;
            } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
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
                    cli_get_images.get_one::<u8>("limit"),
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

                    apply_hw_cluster::exec(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_vec.first().unwrap(),
                        parent_hsm_group_vec.first().unwrap(),
                        cli_apply_hw_cluster.get_one::<String>("pattern").unwrap(),
                    )
                    .await;
                }
            } else if let Some(cli_apply_configuration) =
                cli_apply.subcommand_matches("configuration")
            {
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
                    gitea_token,
                    // &tag,
                    cli_apply_configuration.get_one::<String>("output"),
                )
                .await;
            } else if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
                let hsm_group_name_arg_opt = cli_apply_session.try_get_one("hsm-group");

                let hsm_group_members_opt = cli_apply_session.get_one::<String>("ansible-limit");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt.unwrap_or(None),
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
                    cli_apply_session.get_one::<String>("name").cloned(),
                    Some(target_hsm_group_vec.first().unwrap()),
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
                )
                .await;
            } else if let Some(cli_apply_image) = cli_apply.subcommand_matches("image") {
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
                    gitea_token,
                    cli_apply_image.get_one::<String>("output"),
                )
                .await;
            } else if let Some(cli_apply_cluster) = cli_apply.subcommand_matches("cluster") {
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
                    gitea_token,
                    // &tag,
                    *cli_apply_cluster
                        .get_one::<bool>("do-not-reboot")
                        .unwrap_or(&false),
                )
                .await;
            } else if let Some(cli_apply_sat_file) = cli_apply.subcommand_matches("sat-file") {
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

                apply_sat_file::exec(
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
                    cli_apply_sat_file
                        .get_one::<String>("ansible-verbosity")
                        .cloned()
                        .map(|ansible_verbosity| ansible_verbosity.parse::<u8>().unwrap()),
                    cli_apply_sat_file.get_one::<String>("ansible-passthrough"),
                    gitea_token,
                    // &tag,
                    *cli_apply_sat_file
                        .get_one::<bool>("do-not-reboot")
                        .unwrap_or(&false),
                )
                .await;
            } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
                if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                    /* apply_node_on::exec(
                        settings_hsm_group_name_opt,
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        cli_apply_node_on
                            .get_one::<String>("XNAMES")
                            .unwrap()
                            .split(',')
                            .map(|xname| xname.trim())
                            .collect(),
                        cli_apply_node_on.get_one::<String>("reason").cloned(),
                    )
                    .await; */

                    let xname_vec: Vec<String> = cli_apply_node_on
                        .get_one::<String>("XNAMES")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim().to_string())
                        .collect();

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
                        xname_vec,
                        cli_apply_node_on.get_one::<String>("reason").cloned(),
                    )
                    .await;
                } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
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
                        xname_vec,
                        cli_apply_node_off.get_one::<String>("reason").cloned(),
                        *cli_apply_node_off.get_one::<bool>("force").unwrap(),
                    )
                    .await;
                } else if let Some(cli_apply_node_reset) =
                    cli_apply_node.subcommand_matches("reset")
                {
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
                        xname_vec,
                        cli_apply_node_reset.get_one::<String>("reason").cloned(),
                        *cli_apply_node_reset.get_one::<bool>("force").unwrap(),
                    )
                    .await;
                }
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
            }
        } else if let Some(cli_update) = cli_root.subcommand_matches("update") {
            if let Some(cli_update_node) = cli_update.subcommand_matches("nodes") {
                let hsm_group_name_arg_opt = cli_update_node.get_one::<String>("HSM_GROUP_NAME");

                if hsm_group_name_arg_opt.is_some() {
                    get_target_hsm_group_vec_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_name_arg_opt,
                        settings_hsm_group_name_opt,
                    )
                    .await;
                }

                update_node::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    cli_update_node.get_one::<String>("boot-image"),
                    cli_update_node.get_one::<String>("desired-configuration"),
                    cli_update_node
                        .get_one::<String>("XNAMES")
                        .unwrap()
                        .split(',')
                        .map(|xname| xname.trim())
                        .collect(),
                )
                .await;
            } else if let Some(cli_update_hsm_group) = cli_update.subcommand_matches("hsm-group") {
                let hsm_group_name_arg_opt =
                    cli_update_hsm_group.get_one::<String>("HSM_GROUP_NAME");

                let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name_arg_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                update_hsm_group::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_update_hsm_group.get_one::<String>("boot-image"),
                    cli_update_hsm_group.get_one::<String>("desired-configuration"),
                    target_hsm_group_vec.first().unwrap(),
                )
                .await;
            }
        } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
            let hsm_group_name_arg_opt = cli_log.try_get_one::<String>("hsm-group").unwrap_or(None);

            let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                hsm_group_name_arg_opt,
                settings_hsm_group_name_opt,
            )
            .await;

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
                cli_log.get_one::<String>("SESSION_NAME"),
                settings_hsm_group_name_opt,
            )
            .await;
        /* } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
        console_node::exec(
            hsm_group,
            // cli_console,
            shasta_token,
            shasta_base_url,
            vault_base_url,
            vault_secret_path,
            vault_role_id,
            k8s_api_url,
            cli_console.get_one::<String>("XNAME").unwrap(),
        )
        .await; */
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
            if let Some(cli_migrate) = cli_migrate.subcommand_matches("backup") {
                let bos = cli_migrate.get_one::<String>("bos");
                let destination = cli_migrate.get_one::<String>("destination");
                let prehook = cli_migrate.get_one::<String>("pre-hook");
                let posthook = cli_migrate.get_one::<String>("post-hook");
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
            } else if let Some(cli_migrate) = cli_migrate.subcommand_matches("restore") {
                let bos_file = cli_migrate.get_one::<String>("bos-file");
                let cfs_file = cli_migrate.get_one::<String>("cfs-file");
                let hsm_file = cli_migrate.get_one::<String>("hsm-file");
                let ims_file = cli_migrate.get_one::<String>("ims-file");
                let image_dir = cli_migrate.get_one::<String>("image-dir");
                let prehook = cli_migrate.get_one::<String>("pre-hook");
                let posthook = cli_migrate.get_one::<String>("post-hook");
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
        } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
            let hsm_group_name_arg_opt = cli_delete.get_one::<String>("hsm-group"); // For now, we
                                                                                    // want to panic if this param is missing

            let target_hsm_group_vec = get_target_hsm_group_vec_or_all(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                hsm_group_name_arg_opt,
                settings_hsm_group_name_opt,
            )
            .await;

            let since_opt = if let Some(since) = cli_delete.get_one::<String>("since") {
                let date_time = chrono::NaiveDateTime::parse_from_str(
                    &(since.to_string() + "T00:00:00"),
                    "%Y-%m-%dT%H:%M:%S",
                )
                .unwrap();
                Some(date_time)
            } else {
                None
            };

            let until_opt = if let Some(until) = cli_delete.get_one::<String>("until") {
                let date_time = chrono::NaiveDateTime::parse_from_str(
                    &(until.to_string() + "T00:00:00"),
                    "%Y-%m-%dT%H:%M:%S",
                )
                .unwrap();
                Some(date_time)
            } else {
                None
            };

            let cfs_configuration_name_opt = cli_delete.get_one::<String>("configuration-name");

            let yes = cli_delete.get_one::<bool>("yes").unwrap_or(&false);

            let hsm_group_name_opt = if settings_hsm_group_name_opt.is_some() {
                settings_hsm_group_name_opt
            } else {
                cli_delete.get_one::<String>("hsm-group")
            };

            // INPUT VALIDATION - Check since date is prior until date
            if since_opt.is_some() && until_opt.is_some() && since_opt.unwrap() > until_opt.unwrap()
            {
                println!("since date can't be after until date. Exit");
                std::process::exit(1);
            }

            delete_data_related_cfs_configuration(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                hsm_group_name_opt,
                target_hsm_group_vec,
                cfs_configuration_name_opt,
                since_opt,
                until_opt,
                yes,
            )
            .await;
        }
    }

    Ok(())
}

/// Returns a list of HSM groups the user is expected to work with.
/// This method will exit if the user is asking for HSM group not allowed
/// If the user did not requested any HSM group, then it will return all HSM groups he has access
/// to
pub async fn get_target_hsm_group_vec_or_all(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cli_param_hsm_group: Option<&String>,
    config_file_or_env_hsm_group: Option<&String>,
) -> Vec<String> {
    let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

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
}

/// Returns a list of HSM groups the user is expected to work with or none (empty vec) if user is
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
}

/// Validate user has access to a list of HSM group members provided.
/// HSM members user is asking for are taken from cli command
/// Exit if user does not have access to any of the members provided. By not having access to a HSM
/// members means, the node bleongs to an HSM group which the user does not have access
pub async fn validate_target_hsm_members(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_members_opt: Vec<String>,
) -> Vec<String> {
    let hsm_groups_user_has_access = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    let all_xnames_user_has_access =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &hsm_groups_user_has_access,
        )
        .await;

    // Check user has access to all xnames he is requesting
    if hsm_group_members_opt
        .iter()
        .all(|hsm_member| all_xnames_user_has_access.contains(hsm_member))
    {
        hsm_group_members_opt
    } else {
        println!("Can't access all or any of the HSM members '{}'.\nPlease choose members form the list of HSM groups below:\n{}\nExit", hsm_group_members_opt.join(","), hsm_groups_user_has_access.join(","));
        std::process::exit(1);
    }
}
