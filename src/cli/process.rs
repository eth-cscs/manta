use clap::ArgMatches;
use k8s_openapi::chrono;

use super::commands::{
    apply_cluster, apply_image, apply_node_off, apply_node_on, apply_node_reset, apply_session,
    console, get_configuration, get_hsm, get_images, get_nodes, get_session, get_template, log,
    update_hsm_group, update_node,
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
                None => cli_get_node.get_one::<String>("HSMGROUP"),
                Some(_) => hsm_group,
            };
            get_nodes::exec(
                shasta_token,
                shasta_base_url,
                hsm_group_name,
                *cli_get_node.get_one::<bool>("silent").unwrap_or(&false),
                *cli_get_node
                    .get_one::<bool>("silent-xname")
                    .unwrap_or(&false),
            )
            .await;
        } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
            let hsm_group_name = match hsm_group {
                None => cli_get_hsm_groups.get_one::<String>("HSMGROUP").unwrap(),
                Some(hsm_group_name_value) => hsm_group_name_value,
            };
            get_hsm::exec(shasta_token, shasta_base_url, hsm_group_name).await;
        } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
            let hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_images.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };
            get_images::exec(shasta_token, shasta_base_url, hsm_group_name).await;
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
                    .unwrap()
                    .to_string(),
                cli_apply_session
                    .get_one::<String>("ansible-passthrough")
                    .unwrap()
                    .to_string(),
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
                cli_apply_image.get_one::<bool>("watch-logs"),
                &tag,
                hsm_group,
                k8s_api_url,
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
                k8s_api_url,
                tag,
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
        }
    } else if let Some(cli_update) = cli_root.subcommand_matches("update") {
        if let Some(cli_update_node) = cli_update.subcommand_matches("nodes") {
            let hsm_group_name = if hsm_group.is_none() {
                cli_update_node.get_one::<String>("HSM_GROUP")
            } else {
                hsm_group
            };
            update_node::exec(
                shasta_token,
                shasta_base_url,
                hsm_group_name.unwrap(),
                cli_update_node
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .split(',')
                    .map(|xname| xname.trim())
                    .collect(),
                cli_update_node.get_one::<String>("CFS_CONFIG"),
            )
            .await;
        } else if let Some(cli_update_hsm_group) = cli_update.subcommand_matches("hsm-group") {
            update_hsm_group::exec(
                shasta_token,
                shasta_base_url,
                cli_update_hsm_group,
                hsm_group,
            )
            .await;
        }
    } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
        log::exec(
            // cli_log,
            shasta_token,
            shasta_base_url,
            vault_base_url,
            vault_secret_path,
            vault_role_id,
            k8s_api_url,
            None,
            cli_log.get_one::<String>("SESSION"),
            cli_log.get_one::<u8>("layer-id"),
            hsm_group,
        )
        .await;
    } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
        console::exec(
            hsm_group,
            // cli_console,
            shasta_token,
            shasta_base_url,
            vault_base_url,
            vault_secret_path,
            vault_role_id,
            k8s_api_url,
            cli_console
                .get_one::<String>("XNAME")
                .unwrap()
                .split(',')
                .map(|xname| xname.trim())
                .collect(),
        )
        .await;
    }

    Ok(())
}
