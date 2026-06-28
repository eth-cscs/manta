//! Cluster-scoped node detail queries using HSM group membership.
//!
//! Companion to [`crate::service::node`]: where `node` resolves an
//! arbitrary hosts expression to xnames, this module starts from one
//! or more HSM groups, expands them to xnames, and produces the same
//! [`NodeDetails`] rows.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_shared::types::dto::NodeDetails;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_vec_access;
use crate::service::node_details;
pub use manta_shared::types::api::cluster::GetClusterParams;

/// Fetch full node details for every member of the requested HSM
/// groups.
///
/// When `params.group_name` is unset the scope expands to every
/// group the token can access. The optional `status_filter` matches
/// case-insensitively against either the power or configuration
/// status. Results are sorted by xname for stable rendering.
///
/// # Errors
///
/// - [`Error::BadRequest`] when `params.group_name` names a group the
///   caller can't access.
/// - Backend errors from `get_group_available`,
///   `get_member_vec_from_group_name_vec`, or the per-xname detail
///   fetch in [`node_details::get_node_details`].
pub async fn get_cluster_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetClusterParams,
) -> Result<Vec<NodeDetails>, Error> {
  // Get list of target groups the user is asking for
  let target_group_vec: Vec<String> = if let Some(group) = &params.group_name {
    vec![group.clone()]
  } else {
    infra
      .backend
      .get_group_available(token)
      .await?
      .iter()
      .map(|group| group.label.clone())
      .collect()
  };

  // Validate groups and get list of groups available
  validate_user_group_vec_access(infra, token, &target_group_vec).await?;

  let mut group_vec_node_list = infra
    .backend
    .get_member_vec_from_group_name_vec(token, &target_group_vec)
    .await?;

  group_vec_node_list.sort();

  let mut node_details_list =
    node_details::get_node_details(infra, token, &group_vec_node_list).await?;

  // Apply status filter
  if let Some(ref status) = params.status_filter {
    node_details_list.retain(|nd| {
      nd.power_status.eq_ignore_ascii_case(status)
        || nd.configuration_status.eq_ignore_ascii_case(status)
    });
  }

  node_details_list.sort_by(|a, b| a.xname.cmp(&b.xname));

  Ok(node_details_list)
}
