//! Hardware inventory queries for individual nodes and clusters.
//!
//! Both query shapes (cluster-by-group, nodes-by-expression) fan out
//! one per-xname `get_inventory_hardware_query` call concurrently,
//! rate-limited by a Tokio semaphore at the
//! `HW_INVENTORY_CONCURRENCY_LIMIT` constant defined below. Failed
//! per-node fetches are logged and replaced by an empty [`NodeSummary`]
//! so the response vector lines up with the input xname list.
//!
//! The internal aggregation helper
//! `calculate_group_hw_component_summary` lives in
//! `manta_shared::types::cluster_status`; it's only re-imported here
//! to back the test module.

use std::sync::Arc;
use std::time::Instant;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::{
  component::ComponentTrait, hardware_inventory::HardwareInventory,
};
use manta_backend_dispatcher::types::NodeSummary;
use tokio::sync::Semaphore;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::{
  validate_user_group_members_access, validate_user_group_vec_access,
};
use crate::service::node_ops::from_hosts_expression_to_xname_vec;
pub use manta_shared::types::api::hardware::{
  GetHardwareClusterParams, GetHardwareNodesListParams,
};

/// Maximum number of concurrent hardware inventory requests.
const HW_INVENTORY_CONCURRENCY_LIMIT: usize = 15;

// ── Hardware Cluster ──

/// Result of a hardware cluster query.
pub struct HardwareClusterResult {
  /// Resolved HSM group name the inventory was collected for (may
  /// differ from the requested name if the caller's authorization
  /// only permitted a subset).
  pub hsm_group_name: String,
  /// Per-node hardware summaries, one entry per group member.
  pub node_summaries: Vec<NodeSummary>,
}

/// Fetch hardware inventory for a slice of xnames concurrently,
/// rate-limited by a semaphore. Shared by cluster and nodes-list queries.
async fn fetch_node_summaries(
  infra: &InfraContext<'_>,
  token: &str,
  xnames: &[String],
) -> Vec<NodeSummary> {
  let mut tasks = tokio::task::JoinSet::new();
  let sem = Arc::new(Semaphore::new(HW_INVENTORY_CONCURRENCY_LIMIT));

  let n = xnames.len();
  let width = n.checked_ilog10().unwrap_or(0) as usize + 1;

  for (i, xname) in xnames.iter().enumerate() {
    tracing::info!(
      "\rGetting hw components for node '{xname}' [{:>width$}/{n}]",
      i + 1
    );

    let backend_cp = infra.backend_clone();
    let token_str = token.to_string();
    let xname_str = xname.clone();
    let permit = Arc::clone(&sem).acquire_owned().await;

    tasks.spawn(async move {
      let _permit = permit;
      let hw_inventory_typed = backend_cp
        .get_inventory_hardware_query(
          &token_str, &xname_str, None, None, None, None, None,
        )
        .await;

      // `NodeSummary::from_csm_value` still parses out of a JSON Value;
      // re-serialize the typed `HWInventory` and pluck `/Nodes/0` like
      // before. A future cleanup can replace this round-trip with a
      // typed constructor that takes `&HWInventory` directly.
      let node_hw_opt = match hw_inventory_typed {
        Ok(hw_inv) => serde_json::to_value(&hw_inv)
          .ok()
          .and_then(|v| v.pointer("/Nodes/0").cloned()),
        Err(e) => {
          tracing::error!(
            "Failed to get HW inventory for '{}': {}",
            xname_str,
            e
          );
          None
        }
      };

      match node_hw_opt {
        Some(v) => NodeSummary::from_csm_value(v),
        None => NodeSummary {
          xname: xname_str,
          ..Default::default()
        },
      }
    });
  }

  let mut summaries = Vec::with_capacity(n);
  while let Some(res) = tasks.join_next().await {
    match res {
      Ok(s) => summaries.push(s),
      Err(e) => {
        tracing::error!("Failed fetching node hardware information: {}", e);
      }
    }
  }
  summaries
}

/// Fetch hardware inventory for every member of an HSM group.
///
/// When `params.group_name` is unset, the first group the caller has
/// access to is used and surfaced back through
/// `HardwareClusterResult::hsm_group_name`. Per-node inventory
/// queries run concurrently, capped by `HW_INVENTORY_CONCURRENCY_LIMIT`.
/// Empty groups are logged but not treated as an error.
///
/// # Errors
///
/// - [`Error::BadRequest`] when `params.group_name` is unreachable
///   for the caller.
/// - [`Error::NotFound`] when the caller has no accessible groups
///   and no `params.group_name` was supplied.
/// - [`Error::NetError`] / [`Error::CsmError`] from
///   `get_group_available` / `get_group`. Per-node inventory failures
///   degrade to an empty `NodeSummary` row rather than surfacing an
///   error.
pub async fn get_hardware_cluster(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetHardwareClusterParams,
) -> Result<HardwareClusterResult, Error> {
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

  let hsm_group_name = target_group_vec
    .first()
    .ok_or_else(|| {
      Error::NotFound("No HSM groups available for this user".to_string())
    })?
    .clone();

  let hsm_group = infra.backend.get_group(token, &hsm_group_name).await?;

  let members = hsm_group
    .members
    .unwrap_or_default()
    .ids
    .unwrap_or_default();

  if members.is_empty() {
    tracing::warn!("HSM group '{}' has no members", hsm_group.label);
  }

  tracing::debug!(
    "Get HW artifacts for nodes in HSM group '{}' and members {:?}",
    hsm_group.label,
    members
  );

  let start_total = Instant::now();
  let node_summaries = fetch_node_summaries(infra, token, &members).await;
  tracing::info!(
    "Time elapsed getting hw inventory for HSM '{}': {:?}",
    hsm_group_name,
    start_total.elapsed()
  );

  Ok(HardwareClusterResult {
    hsm_group_name,
    node_summaries,
  })
}

// ── Hardware Nodes List ──

/// Result of a hardware nodes-list query.
pub struct HardwareNodesListResult {
  /// Per-node hardware summaries, one entry per resolved xname.
  pub node_summaries: Vec<NodeSummary>,
}

/// Fetch hardware inventory for the nodes named by
/// `params.host_expression`.
///
/// The expression is parsed by [`from_hosts_expression_to_xname_vec`]
/// (hostlist notation, NIDs, or xnames; siblings are not expanded
/// here). An empty resolution yields `BadRequest` rather than a
/// silent no-op. The caller's group access to every resolved xname is
/// validated through [`validate_user_group_members_access`] before
/// the per-node inventory fan-out runs.
///
/// # Errors
///
/// - [`Error::InvalidNodeId`] / [`Error::BadRequest`] when the
///   expression cannot be parsed or resolves to an empty xname set.
/// - [`Error::BadRequest`] when the caller lacks group access to one
///   of the resolved xnames.
/// - [`Error::NetError`] / [`Error::CsmError`] from
///   `get_node_metadata_available`. Per-node inventory failures
///   degrade to an empty `NodeSummary` row rather than surfacing an
///   error.
pub async fn get_hardware_nodes_list(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetHardwareNodesListParams,
) -> Result<HardwareNodesListResult, Error> {
  let node_metadata_available_vec =
    infra.backend.get_node_metadata_available(token).await?;

  let node_list = from_hosts_expression_to_xname_vec(
    &params.host_expression,
    false,
    &node_metadata_available_vec,
  )?;

  if node_list.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  // Validate xnames
  validate_user_group_members_access(infra, token, &node_list).await?;

  let node_summaries = fetch_node_summaries(infra, token, &node_list).await;
  Ok(HardwareNodesListResult { node_summaries })
}

// `calculate_group_hw_component_summary` and `get_cluster_hw_pattern` moved
// to `manta_shared::types::cluster_status`. Only
// `calculate_group_hw_component_summary` is still needed locally — the
// tests below use it.
#[cfg(test)]
use manta_shared::types::cluster_status::calculate_group_hw_component_summary;

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::{
    ArtifactSummary, ArtifactType, NodeSummary,
  };

  /// Helper: create an ArtifactSummary with the given info string.
  fn make_artifact(
    art_type: ArtifactType,
    info: Option<&str>,
  ) -> ArtifactSummary {
    ArtifactSummary {
      xname: "x0".to_string(),
      r#type: art_type,
      info: info.map(String::from),
    }
  }

  #[test]
  fn summary_counts_processors_and_accels() {
    let nodes = vec![NodeSummary {
      xname: "x1000c0s0b0n0".to_string(),
      processors: vec![
        make_artifact(ArtifactType::Processor, Some("AMD EPYC 7742")),
        make_artifact(ArtifactType::Processor, Some("AMD EPYC 7742")),
      ],
      node_accels: vec![make_artifact(
        ArtifactType::NodeAccel,
        Some("NVIDIA A100"),
      )],
      memory: vec![],
      node_hsn_nics: vec![],
      ..Default::default()
    }];
    let summary = calculate_group_hw_component_summary(&nodes);
    assert_eq!(summary.get("AMD EPYC 7742"), Some(&2));
    assert_eq!(summary.get("NVIDIA A100"), Some(&1));
  }

  #[test]
  fn summary_converts_memory_mib_to_gib() {
    let nodes = vec![NodeSummary {
      xname: "x1000c0s0b0n0".to_string(),
      processors: vec![],
      node_accels: vec![],
      memory: vec![
        ArtifactSummary {
          xname: "x0".to_string(),
          r#type: ArtifactType::Memory,
          info: Some("16384 MiB".to_string()),
        },
        ArtifactSummary {
          xname: "x0".to_string(),
          r#type: ArtifactType::Memory,
          info: Some("16384 MiB".to_string()),
        },
      ],
      node_hsn_nics: vec![],
      ..Default::default()
    }];
    let summary = calculate_group_hw_component_summary(&nodes);
    assert_eq!(summary.get("Memory (GiB)"), Some(&32));
  }

  #[test]
  fn summary_aggregates_across_multiple_nodes() {
    let nodes = vec![
      NodeSummary {
        xname: "n1".to_string(),
        processors: vec![make_artifact(
          ArtifactType::Processor,
          Some("AMD EPYC 7742"),
        )],
        ..Default::default()
      },
      NodeSummary {
        xname: "n2".to_string(),
        processors: vec![
          make_artifact(ArtifactType::Processor, Some("AMD EPYC 7742")),
          make_artifact(ArtifactType::Processor, Some("Intel Xeon Gold")),
        ],
        ..Default::default()
      },
    ];
    let summary = calculate_group_hw_component_summary(&nodes);
    assert_eq!(summary.get("AMD EPYC 7742"), Some(&2));
    assert_eq!(summary.get("Intel Xeon Gold"), Some(&1));
  }

  #[test]
  fn summary_empty_nodes() {
    let nodes: Vec<NodeSummary> = vec![];
    let summary = calculate_group_hw_component_summary(&nodes);
    assert!(summary.is_empty());
  }

  #[test]
  fn summary_skips_none_info_in_processors() {
    let nodes = vec![NodeSummary {
      xname: "n1".to_string(),
      processors: vec![
        make_artifact(ArtifactType::Processor, None),
        make_artifact(ArtifactType::Processor, Some("AMD EPYC 7742")),
      ],
      ..Default::default()
    }];
    let summary = calculate_group_hw_component_summary(&nodes);
    assert_eq!(summary.get("AMD EPYC 7742"), Some(&1));
    assert_eq!(summary.len(), 1);
  }
}
