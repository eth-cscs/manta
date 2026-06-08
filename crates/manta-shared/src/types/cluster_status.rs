//! Pure helpers for summarizing node and cluster status.
//!
//! Both the CLI (table rendering) and the server (`service::hardware`,
//! `service::hw_cluster`) call into these. Living in `manta-shared`
//! keeps the CLI from importing `crate::service::*` for what are
//! really data-only helpers.

use std::collections::HashMap;

use crate::types::dto::NodeDetails;
use manta_backend_dispatcher::types::NodeSummary;

/// Divisor to convert MiB to GiB.
const MIB_PER_GIB: usize = 1024;

/// Compute a summary status from a list of node details.
///
/// Priority order: FAILED > OFF > ON > STANDBY > UNCONFIGURED > OK.
/// The first matching condition wins, regardless of how many nodes
/// fall under lower-priority states.
///
/// # Examples
///
/// One failed node makes the whole cluster `"FAILED"`, even if every
/// other node is on and configured:
///
/// ```
/// use manta_shared::types::cluster_status::compute_summary_status;
/// use manta_shared::types::dto::NodeDetails;
///
/// fn n(power: &str, config: &str) -> NodeDetails {
///   NodeDetails {
///     xname: String::new(), nid: String::new(), hsm: String::new(),
///     power_status: power.into(),
///     desired_configuration: String::new(),
///     configuration_status: config.into(),
///     enabled: String::new(), error_count: String::new(),
///     boot_image_id: String::new(), boot_configuration: String::new(),
///     kernel_params: String::new(),
///   }
/// }
///
/// assert_eq!(
///   compute_summary_status(&[
///     n("ON", "failed"),
///     n("ON", "configured"),
///   ]),
///   "FAILED",
/// );
/// assert_eq!(
///   compute_summary_status(&[n("ON", "configured"), n("OFF", "configured")]),
///   "OFF",
/// );
/// assert_eq!(compute_summary_status(&[]), "OK");
/// ```
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
pub fn calculate_group_hw_component_summary(
  node_summary_vec: &[NodeSummary],
) -> HashMap<String, usize> {
  let mut node_hw_component_summary: HashMap<String, usize> = HashMap::new();

  for node_summary in node_summary_vec {
    for artifact_summary in &node_summary.processors {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.clone())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }
    for artifact_summary in &node_summary.node_accels {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.clone())
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
          .entry(info.clone())
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
  use manta_backend_dispatcher::types::{ArtifactSummary, ArtifactType};

  // ---- fixtures ----

  fn node(power: &str, config: &str) -> NodeDetails {
    NodeDetails {
      xname: String::new(),
      nid: String::new(),
      hsm: String::new(),
      power_status: power.to_string(),
      desired_configuration: String::new(),
      configuration_status: config.to_string(),
      enabled: String::new(),
      error_count: String::new(),
      boot_image_id: String::new(),
      boot_configuration: String::new(),
      kernel_params: String::new(),
    }
  }

  fn artifact(kind: ArtifactType, info: Option<&str>) -> ArtifactSummary {
    ArtifactSummary {
      xname: String::new(),
      r#type: kind,
      info: info.map(String::from),
    }
  }

  fn summary(
    processors: Vec<ArtifactSummary>,
    memory: Vec<ArtifactSummary>,
    accels: Vec<ArtifactSummary>,
    nics: Vec<ArtifactSummary>,
  ) -> NodeSummary {
    NodeSummary {
      xname: String::new(),
      r#type: String::new(),
      processors,
      memory,
      node_accels: accels,
      node_hsn_nics: nics,
    }
  }

  // ---- compute_summary_status priority ladder ----
  //
  // Priority: FAILED > OFF > ON > STANDBY > UNCONFIGURED > OK
  // Each test mixes a higher-priority node with lower-priority ones
  // to pin the precedence — a swap (e.g. OFF and ON reversed) would
  // change what operators see in `manta get cluster` and is silent
  // without these tests.

  #[test]
  fn summary_status_failed_beats_everything() {
    let nodes = [
      node("ON", "failed"),
      node("OFF", "configured"),
      node("on", "configured"),
    ];
    assert_eq!(compute_summary_status(&nodes), "FAILED");
  }

  #[test]
  fn summary_status_off_beats_on() {
    let nodes = [node("OFF", "configured"), node("on", "configured")];
    assert_eq!(compute_summary_status(&nodes), "OFF");
  }

  #[test]
  fn summary_status_on_beats_standby() {
    let nodes = [node("on", "configured"), node("standby", "configured")];
    assert_eq!(compute_summary_status(&nodes), "ON");
  }

  #[test]
  fn summary_status_standby_beats_unconfigured() {
    let nodes = [node("standby", "configured"), node("ready", "pending")];
    assert_eq!(compute_summary_status(&nodes), "STANDBY");
  }

  #[test]
  fn summary_status_unconfigured_when_only_config_differs() {
    let nodes = [node("ready", "pending")];
    assert_eq!(compute_summary_status(&nodes), "UNCONFIGURED");
  }

  #[test]
  fn summary_status_ok_when_all_configured_and_no_known_power_state() {
    let nodes = [node("ready", "configured"), node("ready", "configured")];
    assert_eq!(compute_summary_status(&nodes), "OK");
  }

  #[test]
  fn summary_status_empty_input_is_ok() {
    // No nodes means no `any()` matches, falls through to OK.
    // Worth pinning so callers can rely on it instead of pre-checking.
    assert_eq!(compute_summary_status(&[]), "OK");
  }

  #[test]
  fn summary_status_matches_case_insensitively() {
    // Power and configuration status checks use eq_ignore_ascii_case.
    assert_eq!(compute_summary_status(&[node("off", "configured")]), "OFF");
    assert_eq!(compute_summary_status(&[node("ON", "CONFIGURED")]), "ON");
  }

  // ---- calculate_group_hw_component_summary ----

  #[test]
  fn hw_summary_empty_input_is_empty() {
    assert!(calculate_group_hw_component_summary(&[]).is_empty());
  }

  #[test]
  fn hw_summary_counts_identical_processors_across_nodes() {
    let node_a = summary(
      vec![
        artifact(ArtifactType::Processor, Some("AMD EPYC 7763")),
        artifact(ArtifactType::Processor, Some("AMD EPYC 7763")),
      ],
      vec![],
      vec![],
      vec![],
    );
    let node_b = summary(
      vec![artifact(ArtifactType::Processor, Some("AMD EPYC 7763"))],
      vec![],
      vec![],
      vec![],
    );
    let got = calculate_group_hw_component_summary(&[node_a, node_b]);
    assert_eq!(got.get("AMD EPYC 7763"), Some(&3));
  }

  #[test]
  fn hw_summary_converts_memory_mib_to_gib() {
    // 524 288 MiB / 1024 = 512 GiB.
    let node = summary(
      vec![],
      vec![artifact(ArtifactType::Memory, Some("524288 MiB"))],
      vec![],
      vec![],
    );
    let got = calculate_group_hw_component_summary(&[node]);
    assert_eq!(got.get("Memory (GiB)"), Some(&512));
  }

  #[test]
  fn hw_summary_skips_artifacts_with_no_info_field() {
    // Processors / accels / NICs with `info = None` must not be counted.
    let node = summary(
      vec![artifact(ArtifactType::Processor, None)],
      vec![],
      vec![artifact(ArtifactType::NodeAccel, None)],
      vec![artifact(ArtifactType::NodeHsnNic, None)],
    );
    assert!(calculate_group_hw_component_summary(&[node]).is_empty());
  }

  #[test]
  fn hw_summary_treats_unparseable_memory_as_zero() {
    // "ERROR NA".parse::<usize>() fails — the function defaults to 0,
    // which still creates the entry with value 0. Pin the behaviour
    // so a future "raise on parse error" change is deliberate.
    let node = summary(
      vec![],
      vec![artifact(ArtifactType::Memory, Some("garbage"))],
      vec![],
      vec![],
    );
    let got = calculate_group_hw_component_summary(&[node]);
    assert_eq!(got.get("Memory (GiB)"), Some(&0));
  }

  // ---- get_cluster_hw_pattern ----

  #[test]
  fn hw_pattern_empty_input_is_empty() {
    assert!(get_cluster_hw_pattern(vec![]).is_empty());
  }

  #[test]
  fn hw_pattern_strips_whitespace_from_processor_info() {
    let node = summary(
      vec![artifact(ArtifactType::Processor, Some("AMD EPYC 7763"))],
      vec![],
      vec![],
      vec![],
    );
    let got = get_cluster_hw_pattern(vec![node]);
    assert_eq!(got.get("AMDEPYC7763"), Some(&1));
    assert!(
      !got.contains_key("AMD EPYC 7763"),
      "whitespace-bearing key must NOT be present"
    );
  }

  #[test]
  fn hw_pattern_aggregates_memory_as_raw_value_not_gib() {
    // Unlike `calculate_group_hw_component_summary`, this helper does
    // NOT divide memory by 1024; it sums the raw value under the
    // literal key "memory". Catches a future "let's unify these
    // helpers" change that would silently shift consumers' numbers.
    let node = summary(
      vec![],
      vec![artifact(ArtifactType::Memory, Some("512 MiB"))],
      vec![],
      vec![],
    );
    let got = get_cluster_hw_pattern(vec![node]);
    assert_eq!(got.get("memory"), Some(&512));
  }
}
