use manta_backend_dispatcher::error::Error;
use csm_rs::node::types::NodeDetails;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::common::app_context::InfraContext;
use crate::common::authorization::get_groups_names_available;

/// Typed parameters for fetching cluster node details.
pub struct GetClusterParams {
  pub hsm_group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub status_filter: Option<String>,
}

/// Fetch node details for all nodes in the specified HSM groups.
pub async fn get_cluster_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetClusterParams,
) -> Result<Vec<NodeDetails>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.hsm_group_name.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await
  .map_err(|e| Error::Message(e.to_string()))?;

  let mut hsm_groups_node_list = infra.backend
    .get_member_vec_from_group_name_vec(token, &target_hsm_group_vec)
    .await?;

  hsm_groups_node_list.sort();

  let mut node_details_list = csm_rs::node::utils::get_node_details(
    token,
    infra.shasta_base_url,
    infra.shasta_root_cert,
    hsm_groups_node_list,
  )
  .await
  .map_err(|e: csm_rs::error::Error| Error::Message(e.to_string()))?;

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
