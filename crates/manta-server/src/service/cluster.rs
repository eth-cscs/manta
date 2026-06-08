//! Cluster-scoped node detail queries using HSM group membership.

use csm_rs::node::types::NodeDetails;
use manta_backend_dispatcher::error::Error;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_vec_access;
pub use manta_shared::types::params::cluster::GetClusterParams;

/// Fetch node details for all nodes in the specified HSM groups.
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
      .get_group_available(token)
      .await?
      .iter()
      .map(|group| group.label.clone())
      .collect()
  };

  // Validate groups and get list of groups available
  validate_user_group_vec_access(infra, token, &target_group_vec).await?;

  let mut hsm_groups_node_list = infra
    .get_member_vec_from_group_name_vec(token, &target_group_vec)
    .await?;

  hsm_groups_node_list.sort();

  let mut node_details_list = csm_rs::node::utils::get_node_details(
    token,
    infra.shasta_base_url,
    infra.shasta_root_cert,
    infra.socks5_proxy,
    hsm_groups_node_list,
  )
  .await
  .map_err(|e: csm_rs::error::Error| -> Error { e.into() })?;

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
