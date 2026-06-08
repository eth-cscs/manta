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

/// Move the nodes resolved from `hosts_expression` out of any group
/// in `parent_name_vec` and into every group in
/// `target_group_name_vec`.
///
/// The xname set is curated through
/// [`node_ops::get_curated_group_from_xname_hostlist`] and then
/// filtered to the requested parents — nodes that don't currently
/// belong to one of those parents are silently skipped, which keeps
/// the call idempotent when the user passes the same expression
/// twice. Each `target_name` is required to exist unless
/// `create_group` is true (in dry-run mode the missing-group case
/// is reported as a `BadRequest` so the operator sees what would have
/// been created). Returns the moved xnames and one
/// [`NodeMigrationResult`] per (target, parent) pair, with both
/// membership lists sorted for stable rendering.
pub async fn migrate_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  target_group_name_vec: &[String],
  parent_group_name_vec: &[String],
  hosts_expression: &str,
  dry_run: bool,
  create_group: bool,
) -> Result<(Vec<String>, Vec<NodeMigrationResult>), Error> {
  let xname_to_move_vec =
    node_ops::resolve_hosts_expression(infra, token, hosts_expression, false)
      .await?;

  if xname_to_move_vec.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  let mut group_summary: HashMap<String, Vec<String>> =
    node_ops::get_curated_group_from_xname_hostlist(
      infra,
      token,
      &xname_to_move_vec,
    )
    .await?;

  group_summary.retain(|hsm_name, _| parent_group_name_vec.contains(hsm_name));

  tracing::debug!("xnames to move: {:?}", xname_to_move_vec);

  let mut results = Vec::new();

  for target_name in target_group_name_vec {
    if infra.get_group(token, target_name).await.is_ok() {
      tracing::debug!("The group '{target_name}' exists, good.");
    } else if create_group {
      tracing::info!(
        "The group {} does not exist, it will be created",
        target_name
      );
      if dry_run {
        return Err(Error::BadRequest(format!(
          "Dry-run selected, the group '{target_name}' created"
        )));
      }
    } else {
      return Err(Error::NotFound(format!(
        "The group '{target_name}' does not exist and the option \
                 to create the group was not specified"
      )));
    }

    for (parent_group_name, xnames) in &group_summary {
      let (mut target_members, mut parent_members) = infra
        .migrate_group_members(
          token,
          target_name,
          parent_group_name,
          &xnames.iter().map(String::as_str).collect::<Vec<&str>>(),
          dry_run,
        )
        .await?;

      target_members.sort();
      parent_members.sort();

      results.push(NodeMigrationResult {
        target_hsm_name: target_name.clone(),
        parent_hsm_name: parent_group_name.clone(),
        target_members,
        parent_members,
      });
    }
  }

  Ok((xname_to_move_vec, results))
}
