//! HSM group CRUD operations and membership management.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::{
  component::ComponentTrait, group::GroupTrait,
};
use manta_backend_dispatcher::types::Group;

use crate::server::common::{app_context::InfraContext, jwt_ops};
use crate::service::authorization::{
  validate_group_vec_access, validate_user_group_members_access,
};
use crate::service::node_ops::{self, from_hosts_expression_to_xname_vec};
pub use manta_shared::types::api::group::GetGroupParams;

/// Resolve the caller's accessible groups (`Vec<Group>`) and the
/// target-label vector in a single backend round-trip.
///
/// Three places in the service layer repeated the same three-call
/// dance:
///
/// 1. `get_group_available` to derive labels (used when no settings
///    group is supplied).
/// 2. `validate_user_group_vec_access` which itself called
///    `get_group_name_available` to verify the labels.
/// 3. A second `get_group_available` inside a `try_join!` to fetch the
///    full `Vec<Group>` needed by the downstream filter.
///
/// All three steps want the same data; the helper folds them into one
/// `get_group_available` call plus an in-memory check. Non-admin
/// callers still get the same access-validation guarantee (they're
/// rejected with `BadRequest` if `settings_group_name_opt` names a
/// group they can't see); admin tokens short-circuit, matching the
/// behaviour of [`crate::service::authorization::validate_user_group_vec_access`].
pub async fn resolve_target_and_available_groups(
  infra: &InfraContext<'_>,
  token: &str,
  settings_group_name_opt: Option<&str>,
) -> Result<(Vec<Group>, Vec<String>), Error> {
  let group_available_vec = infra.backend.get_group_available(token).await?;

  let target_group_vec: Vec<String> = match settings_group_name_opt {
    Some(label) => {
      if !jwt_ops::is_user_admin(token) {
        let available_labels: Vec<String> = group_available_vec
          .iter()
          .map(|g| g.label.clone())
          .collect();
        validate_group_vec_access(
          std::slice::from_ref(&label.to_string()),
          &available_labels,
        )?;
      }
      vec![label.to_string()]
    }
    None => group_available_vec
      .iter()
      .map(|g| g.label.clone())
      .collect(),
  };

  Ok((group_available_vec, target_group_vec))
}

/// List HSM groups visible to the caller.
///
/// When `params.group_name` is set the lookup is scoped to that
/// single label; otherwise it spans every group the token already
/// grants access to. Group access is re-validated before the backend
/// call so the response can't leak labels the caller couldn't have
/// listed directly.
pub async fn get_groups(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetGroupParams,
) -> Result<Vec<Group>, Error> {
  // Single backend fetch + in-memory access validation replaces
  // three sequential round-trips (label derivation, validation,
  // backend fetch). See [`resolve_target_and_available_groups`].
  let (_group_available_vec, target_group_vec) =
    resolve_target_and_available_groups(
      infra,
      token,
      params.group_name.as_deref(),
    )
    .await?;

  infra.backend.get_groups(token, Some(&target_group_vec)).await
}

/// Check that deleting `label` would not leave any node without a
/// group.
///
/// An xname is "orphaned" if `label` is its only HSM group. When at
/// least one such node exists, returns
/// `Error::Conflict` listing the orphans so the operator can decide
/// whether to move them first or pass `force` to
/// [`delete_group`].
pub async fn validate_group_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  label: &str,
) -> Result<(), Error> {
  let xname_vec = infra
    .backend
    .get_member_vec_from_group_name_vec(token, &[label.to_string()])
    .await?;

  let xname_vec_ref: Vec<&str> =
    xname_vec.iter().map(String::as_str).collect();
  let mut xname_map = infra
    .backend
    .get_group_map_and_filter_by_group_vec(token, &xname_vec_ref)
    .await?;

  xname_map.retain(|_xname, group_name_vec| {
    group_name_vec.len() == 1
      && group_name_vec.first().is_some_and(|name| name == label)
  });

  let mut members_orphan_if_group_deleted: Vec<String> =
    xname_map.into_keys().collect();
  members_orphan_if_group_deleted.sort();

  if !members_orphan_if_group_deleted.is_empty() {
    return Err(Error::Conflict(format!(
      "The hosts below will become orphan if group '{}' gets deleted: {}",
      label,
      members_orphan_if_group_deleted.join(", ")
    )));
  }

  Ok(())
}

/// Delete the HSM group named `label`.
///
/// Unless `force` is set, [`validate_group_deletion`] runs first and
/// the delete is rejected if any node would be orphaned.
pub async fn delete_group(
  infra: &InfraContext<'_>,
  token: &str,
  label: &str,
  force: bool,
) -> Result<(), Error> {
  if !force {
    validate_group_deletion(infra, token, label).await?;
  }
  infra.backend.delete_group(token, label).await.map(|_| ())
}

/// Create the HSM group described by `group`.
///
/// The backend rejects duplicate labels; manta does no pre-check
/// beyond the standard authorization layer applied by the handler.
pub async fn create_group(
  infra: &InfraContext<'_>,
  token: &str,
  group: Group,
) -> Result<(), Error> {
  infra.backend.add_group(token, group).await.map(|_| ())
}

/// Resolve `host_expression` and remove the resolved nodes from
/// `group_name`.
///
/// With `dry_run = true`, only the resolution runs — no backend
/// mutation. Errors from the per-node deletion abort the loop and
/// surface to the handler, so a partially completed batch is
/// possible.
pub async fn delete_group_members(
  infra: &InfraContext<'_>,
  token: &str,
  group_name: &str,
  host_expression: &str,
  dry_run: bool,
) -> Result<(), Error> {
  let node_metadata_available_vec =
    infra.backend.get_node_metadata_available(token).await?;

  let xname_vec = from_hosts_expression_to_xname_vec(
    host_expression,
    false,
    &node_metadata_available_vec,
  )?;

  validate_user_group_members_access(infra, token, &xname_vec).await?;

  if xname_vec.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  // Defence in depth: callers can only remove nodes from groups they
  // have access to (handler already gates on `group_name`), but a
  // hosts_expression resolved over the full cluster could name xnames
  // outside the caller's reach. The downstream backend call would
  // already no-op on non-members, but rejecting here gives the user
  // an explicit error and keeps `add_nodes_to_group` / this function
  // symmetric.
  validate_user_group_members_access(infra, token, &xname_vec).await?;

  for xname in &xname_vec {
    if dry_run {
      tracing::info!(
        "Dryrun enabled: no changes persisted into the system.\nGroup member '{}' removed from group '{}'",
        xname,
        group_name
      );
    } else {
      infra
        .backend
        .delete_member_from_group(token, group_name, xname)
        .await?;
    }
  }

  Ok(())
}

/// Resolve `hosts_expression` and add the resulting nodes to the
/// existing HSM group `target_hsm_name`.
///
/// The target group must already exist (an explicit `NotFound` is
/// returned rather than the backend's opaque error). An empty
/// resolution is rejected with `BadRequest`. Returns the resolved
/// xnames alongside the group's sorted, post-update membership.
pub async fn add_nodes_to_group(
  infra: &InfraContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
) -> Result<(Vec<String>, Vec<String>), Error> {
  let xname_to_move_vec = node_ops::from_user_hosts_expression_to_xname_vec(
    infra,
    token,
    hosts_expression,
    false,
  )
  .await?;

  validate_user_group_members_access(infra, token, &xname_to_move_vec).await?;

  if xname_to_move_vec.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to move is empty. Nothing to do".to_string(),
    ));
  }

  if infra.backend.get_group(token, target_hsm_name).await.is_err() {
    return Err(Error::NotFound(format!(
      "Target HSM group '{target_hsm_name}' does not exist"
    )));
  }

  let xnames_to_move: Vec<&str> =
    xname_to_move_vec.iter().map(String::as_str).collect();

  let mut updated_members = infra
    .backend
    .add_members_to_group(token, target_hsm_name, &xnames_to_move)
    .await?;

  updated_members.sort();

  Ok((xname_to_move_vec, updated_members))
}
