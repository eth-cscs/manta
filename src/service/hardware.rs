//! Hardware inventory queries for individual nodes and clusters, with concurrent fetching.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::types::NodeSummary;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::common::app_context::InfraContext;
use crate::common::authorization::{get_groups_names_available, validate_target_hsm_members};
use crate::common::node_ops;

/// Maximum number of concurrent hardware inventory requests.
const HW_INVENTORY_CONCURRENCY_LIMIT: usize = 15;

/// Divisor to convert MiB to GiB.
const MIB_PER_GIB: usize = 1024;

// ── Hardware Node ──

/// Typed parameters for fetching hardware node inventory.
pub struct GetHardwareNodeParams {
  pub xnames: String,
  /// Filter results to a specific hardware artifact type (e.g. `Processor`, `Memory`).
  pub type_artifact: Option<String>,
}

/// Result of a hardware node query.
pub struct HardwareNodeResult {
  pub node_summary: NodeSummary,
}

/// Fetch hardware inventory for a single node.
pub async fn get_hardware_node(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetHardwareNodeParams,
) -> Result<HardwareNodeResult, Error> {
  let xname_vec: Vec<String> =
    params.xnames.split(',').map(str::to_string).collect();

  validate_target_hsm_members(infra.backend, token, &xname_vec)
    .await?;

  let mut node_hw_inventory = &infra.backend
    .get_inventory_hardware_query(
      token,
      &params.xnames,
      None,
      None,
      None,
      None,
      None,
    )
    .await?;

  node_hw_inventory =
    node_hw_inventory.pointer("/Nodes/0").ok_or_else(|| {
      Error::NotFound(format!(
        "JSON section '/Nodes' missing in hardware inventory for node '{}'",
        params.xnames
      ))
    })?;

  if let Some(ref type_artifact) = params.type_artifact {
    let nodes_array = node_hw_inventory
      .as_array()
      .ok_or_else(|| Error::MissingField("Expected Nodes to be a JSON array".to_string()))?;
    let matching_node = nodes_array
      .iter()
      .find(|&node| {
        node
          .get("ID")
          .and_then(Value::as_str)
          .is_some_and(|id| id.eq(&params.xnames))
      })
      .ok_or_else(|| {
        Error::NotFound(format!(
          "Node '{}' not found in hardware inventory",
          params.xnames
        ))
      })?;
    let artifact_value = matching_node
      .get(type_artifact.as_str())
      .ok_or_else(|| {
        Error::NotFound(format!(
          "Artifact type '{}' not found in node '{}'",
          type_artifact, params.xnames
        ))
      })?;

    let node_summary = NodeSummary::from_csm_value(artifact_value.clone());
    return Ok(HardwareNodeResult { node_summary });
  }

  let node_summary = NodeSummary::from_csm_value(node_hw_inventory.clone());
  Ok(HardwareNodeResult { node_summary })
}

// ── Hardware Cluster ──

/// Typed parameters for fetching cluster hardware inventory.
pub struct GetHardwareClusterParams {
  pub hsm_group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Result of a hardware cluster query.
pub struct HardwareClusterResult {
  pub hsm_group_name: String,
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

    let backend_cp = infra.backend.clone();
    let token_str = token.to_string();
    let xname_str = xname.to_string();
    let permit = Arc::clone(&sem).acquire_owned().await;

    tasks.spawn(async move {
      let _permit = permit;
      let hw_inventory_value = backend_cp
        .get_inventory_hardware_query(
          &token_str,
          &xname_str,
          None,
          None,
          None,
          None,
          None,
        )
        .await;

      let node_hw_opt = match hw_inventory_value {
        Ok(value) => value.pointer("/Nodes/0").cloned(),
        Err(e) => {
          tracing::error!("Failed to get HW inventory for '{}': {}", xname_str, e);
          None
        }
      };

      match node_hw_opt {
        Some(v) => NodeSummary::from_csm_value(v),
        None => NodeSummary { xname: xname_str, ..Default::default() },
      }
    });
  }

  let mut summaries = Vec::with_capacity(n);
  while let Some(res) = tasks.join_next().await {
    match res {
      Ok(s) => summaries.push(s),
      Err(e) => tracing::error!("Failed fetching node hardware information: {}", e),
    }
  }
  summaries
}

/// Fetch hardware inventory for all nodes in a cluster (HSM group).
///
/// Concurrently queries hardware inventory for each node, rate-limited
/// by a semaphore.
pub async fn get_hardware_cluster(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetHardwareClusterParams,
) -> Result<HardwareClusterResult, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.hsm_group_name.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let hsm_group_name = target_hsm_group_vec
    .first()
    .ok_or_else(|| Error::NotFound("No HSM groups available for this user".to_string()))?
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

  Ok(HardwareClusterResult { hsm_group_name, node_summaries })
}

// ── Hardware Nodes List ──

/// Typed parameters for fetching hardware inventory for an explicit node list.
pub struct GetHardwareNodesListParams {
  /// Comma-separated xnames.
  pub xnames: String,
}

/// Result of a hardware nodes-list query.
pub struct HardwareNodesListResult {
  pub node_summaries: Vec<NodeSummary>,
}

/// Fetch hardware inventory for an explicit node expression.
///
/// The expression is resolved via `resolve_hosts_expression`, which expands
/// hostlist notation, translates NIDs to xnames, and validates that every
/// resolved node actually exists. Authorization is then checked with
/// `validate_target_hsm_members`.
pub async fn get_hardware_nodes_list(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetHardwareNodesListParams,
) -> Result<HardwareNodesListResult, Error> {
  let xnames = node_ops::resolve_hosts_expression(
    infra.backend,
    token,
    &params.xnames,
    false,
  )
  .await?;

  if xnames.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes is empty. Nothing to do.".to_string(),
    ));
  }

  validate_target_hsm_members(infra.backend, token, &xnames).await?;

  let node_summaries = fetch_node_summaries(infra, token, &xnames).await;
  Ok(HardwareNodesListResult { node_summaries })
}

/// Aggregate hardware component counts across nodes (summary view).
///
/// Counts processors and accelerators by info string, converts
/// memory from MiB to GiB, and counts HSN NICs.
pub fn calculate_hsm_hw_component_summary(
  node_summary_vec: &[NodeSummary],
) -> HashMap<String, usize> {
  let mut node_hw_component_summary: HashMap<String, usize> = HashMap::new();

  for node_summary in node_summary_vec {
    for artifact_summary in &node_summary.processors {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.to_string())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }
    for artifact_summary in &node_summary.node_accels {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.to_string())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }
    for artifact_summary in &node_summary.memory {
      let memory_capacity = artifact_summary
        .info
        .as_deref()
        .unwrap_or("ERROR NA")
        .split(' ')
        .collect::<Vec<_>>()
        .first()
        .copied()
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap_or(0);
      node_hw_component_summary
        .entry(artifact_summary.r#type.to_string() + " (GiB)")
        .and_modify(|qty| *qty += memory_capacity / MIB_PER_GIB)
        .or_insert(memory_capacity / MIB_PER_GIB);
    }
    for artifact_summary in &node_summary.node_hsn_nics {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.to_string())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }
  }

  node_hw_component_summary
}

/// Compute a hardware pattern (component counts with whitespace stripped).
pub fn get_cluster_hw_pattern(
  hsm_summary: Vec<NodeSummary>,
) -> HashMap<String, usize> {
  let mut hsm_node_hw_component_count_hashmap: HashMap<String, usize> =
    HashMap::new();

  for node_summary in hsm_summary {
    for processor in node_summary.processors {
      if let Some(info) = processor.info {
        hsm_node_hw_component_count_hashmap
          .entry(info.chars().filter(|c| !c.is_whitespace()).collect())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }

    for node_accel in node_summary.node_accels {
      if let Some(info) = node_accel.info {
        hsm_node_hw_component_count_hashmap
          .entry(info.chars().filter(|c| !c.is_whitespace()).collect())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }

    for memory_dimm in node_summary.memory {
      let memory_capacity = memory_dimm
        .info
        .unwrap_or_else(|| "0".to_string())
        .split(' ')
        .next()
        .unwrap_or("0")
        .to_string()
        .parse::<usize>()
        .unwrap_or(0);

      hsm_node_hw_component_count_hashmap
        .entry("memory".to_string())
        .and_modify(|qty| *qty += memory_capacity)
        .or_insert(memory_capacity);
    }
  }

  hsm_node_hw_component_count_hashmap
}

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::{ArtifactSummary, ArtifactType, NodeSummary};

  /// Helper: create an ArtifactSummary with the given info string.
  fn make_artifact(art_type: ArtifactType, info: Option<&str>) -> ArtifactSummary {
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
      node_accels: vec![
        make_artifact(ArtifactType::NodeAccel, Some("NVIDIA A100")),
      ],
      memory: vec![],
      node_hsn_nics: vec![],
      ..Default::default()
    }];
    let summary = calculate_hsm_hw_component_summary(&nodes);
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
    let summary = calculate_hsm_hw_component_summary(&nodes);
    assert_eq!(summary.get("Memory (GiB)"), Some(&32));
  }

  #[test]
  fn summary_aggregates_across_multiple_nodes() {
    let nodes = vec![
      NodeSummary {
        xname: "n1".to_string(),
        processors: vec![
          make_artifact(ArtifactType::Processor, Some("AMD EPYC 7742")),
        ],
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
    let summary = calculate_hsm_hw_component_summary(&nodes);
    assert_eq!(summary.get("AMD EPYC 7742"), Some(&2));
    assert_eq!(summary.get("Intel Xeon Gold"), Some(&1));
  }

  #[test]
  fn summary_empty_nodes() {
    let nodes: Vec<NodeSummary> = vec![];
    let summary = calculate_hsm_hw_component_summary(&nodes);
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
    let summary = calculate_hsm_hw_component_summary(&nodes);
    assert_eq!(summary.get("AMD EPYC 7742"), Some(&1));
    assert_eq!(summary.len(), 1);
  }
}
