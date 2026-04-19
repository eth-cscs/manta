use std::collections::HashMap;

use anyhow::{Error, bail};
use manta_backend_dispatcher::interfaces::{
    hsm::group::GroupTrait, migrate_backup::MigrateBackupTrait,
    migrate_restore::MigrateRestoreTrait,
};

use crate::common::{app_context::InfraContext, node_ops};

/// Execute a migrate-backup operation against the backend.
pub async fn migrate_backup(
    infra: &InfraContext<'_>,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
) -> Result<(), Error> {
    infra
        .backend
        .migrate_backup(
            token,
            infra.shasta_base_url,
            infra.shasta_root_cert,
            bos,
            destination,
        )
        .await?;
    Ok(())
}

/// Execute a migrate-restore operation against the backend.
#[allow(clippy::too_many_arguments)]
pub async fn migrate_restore(
    infra: &InfraContext<'_>,
    token: &str,
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite: bool,
) -> Result<(), Error> {
    infra
        .backend
        .migrate_restore(
            token,
            infra.shasta_base_url,
            infra.shasta_root_cert,
            bos_file,
            cfs_file,
            hsm_file,
            ims_file,
            image_dir,
            overwrite,
            overwrite,
            overwrite,
            overwrite,
        )
        .await?;
    Ok(())
}

/// Result of migrating nodes for a single parent→target pair.
pub struct NodeMigrationResult {
    pub target_hsm_name: String,
    pub parent_hsm_name: String,
    pub target_members: Vec<String>,
    pub parent_members: Vec<String>,
}

/// Resolve hosts expression, curate HSM groups, validate targets,
/// and migrate nodes between HSM groups.
///
/// Returns the list of xnames that were moved and the per-pair
/// migration results for display.
pub async fn migrate_nodes(
    infra: &InfraContext<'_>,
    token: &str,
    target_hsm_name_vec: &[String],
    parent_hsm_name_vec: &[String],
    hosts_expression: &str,
    dry_run: bool,
    create_hsm_group: bool,
) -> Result<(Vec<String>, Vec<NodeMigrationResult>), Error> {
    let backend = infra.backend;

    // Resolve hosts expression to xnames
    let xname_to_move_vec =
        node_ops::resolve_hosts_expression(backend, token, hosts_expression, false)
            .await?;

    if xname_to_move_vec.is_empty() {
        bail!("The list of nodes to operate is empty. Nothing to do");
    }

    // Get curated HSM groups filtered to parent groups
    let mut hsm_group_summary: HashMap<String, Vec<String>> =
        node_ops::get_curated_hsm_group_from_xname_hostlist(
            backend,
            token,
            &xname_to_move_vec,
        )
        .await?;

    hsm_group_summary
        .retain(|hsm_name, _| parent_hsm_name_vec.contains(hsm_name));

    log::debug!("xnames to move: {:?}", xname_to_move_vec);

    let mut results = Vec::new();

    for target_hsm_name in target_hsm_name_vec {
        if backend.get_group(token, target_hsm_name).await.is_ok() {
            log::debug!("The HSM group {} exists, good.", target_hsm_name);
        } else if create_hsm_group {
            log::info!(
                "HSM group {} does not exist, it will be created",
                target_hsm_name
            );
            if dry_run {
                bail!(
                    "Dry-run selected, cannot create the new group continue.",
                );
            }
        } else {
            bail!(
                "HSM group {} does not exist, but the option \
                 to create the group was NOT specified, cannot continue.",
                target_hsm_name
            );
        }

        for (parent_hsm_name, xnames) in &hsm_group_summary {
            let (mut target_members, mut parent_members) = backend
                .migrate_group_members(
                    token,
                    target_hsm_name,
                    parent_hsm_name,
                    &xnames.iter().map(String::as_str).collect::<Vec<&str>>(),
                )
                .await?;

            target_members.sort();
            parent_members.sort();

            results.push(NodeMigrationResult {
                target_hsm_name: target_hsm_name.clone(),
                parent_hsm_name: parent_hsm_name.clone(),
                target_members,
                parent_members,
            });
        }
    }

    Ok((xname_to_move_vec, results))
}
