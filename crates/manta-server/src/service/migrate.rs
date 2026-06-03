//! Node migration between HSM groups.
//!
//! vCluster backup/restore are 1:1 pass-throughs to `InfraContext`;
//! handlers call those directly. Only `migrate_nodes` carries real
//! orchestration (hosts-expression resolution, HSM-group curation,
//! per-pair member migration).

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;

use crate::server::common::app_context::InfraContext;
use crate::service::node_ops;

/// Result of migrating nodes for a single parent→target pair.
#[derive(serde::Serialize)]
pub struct NodeMigrationResult {
  /// HSM group that received the nodes.
  pub target_hsm_name: String,
  /// HSM group that the nodes were moved out of.
  pub parent_hsm_name: String,
  /// Final member list of the target group after migration.
  pub target_members: Vec<String>,
  /// Remaining member list of the parent group after migration.
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
  let xname_to_move_vec =
    node_ops::resolve_hosts_expression(infra, token, hosts_expression, false)
      .await?;

  if xname_to_move_vec.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  let mut hsm_group_summary: HashMap<String, Vec<String>> =
    node_ops::get_curated_hsm_group_from_xname_hostlist(
      infra,
      token,
      &xname_to_move_vec,
    )
    .await?;

  hsm_group_summary
    .retain(|hsm_name, _| parent_hsm_name_vec.contains(hsm_name));

  tracing::debug!("xnames to move: {:?}", xname_to_move_vec);

  let mut results = Vec::new();

  for target_hsm_name in target_hsm_name_vec {
    if infra.get_group(token, target_hsm_name).await.is_ok() {
      tracing::debug!("The group '{target_hsm_name}' exists, good.");
    } else if create_hsm_group {
      tracing::info!(
        "The group {} does not exist, it will be created",
        target_hsm_name
      );
      if dry_run {
        return Err(Error::BadRequest(format!(
          "Dry-run selected, the group '{target_hsm_name}' created"
        )));
      }
    } else {
      return Err(Error::NotFound(format!(
        "The group '{target_hsm_name}' does not exist and the option \
                 to create the group was not specified"
      )));
    }

    for (parent_hsm_name, xnames) in &hsm_group_summary {
      let (mut target_members, mut parent_members) = infra
        .migrate_group_members(
          token,
          target_hsm_name,
          parent_hsm_name,
          &xnames.iter().map(String::as_str).collect::<Vec<&str>>(),
          dry_run,
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
