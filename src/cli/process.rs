use std::io::IsTerminal;

use clap::ArgMatches;
use comfy_table::Table;
use dialoguer::{theme::ColorfulTheme, Confirm};
use k8s_openapi::chrono;
use mesa::manta::{
    bos::template::get_image_id_from_bos_sessiontemplate_related_to_cfs_configuration,
    cfs::session::get_image_id_from_cfs_session_related_to_cfs_configuration,
};

use crate::{
    cli::commands::delete_data_related_to_cfs_configuration,
    common::node_ops::get_node_vec_booting_image,
};

use super::commands::{
    self, apply_cluster, apply_ephemeral_env, apply_image, apply_node_off, apply_node_on,
    apply_node_reset, apply_session, config_set, config_show,
    console_cfs_session_image_target_ansible, console_node, get_configuration, get_hsm, get_images,
    get_nodes, get_session, get_template, update_hsm_group, update_node,
};

pub async fn process_cli(
    cli_root: ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    gitea_token: &str,
    gitea_base_url: &str,
    hsm_group: Option<&String>,
    // base_image_id: &str,
    k8s_api_url: &str,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
    if let Some(cli_get) = cli_root.subcommand_matches("get") {
        if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration") {
            /*        let hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_configuration.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            }; */
            get_configuration::exec(
                gitea_base_url,
                gitea_token,
                shasta_token,
                shasta_base_url,
                cli_get_configuration.get_one::<String>("name"),
                // hsm_group_name,
                cli_get_configuration
                    .get_one::<bool>("most-recent")
                    .cloned(),
                cli_get_configuration.get_one::<u8>("limit"),
            )
            .await;
        } else if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
            let session_name = cli_get_session.get_one::<String>("name");

            let hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_session.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };

            let most_recent = cli_get_session.get_one::<bool>("most-recent");

            let limit_number = if let Some(true) = most_recent {
                Some(&1)
            } else if let Some(false) = most_recent {
                cli_get_session.get_one::<u8>("limit")
            } else {
                None
            };

            get_session::exec(
                shasta_token,
                shasta_base_url,
                hsm_group_name,
                session_name,
                limit_number,
                cli_get_session.get_one("output"),
            )
            .await;
        } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
            let hsm_group_name = match hsm_group {
                None => cli_get_template.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };
            get_template::exec(
                // hsm_group,
                shasta_token,
                shasta_base_url,
                hsm_group_name,
                cli_get_template.get_one::<String>("name"),
                cli_get_template.get_one::<bool>("most-recent").cloned(),
                cli_get_template.get_one::<u8>("limit"),
            )
            .await;
        } else if let Some(cli_get_node) = cli_get.subcommand_matches("nodes") {
            // Check HSM group name provided and configuration file
            let hsm_group_name = match hsm_group {
                None => cli_get_node.get_one::<String>("HSM_GROUP_NAME"),
                Some(_) => hsm_group,
            };
            get_nodes::exec(
                shasta_token,
                shasta_base_url,
                hsm_group_name,
                *cli_get_node
                    .get_one::<bool>("nids-only-one-line")
                    .unwrap_or(&false),
                *cli_get_node
                    .get_one::<bool>("xnames-only-one-line")
                    .unwrap_or(&false),
                cli_get_node.get_one::<String>("output"),
            )
            .await;
        } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
            let hsm_group_name = match hsm_group {
                None => cli_get_hsm_groups
                    .get_one::<String>("HSM_GROUP_NAME")
                    .unwrap(),
                Some(hsm_group_name_value) => hsm_group_name_value,
            };
            get_hsm::exec(shasta_token, shasta_base_url, hsm_group_name).await;
        } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
            let hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_images.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };
            get_images::exec(
                shasta_token,
                shasta_base_url,
                hsm_group_name,
                cli_get_images.get_one::<u8>("limit"),
            )
            .await;
        }
    } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
        /* if let Some(cli_apply_configuration) = cli_apply.subcommand_matches("configuration") {
            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
            apply_configuration::exec(
                cli_apply_configuration.get_one("file").unwrap(),
                shasta_token,
                shasta_base_url,
                &timestamp,
            )
            .await;
        } else  */
        if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
            let hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_apply_session.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };
            apply_session::exec(
                gitea_token,
                gitea_base_url,
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                shasta_token,
                shasta_base_url,
                k8s_api_url,
                cli_apply_session.get_one::<String>("name").cloned(),
                hsm_group_name,
                cli_apply_session
                    .get_many("repo-path")
                    .unwrap()
                    .cloned()
                    .collect(),
                cli_apply_session
                    .get_one::<String>("ansible-limit")
                    .cloned(),
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
                // base_image_id,
                cli_apply_image.get_one::<String>("ansible-verbosity"),
                cli_apply_image.get_one::<String>("ansible-passthrough"),
                cli_apply_image.get_one::<bool>("watch-logs"),
                &tag,
                hsm_group,
                k8s_api_url,
                cli_apply_image.get_one::<String>("output"),
            )
            .await;
        } else if let Some(cli_apply_cluster) = cli_apply.subcommand_matches("cluster") {
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
                cli_apply_cluster.get_one("file").unwrap(),
                // base_image_id,
                hsm_group,
                cli_apply_cluster.get_one::<String>("ansible-verbosity"),
                cli_apply_cluster.get_one::<String>("ansible-passthrough"),
                k8s_api_url,
                tag,
                cli_apply_cluster.get_one::<String>("output"),
            )
            .await;
        } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                apply_node_on::exec(
                    hsm_group,
                    shasta_token,
                    shasta_base_url,
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
                    hsm_group,
                    shasta_token,
                    shasta_base_url,
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
            } else if let Some(cli_apply_node_reset) = cli_apply_node.subcommand_matches("reset") {
                apply_node_reset::exec(
                    hsm_group,
                    shasta_token,
                    shasta_base_url,
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
            let hsm_group_name = if hsm_group.is_none() {
                cli_update_node.get_one::<String>("HSM_GROUP_NAME")
            } else {
                hsm_group
            };
            update_node::exec(
                shasta_token,
                shasta_base_url,
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
            let hsm_group_name = if hsm_group.is_none() {
                cli_update_hsm_group.get_one::<String>("HSM_GROUP_NAME")
            } else {
                hsm_group
            };
            update_hsm_group::exec(
                shasta_token,
                shasta_base_url,
                cli_update_hsm_group.get_one::<String>("boot-image"),
                cli_update_hsm_group.get_one::<String>("desired-configuration"),
                hsm_group_name.unwrap(),
            )
            .await;
        }
    } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
        commands::log::exec(
            // cli_log,
            shasta_token,
            shasta_base_url,
            vault_base_url,
            vault_secret_path,
            vault_role_id,
            k8s_api_url,
            None,
            cli_log.get_one::<String>("SESSION_NAME"),
            // cli_log.get_one::<u8>("layer-id"),
            hsm_group,
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
                hsm_group,
                // cli_console,
                shasta_token,
                shasta_base_url,
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

            console_cfs_session_image_target_ansible::exec(
                hsm_group,
                // cli_console,
                shasta_token,
                shasta_base_url,
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
    } else if let Some(cli_config) = cli_root.subcommand_matches("config") {
        if let Some(_cli_config_show) = cli_config.subcommand_matches("show") {
            config_show::exec(shasta_token, shasta_base_url).await;
        } else if let Some(cli_config_set) = cli_config.subcommand_matches("set") {
            if let Some(cli_config_set_hsm) = cli_config_set.subcommand_matches("hsm") {
                config_set::exec(
                    shasta_token,
                    shasta_base_url,
                    cli_config_set_hsm.get_one::<String>("HSM_GROUP_NAME"),
                )
                .await;
            }
        }
    } else if let Some(cli_delete) = cli_root.subcommand_matches("delete") {
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

        let hsm_group_name_opt = if hsm_group.is_some() {
            hsm_group
        } else {
            cli_delete.get_one::<String>("hsm-group")
        };

        // INPUT VALIDATION - Check since date is prior until date
        if since_opt.is_some() && until_opt.is_some() && since_opt.unwrap() > until_opt.unwrap() {
            println!("since date can't be after until date. Exit");
            std::process::exit(1);
        }

        // COLLECT SITE WIDE DATA FOR VALIDATION
        //

        // Check dessired configuration not using any CFS configuration to delete: Get all CFS components in CSM
        let cfs_components = mesa::shasta::cfs::component::http_client::get_multiple_components(
            shasta_token,
            shasta_base_url,
            None,
            None,
        )
        .await
        .unwrap();

        // Check images related to CFS configurations to delete are not used to boot nodes. For
        // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
        // Get all BSS boot params
        let boot_param_vec =
            mesa::shasta::bss::http_client::get_boot_params(shasta_token, shasta_base_url, &[])
                .await
                .unwrap();

        let mut cfs_configuration_value_vec = mesa::shasta::cfs::configuration::http_client::get(
            shasta_token,
            shasta_base_url,
            cfs_configuration_name_opt,
            None,
        )
        .await
        .unwrap();

        // Filter CFS configurations based on user input
        if since_opt.is_some() && until_opt.is_some() {
            cfs_configuration_value_vec.retain(|cfs_configuration_value| {
                let date = chrono::DateTime::parse_from_rfc3339(
                    cfs_configuration_value["lastUpdated"].as_str().unwrap(),
                )
                .unwrap()
                .naive_utc();

                since_opt.unwrap() <= date && date < until_opt.unwrap()
            });
        } else if cfs_configuration_name_opt.is_some() {
            cfs_configuration_value_vec.retain(|cfs_configuration_value| {
                cfs_configuration_value["name"]
                    .as_str()
                    .unwrap()
                    .eq_ignore_ascii_case(cfs_configuration_name_opt.unwrap())
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
        let mut bos_sessiontemplate_value_vec = mesa::shasta::bos::template::http_client::get(
            shasta_token,
            shasta_base_url,
            hsm_group_name_opt,
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
        let mut cfs_session_value_vec = mesa::shasta::cfs::session::http_client::get(
            shasta_token,
            shasta_base_url,
            hsm_group_name_opt,
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
            cfs_configuration_name_vec.contains(&cfs_configuration_value["name"].as_str().unwrap())
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
                hsm_group_name_opt,
                Some(image_id),
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
            println!("Nothing to delete. Exit");
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

        for bos_sessiontemplate_tuple in &bos_sessiontemplate_cfs_configuration_image_id_tuple_iter
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
            let mut nodes_using_cfs_configuration_as_dessired_configuration_vec = cfs_components
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
            &cfs_configuration_name_vec,
            &image_id_vec,
            // &cfs_components,
            &cfs_session_value_vec,
            &bos_sessiontemplate_value_vec,
            // &boot_param_vec,
        )
        .await;
    }

    Ok(())
}
