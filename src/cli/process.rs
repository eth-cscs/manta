use std::io::IsTerminal;

use clap::ArgMatches;
use config::Config;
use k8s_openapi::chrono;
use mesa::shasta::{authentication, hsm::utils::get_member_vec_from_hsm_name_vec};

use crate::{
    cli::commands::delete_data_related_to_cfs_configuration,
    common::node_ops::get_node_vec_booting_image,
};

use super::commands::{
    self, apply_cluster, apply_configuration, apply_ephemeral_env, apply_image, apply_node_off,
    apply_node_on, apply_node_reset, apply_session, config_set_hsm, config_set_log,
    config_set_site, config_show, config_unset_auth, config_unset_hsm,
    console_cfs_session_image_target_ansible, console_node,
    delete_data_related_to_cfs_configuration::delete_data_related_cfs_configuration,
    get_configuration, get_hsm, get_images, get_nodes, get_session, get_template, migrate_backup,
    migrate_restore, update_hsm_group, update_node,
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
    if let Some(cli_config) = cli_root.subcommand_matches("config") {
        if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
            let shasta_token = &authentication::get_api_token(
                shasta_base_url,
                shasta_root_cert,
                keycloak_base_url,
            )
            .await?;

            config_show::exec(shasta_token, shasta_base_url, shasta_root_cert, settings).await;
        } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
            if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
                let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
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
            if let Some(cli_config_set_site) = cli_config_set.subcommand_matches("site") {
                config_set_site::exec(cli_config_set_site.get_one::<String>("SITE_NAME")).await;
            }
            if let Some(cli_config_set_log) = cli_config_set.subcommand_matches("log") {
                config_set_log::exec(cli_config_set_log.get_one::<String>("LOG_LEVEL")).await;
            }
        } else if let Some(cli_config_unset) = cli_config.subcommand_matches("unset") {
            if let Some(_cli_config_unset_hsm) = cli_config_unset.subcommand_matches("hsm") {
                let shasta_token = &authentication::get_api_token(
                    shasta_base_url,
                    shasta_root_cert,
                    keycloak_base_url,
                )
                .await?;

                config_unset_hsm::exec(shasta_token).await;
            } else if let Some(_cli_config_unset_auth) = cli_config_unset.subcommand_matches("auth")
            {
                config_unset_auth::exec().await;
            }
        }
    } else {
        let shasta_token =
            &authentication::get_api_token(shasta_base_url, shasta_root_cert, keycloak_base_url)
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

        if let Some(cli_get) = cli_root.subcommand_matches("get") {
            if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration") {
                let hsm_group_name_arg_opt = cli_get_configuration.get_one::<String>("hsm-group");

                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };

                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
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
                    &hsm_group_target_vec,
                    limit,
                    cli_get_configuration.get_one("output"),
                )
                .await;
            } else if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
                let hsm_group_name_arg_opt = cli_get_session.get_one::<String>("hsm-group");
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                // let session_name = cli_get_session.get_one::<String>("name");

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
                    &hsm_group_target_vec,
                    cli_get_session.get_one::<String>("name"),
                    limit,
                    cli_get_session.get_one("output"),
                )
                .await;
            } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
                let hsm_group_name_arg_opt = cli_get_template.get_one::<String>("hsm-group");
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                let hsm_member_vec = get_member_vec_from_hsm_name_vec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_group_target_vec,
                )
                .await;

                get_template::exec(
                    // hsm_group,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_group_target_vec,
                    &hsm_member_vec,
                    cli_get_template.get_one::<String>("name"),
                    cli_get_template.get_one::<u8>("limit"),
                )
                .await;
            } else if let Some(cli_get_node) = cli_get.subcommand_matches("cluster") {
                let hsm_group_name_arg_opt = cli_get_node.get_one::<String>("HSM_GROUP_NAME");
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_group_target_vec,
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
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_nodes::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_group_target_vec,
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
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_hsm::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_target_vec.first().unwrap(),
                )
                .await;
            } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
                let hsm_group_name_arg_opt = cli_get_images.get_one::<String>("hsm-group");
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                get_images::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_group_target_vec,
                    cli_get_images.get_one::<u8>("limit"),
                )
                .await;
            }
        } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
            if let Some(cli_apply_configuration) = cli_apply.subcommand_matches("configuration") {
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    settings_hsm_group_name_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                let tag = if let Some(input_tag) = cli_apply_configuration.get_one::<String>("tag")
                {
                    input_tag.clone()
                } else {
                    chrono::Utc::now().format("%Y%m%d%H%M%S").to_string()
                };

                let _ = apply_configuration::exec(
                    cli_apply_configuration.get_one("file").unwrap(),
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &tag,
                    cli_apply_configuration.get_one::<String>("output"),
                )
                .await;
            } else if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
                let hsm_group_name_arg_opt = cli_apply_session.get_one::<String>("hsm-group");
                let target_hsm_group_name =
                    if let Some(hsm_group_name) = settings_hsm_group_name_opt {
                        Some(hsm_group_name)
                    } else {
                        hsm_group_name_arg_opt
                    };
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    settings_hsm_group_name_opt,
                )
                .await;

                let hsm_group_members = validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_apply_session
                        .get_one::<String>("ansible-limit")
                        .cloned()
                        .unwrap_or_default()
                        .split(',')
                        .map(|xname| xname.trim().to_string())
                        .collect::<Vec<String>>(),
                )
                .await;

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
                    hsm_group_target_vec.first(),
                    cli_apply_session
                        .get_many("repo-path")
                        .unwrap()
                        .cloned()
                        .collect(),
                    Some(hsm_group_members.join(",")),
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
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    settings_hsm_group_name_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                let tag = if let Some(input_tag) = cli_apply_image.get_one::<String>("tag") {
                    input_tag.clone()
                } else {
                    chrono::Utc::now().format("%Y%m%d%H%M%S").to_string()
                };

                apply_image::exec(
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    cli_apply_image.get_one("file").unwrap(),
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    // base_image_id,
                    cli_apply_image.get_one::<String>("ansible-verbosity"),
                    cli_apply_image.get_one::<String>("ansible-passthrough"),
                    cli_apply_image.get_one::<bool>("watch-logs"),
                    &tag,
                    &hsm_group_target_vec,
                    k8s_api_url,
                    cli_apply_image.get_one::<String>("output"),
                )
                .await;
            } else if let Some(cli_apply_cluster) = cli_apply.subcommand_matches("cluster") {
                let hsm_group_target_vec = validate_target_hsm_name(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    settings_hsm_group_name_opt,
                    settings_hsm_group_name_opt,
                )
                .await;

                let tag = if let Some(input_tag) = cli_apply_cluster.get_one::<String>("tag") {
                    input_tag.clone()
                } else {
                    chrono::Utc::now().format("%Y%m%d%H%M%S").to_string()
                };

                apply_cluster::exec(
                    vault_base_url,
                    vault_secret_path,
                    vault_role_id,
                    // cli_apply_cluster,
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_apply_cluster.get_one("file").unwrap(),
                    // base_image_id,
                    settings_hsm_group_name_opt,
                    &hsm_group_target_vec,
                    cli_apply_cluster.get_one::<String>("ansible-verbosity"),
                    cli_apply_cluster.get_one::<String>("ansible-passthrough"),
                    k8s_api_url,
                    cli_apply_cluster.get_one::<bool>("watch-logs"),
                    tag,
                    cli_apply_cluster.get_one::<String>("output"),
                )
                .await;
            } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
                if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                    apply_node_on::exec(
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
                    .await;
                } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
                    apply_node_off::exec(
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
                    .await;
                } else if let Some(cli_apply_node_reset) =
                    cli_apply_node.subcommand_matches("reset")
                {
                    apply_node_reset::exec(
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
                let hsm_group_name = if settings_hsm_group_name_opt.is_none() {
                    cli_update_node.get_one::<String>("HSM_GROUP_NAME")
                } else {
                    settings_hsm_group_name_opt
                };
                update_node::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_name,
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
                let hsm_group_name = if settings_hsm_group_name_opt.is_none() {
                    cli_update_hsm_group.get_one::<String>("HSM_GROUP_NAME")
                } else {
                    settings_hsm_group_name_opt
                };
                update_hsm_group::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    cli_update_hsm_group.get_one::<String>("boot-image"),
                    cli_update_hsm_group.get_one::<String>("desired-configuration"),
                    hsm_group_name.unwrap(),
                )
                .await;
            }
        } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
            let hsm_group_target_vec = validate_target_hsm_name(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cli_log.try_get_one::<String>("hsm-group").unwrap_or(None),
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
                &hsm_group_target_vec,
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

                let hsm_name_available_vec = if let Some(hsm_name) = settings_hsm_group_name_opt {
                    [hsm_name.clone()].to_vec()
                } else {
                    config_show::get_hsm_name_available_from_jwt_or_all(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                    )
                    .await
                };

                console_cfs_session_image_target_ansible::exec(
                    &hsm_name_available_vec,
                    // cli_console,
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
                migrate_backup::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    bos,
                    destination,
                )
                .await;
            } else if let Some(cli_migrate) = cli_migrate.subcommand_matches("restore") {
                log::info!(">>> MIGRATE RESTORE not implemented yet")
            }
        } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
            let hsm_name_available_vec = if let Some(hsm_name) = settings_hsm_group_name_opt {
                [hsm_name.clone()].to_vec()
            } else if let Some(hsm_name) = cli_delete.get_one::<String>("hsm-group") {
                [hsm_name.clone()].to_vec()
            } else {
                config_show::get_hsm_name_available_from_jwt_or_all(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                )
                .await
            };

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

            let force = cli_delete.get_one::<bool>("force").unwrap_or(&false);

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
                hsm_name_available_vec,
                cfs_configuration_name_opt,
                since_opt,
                until_opt,
                force,
            )
            .await;
        }
    }

    Ok(())
}

/// Validate user has access to a list of HSM group names provided.
/// HSM groups user is asking for are taken from either config file or env or cli command
/// Returns a curated list of HSM groups user is asking.
pub async fn validate_target_hsm_name(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_config_opt: Option<&String>,
    hsm_group_name_opt: Option<&String>,
) -> Vec<String> {
    let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    let hsm_group_target_opt = if let Some(hsm_name) = hsm_group_name_opt {
        Some(hsm_name)
    } else if let Some(hsm_name) = hsm_group_config_opt {
        Some(hsm_name)
    } else {
        None
    };

    let hsm_group_target_vec: Vec<String> = if let Some(hsm_group_target) = hsm_group_target_opt {
        hsm_name_available_vec
            .clone()
            .into_iter()
            .filter(|hsm_name_aux| hsm_name_aux.eq(hsm_group_target))
            .collect::<Vec<String>>()
    } else {
        hsm_name_available_vec.clone()
    };

    if hsm_group_target_vec.is_empty() {
        println!(
            "Target HSM group '{}' not available, plese chose one of {:?}. Exit",
            hsm_group_target_opt.unwrap(),
            hsm_name_available_vec
        );
        std::process::exit(1);
    }

    hsm_group_target_vec
}

/// Validate user has access to a list of HSM group members provided.
/// HSM members user is asking for are taken from cli command
/// Returns a cureated list of HSM members user is asking.
pub async fn validate_target_hsm_members(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_members_opt: Vec<String>,
) -> Vec<String> {
    let hsm_name_available_vec = config_show::get_hsm_name_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    let hsm_member_vec_available_vec = get_member_vec_from_hsm_name_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &hsm_name_available_vec,
    )
    .await;

    if hsm_group_members_opt
        .iter()
        .all(|hsm_member| hsm_member_vec_available_vec.contains(hsm_member))
    {
        hsm_group_members_opt
    } else {
        println!("HSM members '{:?}' invalid. Exit", hsm_group_members_opt,);
        std::process::exit(1);
    }
}
