use clap::ArgMatches;
use k8s_openapi::chrono;


use super::commands::{get_configuration, get_session, get_template, get_nodes, get_hsm, get_images, apply_configuration, apply_session, apply_image, apply_cluster, apply_node_on, apply_node_off, apply_node_reset, update_node, update_hsm_group, console, log};

pub async fn process_cli(
    cli_root: ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    vault_base_url: &str,
    vault_role_id: &str,
    gitea_token: &str,
    gitea_base_url: &str,
    hsm_group: Option<&String>,
    base_image_id: &str,
    k8s_api_url: &str,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
    if let Some(cli_get) = cli_root.subcommand_matches("get") {
        if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration") {
            get_configuration::exec(
                gitea_token,
                hsm_group,
                cli_get_configuration,
                shasta_token,
                shasta_base_url,
            )
            .await;
        } else if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
            get_session::exec(hsm_group, cli_get_session, shasta_token, shasta_base_url).await;
        } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
            get_template::exec(hsm_group, cli_get_template, shasta_token, shasta_base_url).await;
        } else if let Some(cli_get_node) = cli_get.subcommand_matches("nodes") {
            get_nodes::exec(hsm_group, cli_get_node, shasta_token, shasta_base_url).await;
        } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
            get_hsm::exec(hsm_group, cli_get_hsm_groups, shasta_token, shasta_base_url).await;
        } else if let Some(cli_get_images) = cli_get.subcommand_matches("images") {
            get_images::exec(
                shasta_token,
                shasta_base_url,
                cli_get_images,
                None,
                None,
                hsm_group,
            )
            .await;
        }
    } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
        if let Some(cli_apply_configuration) = cli_apply.subcommand_matches("configuration") {
            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
            apply_configuration::exec(
                cli_apply_configuration,
                shasta_token,
                shasta_base_url,
                &timestamp,
            )
            .await;
        } else if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
            apply_session::exec(
                gitea_token,
                gitea_base_url,
                vault_base_url,
                vault_role_id,
                hsm_group,
                cli_apply_session,
                shasta_token,
                shasta_base_url,
                k8s_api_url,
            )
            .await;
        } else if let Some(cli_apply_image) = cli_apply.subcommand_matches("image") {
            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
            apply_image::exec(
                vault_base_url,
                vault_role_id,
                cli_apply_image,
                shasta_token,
                shasta_base_url,
                base_image_id,
                cli_apply_image.get_one::<bool>("watch-logs"),
                &timestamp,
                // hsm_group,
                k8s_api_url,
            )
            .await;
        } else if let Some(cli_apply_cluster) = cli_apply.subcommand_matches("cluster") {
            apply_cluster::exec(
                vault_base_url,
                vault_role_id,
                cli_apply_cluster,
                shasta_token,
                shasta_base_url,
                base_image_id,
                hsm_group,
                k8s_api_url,
            )
            .await;
        } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                apply_node_on::exec(hsm_group, cli_apply_node_on, shasta_token, shasta_base_url)
                    .await;
            } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
                apply_node_off::exec(hsm_group, cli_apply_node_off, shasta_token, shasta_base_url)
                    .await;
            } else if let Some(cli_apply_node_reset) = cli_apply_node.subcommand_matches("reset") {
                apply_node_reset::exec(
                    hsm_group,
                    cli_apply_node_reset,
                    shasta_token,
                    shasta_base_url,
                )
                .await;
            }
        }
    } else if let Some(cli_update) = cli_root.subcommand_matches("update") {
        if let Some(cli_update_node) = cli_update.subcommand_matches("nodes") {
            update_node::exec(
                shasta_token,
                shasta_base_url,
                cli_update_node,
                cli_update_node
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .split(',')
                    .map(|xname| xname.trim())
                    .collect(),
                cli_update_node.get_one::<String>("CFS_CONFIG"),
                hsm_group,
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
            cli_log,
            shasta_token,
            shasta_base_url,
            vault_base_url,
            vault_role_id,
            k8s_api_url,
        )
        .await;
    } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
        console::exec(
            hsm_group,
            cli_console,
            shasta_token,
            shasta_base_url,
            vault_base_url,
            vault_role_id,
            k8s_api_url,
        )
        .await;
    }

    Ok(())
}
