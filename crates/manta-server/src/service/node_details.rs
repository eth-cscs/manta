//! Per-xname `NodeDetails` aggregation built from the backend
//! dispatcher.
//!
//! Replaces the direct `csm_rs::node::utils::get_node_details` call
//! that previously lived in `service/cluster.rs` and `service/node.rs`.
//! Going through the dispatcher's per-trait methods keeps the service
//! layer backend-agnostic — both CSM and OCHAMI implement the
//! underlying `CfsTrait` / `BootParametersTrait` / `ComponentTrait` /
//! `GroupTrait` calls used here, so the function works on either
//! site without a runtime branch.
//!
//! The flow is one round of five parallel fetches followed by an
//! in-memory join keyed by xname, intentionally trading marginally
//! more total bytes pulled (the CFS session list is unfiltered) for
//! O(1) per-xname lookup and no N+1 per-node HSM call. The
//! `csm_rs::node::utils::get_node_details` it replaces used a
//! semaphore-bounded `JoinSet` to fan out one HSM-membership call per
//! node; this version derives memberships from a single
//! `get_groups(None)` instead.

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_shared::types::dto::NodeDetails;

use crate::server::common::app_context::InfraContext;

/// Fallback string used when a backend field is absent. Matches the
/// historical csm-rs behavior so callers (CLI table renderer, status
/// summary) don't need to special-case.
const NOT_FOUND: &str = "Not found";

/// Return one [`NodeDetails`] per xname in `xnames`.
///
/// Xnames that are present in `xnames` but missing from any one of
/// the five backend responses still get a row; the affected fields
/// are filled with `"Not found"` so the per-row position in the
/// returned vector matches `xnames` after sorting.
///
/// The caller is expected to have already validated group access to
/// every xname; this helper does no authorization of its own.
pub async fn get_node_details(
  infra: &InfraContext<'_>,
  token: &str,
  xnames: &[String],
) -> Result<Vec<NodeDetails>, Error> {
  // CFS components endpoint takes a comma-separated id filter; build
  // it once. The other backends accept xname slices directly.
  let xname_filter = xnames.join(",");

  let (cfs_components, boot_params_vec, hsm_components, cfs_sessions, groups) = tokio::try_join!(
    infra.get_cfs_components(token, None, Some(&xname_filter), None),
    infra.get_bootparameters(token, xnames),
    infra.get_node_metadata_available(token),
    // Successful sessions only — we use them to resolve image id →
    // CFS configuration that built the image.
    infra.get_sessions(
      token, None, None, None, None, None, None, None, Some(true), None
    ),
    infra.get_groups(token, None),
  )?;

  // Build xname → comma-separated group label lookup once.
  let mut xname_to_groups: HashMap<String, Vec<String>> = HashMap::new();
  for group in &groups {
    if let Some(member_ids) = group.members.as_ref().and_then(|m| m.ids.as_ref())
    {
      for id in member_ids {
        xname_to_groups
          .entry(id.clone())
          .or_default()
          .push(group.label.clone());
      }
    }
  }

  // Index the per-xname lookups so the build loop below is O(N).
  let cfs_by_id: HashMap<&str, &_> = cfs_components
    .iter()
    .filter_map(|c| c.id.as_deref().map(|id| (id, c)))
    .collect();
  let hsm_by_id: HashMap<&str, &_> = hsm_components
    .iter()
    .filter_map(|c| c.id.as_deref().map(|id| (id, c)))
    .collect();

  // Image id → CFS configuration name that produced it.
  let image_to_cfs_config: HashMap<String, String> = cfs_sessions
    .iter()
    .filter_map(|session| {
      let result_id = session.get_first_result_id()?;
      let configuration_name = session.configuration.as_ref()?.name.as_ref()?;
      Some((result_id, configuration_name.clone()))
    })
    .collect();

  let mut out: Vec<NodeDetails> = xnames
    .iter()
    .map(|xname| {
      let hsm_info = hsm_by_id.get(xname.as_str());
      let nid = hsm_info
        .and_then(|c| c.nid)
        .map_or_else(|| NOT_FOUND.to_string(), |n| format!("nid{n:0>6}"));
      let power_status = hsm_info
        .and_then(|c| c.state.as_ref())
        .map_or_else(|| NOT_FOUND.to_string(), |s| s.to_uppercase());

      let cfs = cfs_by_id.get(xname.as_str());
      let desired_configuration = cfs
        .and_then(|c| c.desired_config.clone())
        .unwrap_or_else(|| NOT_FOUND.to_string());
      let configuration_status = cfs
        .and_then(|c| c.configuration_status.clone())
        .unwrap_or_else(|| NOT_FOUND.to_string());
      let enabled = cfs
        .and_then(|c| c.enabled)
        .map_or_else(|| NOT_FOUND.to_string(), |b| b.to_string());
      let error_count = cfs
        .and_then(|c| c.error_count)
        .map_or_else(|| NOT_FOUND.to_string(), |n| n.to_string());

      let boot_params = boot_params_vec
        .iter()
        .find(|bp| bp.hosts.iter().any(|h| h == xname));
      let (boot_image_id, kernel_params) = boot_params.map_or_else(
        || (NOT_FOUND.to_string(), NOT_FOUND.to_string()),
        |bp| {
          (
            bp.try_get_boot_image_id().unwrap_or_else(|| NOT_FOUND.to_string()),
            bp.params.clone(),
          )
        },
      );

      let boot_configuration = image_to_cfs_config
        .get(&boot_image_id)
        .cloned()
        .unwrap_or_else(|| NOT_FOUND.to_string());

      let hsm = xname_to_groups
        .get(xname)
        .map(|labels| labels.join(", "))
        .unwrap_or_default();

      NodeDetails {
        xname: xname.clone(),
        nid,
        hsm,
        power_status,
        desired_configuration,
        configuration_status,
        enabled,
        error_count,
        boot_image_id,
        boot_configuration,
        kernel_params,
      }
    })
    .collect();

  out.sort_by(|a, b| a.xname.cmp(&b.xname));

  Ok(out)
}
