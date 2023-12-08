use std::io::IsTerminal;

use clap::ArgMatches;
use comfy_table::Table;
use config::Config;
use dialoguer::{theme::ColorfulTheme, Confirm};
use k8s_openapi::chrono;
use mesa::{
    manta::{
        bos::template::get_image_id_from_bos_sessiontemplate_related_to_cfs_configuration,
        cfs::session::get_image_id_from_cfs_session_related_to_cfs_configuration,
    },
    shasta::{authentication, hsm::utils::get_member_vec_from_hsm_name_vec},
};

use crate::{
    cli::commands::delete_data_related_to_cfs_configuration,
    common::node_ops::get_node_vec_booting_image,
};

use super::commands::{
    self, apply_cluster, apply_configuration, apply_ephemeral_env, apply_image, apply_node_off,
    apply_node_on, apply_node_reset, apply_session, config_set_hsm, config_set_log,
    config_set_site, config_show, config_unset_auth, config_unset_hsm,
    console_cfs_session_image_target_ansible, console_node, get_configuration, get_hsm, get_images,
    get_nodes, get_session, get_template, migrate_backup, migrate_restore, update_hsm_group,
    update_node,
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

                let limit: Option<&u8> = if let Some(true) = cli_get_session.get_one("most-recent") {
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

            // COLLECT SITE WIDE DATA FOR VALIDATION
            //

            // Check dessired configuration not using any CFS configuration to delete: Get all CFS components in CSM
            let cfs_components =
                mesa::shasta::cfs::component::http_client::get_multiple_components(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    None,
                )
                .await
                .unwrap();

            // Check images related to CFS configurations to delete are not used to boot nodes. For
            // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
            // Get all BSS boot params
            let boot_param_vec = mesa::shasta::bss::http_client::get_boot_params(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &[],
            )
            .await
            .unwrap();

            let mut cfs_configuration_value_vec =
                mesa::shasta::cfs::configuration::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    None,
                    cfs_configuration_name_opt,
                    None,
                )
                .await
                .unwrap();

            // Filter CFS configurations based on user input
            if let (Some(since), Some(until)) = (since_opt, until_opt) {
                cfs_configuration_value_vec.retain(|cfs_configuration_value| {
                    let date = chrono::DateTime::parse_from_rfc3339(
                        cfs_configuration_value["lastUpdated"].as_str().unwrap(),
                    )
                    .unwrap()
                    .naive_utc();

                    since <= date && date < until
                });
            } else if let Some(cfs_configuration_name) = cfs_configuration_name_opt {
                cfs_configuration_value_vec.retain(|cfs_configuration_value| {
                    cfs_configuration_value["name"]
                        .as_str()
                        .unwrap()
                        .eq_ignore_ascii_case(cfs_configuration_name)
                });
            }

            // Get list CFS configuration names
            let mut cfs_configuration_name_vec = cfs_configuration_value_vec
                .iter()
                .map(|configuration_value| configuration_value["name"].as_str().unwrap())
                .collect::<Vec<&str>>();

            // Check images related to CFS configurations to delete are not used to boot nodes. For
            // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
            // Get all BOS session templates
            let mut bos_sessiontemplate_value_vec =
                mesa::shasta::bos::template::http_client::filter(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_name_available_vec,
                    None,
                    None,
                )
                .await
                .unwrap();

            // TODO: change to iter so we can later on get its image ids without having to copy memory
            // to create new Vec
            bos_sessiontemplate_value_vec.retain(|bos_sessiontemplate_value| {
                cfs_configuration_name_vec.contains(
                    &bos_sessiontemplate_value
                        .pointer("/cfs/configuration")
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )
            });

            let cfs_configuration_name_from_bos_sessiontemplate_value_iter =
                bos_sessiontemplate_value_vec
                    .iter()
                    .map(|bos_sessiontemplate_value| {
                        bos_sessiontemplate_value
                            .pointer("/cfs/configuration")
                            .unwrap()
                            .as_str()
                            .unwrap()
                    });

            // Check images related to CFS configurations to delete are not used to boot nodes. For
            // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
            // Get all CFS sessions
            let mut cfs_session_value_vec = mesa::shasta::cfs::session::http_client::filter(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &hsm_name_available_vec,
                None,
                None,
                None,
            )
            .await
            .unwrap();

            // TODO: change to iter so we can later on get its image ids without having to copy memory
            // to create new Vec
            cfs_session_value_vec.retain(|cfs_session_value| {
                cfs_configuration_name_vec.contains(
                    &cfs_session_value
                        .pointer("/configuration/name")
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )
            });

            let cfs_configuration_name_from_cfs_sessions =
                cfs_session_value_vec.iter().map(|cfs_session_value| {
                    cfs_session_value
                        .pointer("/configuration/name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                });

            // Get list of CFS configuration names related to CFS sessions and BOS sessiontemplates
            cfs_configuration_name_vec = cfs_configuration_name_from_bos_sessiontemplate_value_iter
                .chain(cfs_configuration_name_from_cfs_sessions)
                .collect::<Vec<&str>>();
            cfs_configuration_name_vec.sort();
            cfs_configuration_name_vec.dedup();

            // Get final list of CFS configuration serde values related to CFS sessions and BOS
            // sessiontemplates
            cfs_configuration_value_vec.retain(|cfs_configuration_value| {
                cfs_configuration_name_vec
                    .contains(&cfs_configuration_value["name"].as_str().unwrap())
            });

            // Get image ids from CFS sessions and BOS sessiontemplate related to CFS configuration to delete
            let image_id_from_cfs_session_vec =
                get_image_id_from_cfs_session_related_to_cfs_configuration(&cfs_session_value_vec);

            // Get image ids from BOS session template related to CFS configuration to delete
            let image_id_from_bos_sessiontemplate_vec =
                get_image_id_from_bos_sessiontemplate_related_to_cfs_configuration(
                    &bos_sessiontemplate_value_vec,
                );

            // Combine image ids from CFS session and BOS session template
            let mut image_id_related_from_cfs_session_bos_sessiontemplate_vec = [
                image_id_from_cfs_session_vec,
                image_id_from_bos_sessiontemplate_vec,
            ]
            .concat();

            image_id_related_from_cfs_session_bos_sessiontemplate_vec.sort();
            image_id_related_from_cfs_session_bos_sessiontemplate_vec.dedup();

            // Filter list of image ids by removing the ones that does not exists. This is because we
            // currently image id list contains the values from CFS session and BOS sessiontemplate
            // which does not means the image still exists (the image perse could have been deleted
            // previously and the CFS session and BOS sessiontemplate not being cleared)
            let mut image_id_vec = Vec::new();
            for image_id in &image_id_related_from_cfs_session_bos_sessiontemplate_vec {
                if mesa::shasta::ims::image::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &hsm_name_available_vec,
                    Some(image_id),
                    None,
                    None,
                )
                .await
                .is_ok()
                {
                    log::info!("Image ID {} exists", image_id);
                    image_id_vec.push(image_id.to_string());
                }
            }

            log::info!(
                "Image id related to CFS sessions and/or BOS sessiontemplate: {:?}",
                image_id_related_from_cfs_session_bos_sessiontemplate_vec
            );
            log::info!("Image ids to delete: {:?}", image_id_vec);

            // Get list of CFS session name, CFS configuration name and image id
            let cfs_session_cfs_configuration_image_id_tuple_iter =
                cfs_session_value_vec.iter().map(|cfs_session_value| {
                    (
                        cfs_session_value["name"].as_str().unwrap(),
                        cfs_session_value
                            .pointer("/configuration/name")
                            .unwrap()
                            .as_str()
                            .unwrap(),
                        cfs_session_value
                            .pointer("/status/artifacts/0/result_id")
                            .and_then(|result_id| result_id.as_str())
                            .unwrap_or(""),
                    )
                });

            // Get list of BOS sessiontemplate name, CFS configuration name and image ids for compute nodes
            let bos_sessiontemplate_cfs_configuration_compute_image_id_tuple_iter =
                bos_sessiontemplate_value_vec
                    .iter()
                    .map(|bos_sessiontemplate_value| {
                        let cfs_session_name = bos_sessiontemplate_value["name"].as_str().unwrap();
                        let cfs_configuration_name = bos_sessiontemplate_value
                            .pointer("/cfs/configuration")
                            .unwrap()
                            .as_str()
                            .unwrap();
                        let image_id = if let Some(image_path_value) =
                            bos_sessiontemplate_value.pointer("/boot_sets/compute/path")
                        {
                            image_path_value
                                .as_str()
                                .unwrap()
                                .strip_prefix("s3://boot-images/")
                                .unwrap()
                                .strip_suffix("/manifest.json")
                                .unwrap()
                        } else {
                            ""
                        };
                        (cfs_session_name, cfs_configuration_name, image_id)
                    });

            // Get list of BOS sessiontemplate name, CFS configuration name and image ids for uan nodes
            let bos_sessiontemplate_cfs_configuration_uan_image_id_tuple_iter =
                bos_sessiontemplate_value_vec
                    .iter()
                    .map(|bos_sessiontemplate_value| {
                        let bos_sessiontemplate_name =
                            bos_sessiontemplate_value["name"].as_str().unwrap();
                        let cfs_configuration_name = bos_sessiontemplate_value
                            .pointer("/cfs/configuration")
                            .unwrap()
                            .as_str()
                            .unwrap();
                        let image_id = if let Some(image_path_value) =
                            bos_sessiontemplate_value.pointer("/boot_sets/uan/path")
                        {
                            image_path_value
                                .as_str()
                                .unwrap()
                                .strip_prefix("s3://boot-images/")
                                .unwrap()
                                .strip_suffix("/manifest.json")
                                .unwrap()
                        } else {
                            ""
                        };
                        (bos_sessiontemplate_name, cfs_configuration_name, image_id)
                    });

            // Get final list of CFS configurations to delete. NOTE this list won't include CFS configurations with neither BOS sessiontemplate nor CFS session related, the reason is must filter data to delete by HSM group and CFS configurations by default are not related to any HSM group
            let bos_sessiontemplate_cfs_configuration_image_id_tuple_iter =
                bos_sessiontemplate_cfs_configuration_compute_image_id_tuple_iter
                    .chain(bos_sessiontemplate_cfs_configuration_uan_image_id_tuple_iter)
                    .collect::<Vec<(&str, &str, &str)>>();

            // EVALUATE IF NEED TO CONTINUE. EXIT IF THERE IS NO DATA TO DELETE
            //
            if cfs_configuration_name_vec.is_empty()
                && image_id_vec.is_empty()
                && cfs_session_value_vec.is_empty()
                && bos_sessiontemplate_value_vec.is_empty()
            {
                print!("Nothing to delete.");
                if cfs_configuration_name_opt.is_some() {
                    print!(
                        " Could not find information related to CFS configuration '{}'",
                        cfs_configuration_name_opt.unwrap()
                    );
                }
                if since_opt.is_some() && until_opt.is_some() {
                    print!(
                        " Could not find information between dates {} and {}",
                        since_opt.unwrap(),
                        until_opt.unwrap()
                    );
                }
                print!(" in HSM '{}'. Exit", hsm_group_name_opt.unwrap());

                std::process::exit(0);
            }

            // PRINT SUMMARY/DATA TO DELETE
            //
            println!("CFS sessions to delete:");

            let mut cfs_session_table = Table::new();

            cfs_session_table.set_header(vec!["Name", "Configuration", "Image ID"]);

            for cfs_session_tuple in cfs_session_cfs_configuration_image_id_tuple_iter {
                cfs_session_table.add_row(vec![
                    cfs_session_tuple.0,
                    cfs_session_tuple.1,
                    cfs_session_tuple.2,
                ]);
            }

            println!("{cfs_session_table}");

            println!("BOS sessiontemplates to delete:");

            let mut bos_sessiontemplate_table = Table::new();

            bos_sessiontemplate_table.set_header(vec!["Name", "Configuration", "Image ID"]);

            for bos_sessiontemplate_tuple in
                &bos_sessiontemplate_cfs_configuration_image_id_tuple_iter
            {
                bos_sessiontemplate_table.add_row(vec![
                    bos_sessiontemplate_tuple.0,
                    bos_sessiontemplate_tuple.1,
                    bos_sessiontemplate_tuple.2,
                ]);
            }

            println!("{bos_sessiontemplate_table}");

            println!("CFS configurations to delete:");

            let mut cfs_configuration_table = Table::new();

            cfs_configuration_table.set_header(vec!["Name", "Last Update"]);

            for cfs_configuration_value in &cfs_configuration_value_vec {
                cfs_configuration_table.add_row(vec![
                    cfs_configuration_value["name"].as_str().unwrap(),
                    cfs_configuration_value["lastUpdated"].as_str().unwrap(),
                ]);
            }

            println!("{cfs_configuration_table}");

            println!("Images to delete:");

            let mut image_id_table = Table::new();

            image_id_table.set_header(vec!["Image ID"]);

            for image_id in &image_id_vec {
                image_id_table.add_row(vec![image_id]);
            }

            println!("{image_id_table}");

            // VALIDATION
            //
            // Process CFS configurations to delete one by one
            for cfs_configuration_name in &cfs_configuration_name_vec {
                // Check dessired configuration not using any CFS configuration to delete
                let mut nodes_using_cfs_configuration_as_dessired_configuration_vec =
                    cfs_components
                        .iter()
                        .filter(|cfs_component| {
                            cfs_component["desiredConfig"]
                                .as_str()
                                .unwrap()
                                .eq(*cfs_configuration_name)
                        })
                        .map(|cfs_component| cfs_component["id"].as_str().unwrap())
                        .collect::<Vec<&str>>();

                nodes_using_cfs_configuration_as_dessired_configuration_vec.sort();

                if !nodes_using_cfs_configuration_as_dessired_configuration_vec.is_empty() {
                    eprintln!(
                    "CFS configuration {} can't be deleted. Reason:\nCFS configuration {} used as desired configuration for nodes: {}",
                    cfs_configuration_name, cfs_configuration_name, nodes_using_cfs_configuration_as_dessired_configuration_vec.join(", ")
                );
                    std::process::exit(1);
                }
            }

            for cfs_configuration_name in &cfs_configuration_name_vec {
                // Check images related to CFS configurations to delete are not used to boot nodes. For
                // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff

                // Check images related to CFS configurations to delete are not used to boot nodes. For
                // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
                let mut boot_image_node_vec = Vec::new();

                for image_id in &image_id_vec {
                    let nodes = get_node_vec_booting_image(image_id, &boot_param_vec);

                    if !nodes.is_empty() {
                        boot_image_node_vec.push((image_id, nodes));
                    }
                }

                if !boot_image_node_vec.is_empty() {
                    eprintln!(
                        "Image based on CFS configuration {} can't be deleted. Reason:",
                        cfs_configuration_name
                    );
                    for (image_id, node_vec) in boot_image_node_vec {
                        eprintln!("Image id {} used to boot nodes:\n{:?}", image_id, node_vec);
                    }
                    std::process::exit(1);
                }
            }

            // ASK USER FOR CONFIRMATION
            //
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Please revew the data above and confirm to delete:")
                .interact()
                .unwrap()
            {
                println!("Continue");
            } else {
                println!("Cancelled by user. Aborting.");
                std::process::exit(0);
            }

            // DELETE DATA
            //
            delete_data_related_to_cfs_configuration::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &cfs_configuration_name_vec,
                &image_id_vec,
                // &cfs_components,
                &cfs_session_value_vec,
                &bos_sessiontemplate_value_vec,
                // &boot_param_vec,
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
