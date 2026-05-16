//! Pure helpers for summarizing node and cluster status.
//!
//! Both the CLI (table rendering) and the server (`service::hardware`,
//! `service::hw_cluster`) call into these. Living in `shared/` keeps the
//! CLI from importing `crate::service::*` for what are really data-only
//! helpers.

use std::collections::HashMap;

use csm_rs::node::types::NodeDetails;
use manta_backend_dispatcher::types::NodeSummary;

/// Divisor to convert MiB to GiB.
const MIB_PER_GIB: usize = 1024;

/// Compute a summary status from a list of node details.
///
/// Priority order: FAILED > OFF > ON > STANDBY > UNCONFIGURED > OK
pub fn compute_summary_status(nodes: &[NodeDetails]) -> &'static str {
  if nodes
    .iter()
    .any(|n| n.configuration_status.eq_ignore_ascii_case("failed"))
  {
    "FAILED"
  } else if nodes
    .iter()
    .any(|n| n.power_status.eq_ignore_ascii_case("OFF"))
  {
    "OFF"
  } else if nodes
    .iter()
    .any(|n| n.power_status.eq_ignore_ascii_case("on"))
  {
    "ON"
  } else if nodes
    .iter()
    .any(|n| n.power_status.eq_ignore_ascii_case("standby"))
  {
    "STANDBY"
  } else if nodes
    .iter()
    .any(|n| !n.configuration_status.eq_ignore_ascii_case("configured"))
  {
    "UNCONFIGURED"
  } else {
    "OK"
  }
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
