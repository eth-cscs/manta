use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::types::NodeSummary;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::common::app_context::InfraContext;
use crate::common::authorization::{get_groups_names_available, validate_target_hsm_members};

/// Maximum number of concurrent hardware inventory requests.
const HW_INVENTORY_CONCURRENCY_LIMIT: usize = 15;

/// Divisor to convert MiB to GiB.
const MIB_PER_GIB: usize = 1024;

// ── Hardware Node ──

/// Typed parameters for fetching hardware node inventory.
pub struct GetHardwareNodeParams {
  pub xnames: String,
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

  validate_target_hsm_members(infra.backend, token, &xname_vec).await?;

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
    .await
    .context("Failed to query hardware inventory")?;

  node_hw_inventory =
    node_hw_inventory.pointer("/Nodes/0").ok_or_else(|| {
      Error::msg(format!(
        "JSON section '/Nodes' missing in json response API for node '{}'",
        params.xnames
      ))
    })?;

  if let Some(ref type_artifact) = params.type_artifact {
    let nodes_array = node_hw_inventory
      .as_array()
      .context("Expected Nodes to be a JSON array")?;
    let matching_node = nodes_array
      .iter()
      .find(|&node| {
        node
          .get("ID")
          .and_then(Value::as_str)
          .is_some_and(|id| id.eq(&params.xnames))
      })
      .ok_or_else(|| {
        Error::msg(format!(
          "Node '{}' not found in hardware inventory",
          params.xnames
        ))
      })?;
    let artifact_value = matching_node
      .get(type_artifact.as_str())
      .ok_or_else(|| {
        Error::msg(format!(
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
    .context("No HSM groups available for this user")?
    .clone();

  let hsm_group = infra.backend
    .get_group(token, &hsm_group_name)
    .await
    .context("Failed to get HSM group")?;

  let hsm_group_target_members = hsm_group
    .members
    .unwrap_or_default()
    .ids
    .unwrap_or_default();

  if hsm_group_target_members.is_empty() {
    log::warn!("HSM group '{}' has no members", hsm_group.label);
  }

  log::debug!(
    "Get HW artifacts for nodes in HSM group '{}' and members {:?}",
    hsm_group.label,
    hsm_group_target_members
  );

  let mut hsm_summary: Vec<NodeSummary> = Vec::new();

  let start_total = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();
  let sem = Arc::new(Semaphore::new(HW_INVENTORY_CONCURRENCY_LIMIT));

  let num_hsm_group_members = hsm_group_target_members.len();
  let width = num_hsm_group_members.checked_ilog10().unwrap_or(0) as usize + 1;
  let mut i = 1;

  for hsm_member in hsm_group_target_members.iter() {
    log::info!(
      "\rGetting hw components for node '{hsm_member}' [{:>width$}/{num_hsm_group_members}]",
      i + 1
    );

    let backend_cp = infra.backend.clone();
    let shasta_token_string = token.to_string();
    let hsm_member_string = hsm_member.to_string();

    let permit = Arc::clone(&sem).acquire_owned().await;

    tasks.spawn(async move {
      let _permit = permit;
      let hw_inventory_value = backend_cp
        .get_inventory_hardware_query(
          &shasta_token_string,
          &hsm_member_string,
          None,
          None,
          None,
          None,
          None,
        )
        .await;

      let node_hw_inventory_value_opt = match hw_inventory_value {
        Ok(value) => value.pointer("/Nodes/0").cloned(),
        Err(e) => {
          log::error!(
            "Failed to get HW inventory for '{}': {}",
            hsm_member_string,
            e
          );
          None
        }
      };

      match node_hw_inventory_value_opt {
        Some(node_hw_inventory) => NodeSummary::from_csm_value(node_hw_inventory),
        None => NodeSummary {
          xname: hsm_member_string,
          ..Default::default()
        },
      }
    });

    i += 1;
  }

  while let Some(message) = tasks.join_next().await {
    match message {
      Ok(node_hw_inventory) => {
        hsm_summary.push(node_hw_inventory);
      }
      Err(e) => log::error!("Failed fetching node hardware information: {}", e),
    }
  }

  let duration = start_total.elapsed();

  log::info!(
    "Time elapsed in http calls to get hw inventory for HSM '{}' is: {:?}",
    hsm_group_name,
    duration
  );

  Ok(HardwareClusterResult {
    hsm_group_name,
    node_summaries: hsm_summary,
  })
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
