//! Implements the `manta migrate nodes` command.

use std::collections::HashMap;

use anyhow::Error;
use nodeset::NodeSet;

use crate::cli::http_client::MantaClient;
use crate::common::{app_context::AppContext, audit};
use crate::service::migrate;

/// Move nodes between HSM groups with validation.
pub async fn exec(
    ctx: &AppContext<'_>,
    token: &str,
    target_hsm_name_vec: &[String],
    parent_hsm_name_vec: &[String],
    hosts_expression: &str,
    dry_run: bool,
    create_hsm_group: bool,
) -> Result<(), Error> {
    if let Some(server_url) = ctx.infra.manta_server_url {
        let result = MantaClient::new(server_url, ctx.infra.site_name)?
            .migrate_nodes(token, target_hsm_name_vec, parent_hsm_name_vec, hosts_expression, dry_run, create_hsm_group)
            .await?;
        if dry_run {
            println!("dry-run enabled, changes not persisted.");
        }
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());

        audit::maybe_send_audit(
            ctx.cli.kafka_audit_opt,
            token,
            format!("Migrate nodes from {:?} to {:?}", parent_hsm_name_vec, target_hsm_name_vec),
            None,
            Some(serde_json::json!(vec![parent_hsm_name_vec, target_hsm_name_vec])),
        )
        .await;

        return Ok(());
    }

    let (xname_to_move_vec, results) = migrate::migrate_nodes(
        &ctx.infra,
        token,
        target_hsm_name_vec,
        parent_hsm_name_vec,
        hosts_expression,
        dry_run,
        create_hsm_group,
    )
    .await?;

    // Display results
    let mut group_map: HashMap<String, Vec<String>> = HashMap::new();
    for result in &results {
        group_map.entry(result.target_hsm_name.clone()).and_modify(|members| members.extend(result.target_members.clone())).or_insert(result.target_members.clone());
        group_map.entry(result.parent_hsm_name.clone()).and_modify(|members| members.extend(result.parent_members.clone())).or_insert(result.parent_members.clone());
    }

    if dry_run {
        println!("dry-run enabled, changes not persisted.")
    }
    for (group_name, mut group_members) in group_map {
        group_members.sort();
        let group_members_nodeset: NodeSet =
        group_members.join(", ").parse().unwrap_or_default();

        println!(
            "Group '{}' members: {:?}",
            group_name, group_members_nodeset.to_string()
        );
    }

    // Audit
    audit::maybe_send_audit(
        ctx.cli.kafka_audit_opt,
        token,
        format!(
            "Migrate nodes from {:?} to {:?}",
            parent_hsm_name_vec, target_hsm_name_vec
        ),
        Some(serde_json::json!(xname_to_move_vec)),
        Some(serde_json::json!(vec![
            parent_hsm_name_vec,
            target_hsm_name_vec
        ])),
    )
    .await;

    Ok(())
}
