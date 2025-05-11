use std::collections::HashMap;

use crate::cli::commands::apply_hw_cluster_pin::utils::{
  calculate_hsm_hw_component_summary, resolve_hw_description_to_xnames,
};

#[tokio::test]
pub async fn test_hsm_hw_management_1() {
  let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

  let hsm_zinal_hw_counters = vec![
    (
      "x1001c1s5b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 15),
      ]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s7b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s7b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s7b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s7b1n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("a100".to_string(), 4),
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
        ("a100".to_string(), 4),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("instinct".to_string(), 8),
        ("Memory 16384".to_string(), 32),
        ("epyc".to_string(), 1),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("instinct".to_string(), 8),
        ("epyc".to_string(), 1),
        ("Memory 16384".to_string(), 32),
      ]),
    ),
  ];

  let hsm_nodes_free_hw_conters = vec![
    (
      "x1000c1s7b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1000c1s7b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1000c1s7b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1000c1s7b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s1b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s1b0n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s1b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s1b1n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s2b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s2b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b0n0".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
    (
      "x1001c1s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 2),
        ("Memory 16384".to_string(), 16),
      ]),
    ),
    (
      "x1001c1s4b1n1".to_string(),
      HashMap::from([
        ("Memory 16384".to_string(), 16),
        ("epyc".to_string(), 2),
      ]),
    ),
  ];

  let (target_hsm_node_hw_component_count_vec, _) =
    resolve_hw_description_to_xnames(
      hsm_zinal_hw_counters,
      hsm_nodes_free_hw_conters,
      user_request_hw_summary.clone(),
    )
    .await;

  println!(
    "DEBUG - target HSM group:\n{:#?}",
    target_hsm_node_hw_component_count_vec
  );

  let target_hsm_hw_summary: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  println!(
    "DEBUG - target HSM group hw summary:\n{:#?}",
    target_hsm_hw_summary
  );

  // Check if user request is fulfilled
  let mut success = true;
  for (hw_component, qty) in user_request_hw_summary {
    if target_hsm_hw_summary.get(&hw_component).is_none()
      || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
    {
      println!("DEBUG - hw component '{}' with quantity '{}' in user request does not comply with solution {:#?}", hw_component, qty, target_hsm_hw_summary);
      success = false;
    }
  }

  println!("DEBUG - success? {}", success);

  assert!(success)
}

/// Test pinning
#[tokio::test]
pub async fn test_hsm_hw_management_2() {
  let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

  let hsm_zinal_hw_counters = vec![
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
      ]),
    ),
  ];

  let hsm_nodes_free_hw_conters = vec![
    (
      "x1001c1s5b0n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s5b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s5b1n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b0n1".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n0".to_string(),
      HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
    ),
    (
      "x1001c1s6b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b0n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b0n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b1n0".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1001c1s7b1n1".to_string(),
      HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
    ),
    (
      "x1005c0s4b0n0".to_string(),
      HashMap::from([
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1005c0s4b0n1".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("nvidia_a100-sxm4-80gb".to_string(), 4),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b0n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
        ("memory".to_string(), 32),
      ]),
    ),
    (
      "x1006c1s4b1n0".to_string(),
      HashMap::from([
        ("epyc".to_string(), 1),
        ("memory".to_string(), 32),
        ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
      ]),
    ),
  ];

  let (target_hsm_node_hw_component_count_vec, _) =
    resolve_hw_description_to_xnames(
      hsm_zinal_hw_counters.clone(),
      hsm_nodes_free_hw_conters,
      user_request_hw_summary.clone(),
    )
    .await;

  println!(
    "DEBUG - target HSM group:\n{:#?}",
    target_hsm_node_hw_component_count_vec
  );

  let target_hsm_hw_summary: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  println!(
    "DEBUG - target HSM group hw summary:\n{:#?}",
    target_hsm_hw_summary
  );

  // Check if user request is fulfilled
  // Check new target HSM group hw summary fulfills the user request
  let mut success = true;
  for (hw_component, qty) in user_request_hw_summary {
    if target_hsm_hw_summary.get(&hw_component).is_none()
      || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
    {
      println!("DEBUG - hw component '{}' with quantity '{}' in user request does not comply with solution {:#?}", hw_component, qty, target_hsm_hw_summary);
      success = false;
    }
  }

  // Check pinning and the number of xnames in new target HSM group maximizes the ones in the old
  // target HSM group
  success = success
    && target_hsm_node_hw_component_count_vec.iter().all(
      |(new_target_xname, _)| {
        hsm_zinal_hw_counters
          .iter()
          .any(|(old_target_xname, _)| old_target_xname == new_target_xname)
      },
    );

  println!("DEBUG - success? {}", success);

  assert!(success)
}
