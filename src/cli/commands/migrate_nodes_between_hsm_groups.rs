use anyhow::Error;

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
    for result in &results {
        println!(
            "HSM '{}' members: {:?}",
            result.target_hsm_name, result.target_members
        );
        println!(
            "HSM '{}' members: {:?}",
            result.parent_hsm_name, result.parent_members
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
