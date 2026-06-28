//! Node migration between HSM groups.
//!
//! `backup` and `restore` are thin service wrappers so handlers can
//! comply with the CLAUDE.md boundary rule (handlers → service →
//! backend, never handlers → backend directly). Only `migrate_nodes`
//! carries real orchestration (hosts-expression resolution,
//! HSM-group curation, per-pair member migration).
//!
//! Migration is a (target × parent) fan-out: for every requested
//! target group, every parent group the resolved xnames actually
//! belong to is paired with it and the backend
//! `migrate_group_members` call is issued. Nodes that don't live in
//! one of the named parents are silently skipped, which keeps the
//! call idempotent when a hosts expression is re-run.

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;
use manta_backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_members_access;
use crate::service::node_ops;

/// Result of migrating nodes for a single parent→target pair.
#[derive(serde::Serialize, utoipa::ToSchema)]
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
///
/// # Errors
///
/// - [`Error::BadRequest`] when the resolved xname list is empty, the
///   caller lacks group access to one of the resolved xnames, or a
///   dry-run would require creating a missing target group.
/// - [`Error::NotFound`] when a target group is missing and
///   `create_group` is false.
/// - [`Error::NetError`] / [`Error::CsmError`] from any of the
///   `get_group` / `migrate_group_members` backend calls.
pub async fn migrate_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  target_group_name_vec: &[String],
  parent_group_name_vec: &[String],
  hosts_expression: &str,
  dry_run: bool,
  create_group: bool,
) -> Result<(Vec<String>, Vec<NodeMigrationResult>), Error> {
  let xname_to_move_vec = node_ops::from_user_hosts_expression_to_xname_vec(
    infra,
    token,
    hosts_expression,
    false,
  )
  .await?;

  if xname_to_move_vec.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  // Defence in depth: the handler already validates every named
  // target/parent group, and the `retain` below filters out any
  // resolved xname that isn't in a parent group the caller can
  // reach — so the migration itself is bounded. We still gate on
  // member access here so the resolved `xname_to_move_vec` returned
  // in the response doesn't disclose nodes outside the caller's
  // groups (the resolver runs against full cluster metadata).
  validate_user_group_members_access(infra, token, &xname_to_move_vec).await?;

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
    if infra.backend.get_group(token, target_name).await.is_ok() {
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
      let xnames_ref: Vec<&str> = xnames.iter().map(String::as_str).collect();
      let (mut target_members, mut parent_members) = infra
        .backend
        .migrate_group_members(
          token,
          target_name,
          parent_group_name,
          &xnames_ref,
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

/// Export BOS session templates (and related artifacts) to backup files.
///
/// Thin forwarder; authorization (admin-only) and filesystem path
/// confinement are enforced by the caller before this function is
/// invoked.
pub async fn backup(
  infra: &InfraContext<'_>,
  token: &str,
  bos: Option<&str>,
  destination: Option<&str>,
) -> Result<(), Error> {
  infra.backend.migrate_backup(token, bos, destination).await
}

/// Restore BOS session templates and related artifacts from backup files.
///
/// Thin forwarder; authorization (admin-only) and filesystem path
/// confinement are enforced by the caller before this function is
/// invoked.
#[allow(clippy::too_many_arguments)]
pub async fn restore(
  infra: &InfraContext<'_>,
  token: &str,
  bos_file: Option<&str>,
  cfs_file: Option<&str>,
  hsm_file: Option<&str>,
  ims_file: Option<&str>,
  image_dir: Option<&str>,
  overwrite_group: bool,
  overwrite_configuration: bool,
  overwrite_image: bool,
  overwrite_template: bool,
) -> Result<(), Error> {
  infra
    .backend
    .migrate_restore(
      token,
      bos_file,
      cfs_file,
      hsm_file,
      ims_file,
      image_dir,
      overwrite_group,
      overwrite_configuration,
      overwrite_image,
      overwrite_template,
    )
    .await
}
