use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;

use crate::cli::commands::hw_cluster_common::utils::{
  NodeHwCountVec, calculate_hsm_hw_component_summary,
  calculate_hsm_node_scores_from_final_hsm,
  get_best_candidate_in_target_and_parent_hsm, keep_iterating_final_hsm,
  print_table_f32_score,
};

/// Generates a list of tuples with xnames and the hardware
/// summary for each node. This method keeps as many nodes
/// from the target HSM group as it can, to minimize the
/// number of nodes being changed in the cluster.
/// Returns a list of tuples, the first element is the xname
/// and the last element is a hardware summary of the node.
pub fn calculate_target_hsm_pin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>,
  user_defined_hw_component_vec: &[String],
  combination_target_parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  target_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f32>,
) -> Result<NodeHwCountVec, Error> {
  ////////////////////////////////
  // Initialize

  // Calculate hw component counters for the whole HSM group
  let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<
    String,
    usize,
  > = calculate_hsm_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );
  // Calculate hw component counters for the whole HSM group
  let target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(target_hsm_node_hw_component_count_vec);
  // Calculate hw component counters for the whole HSM group
  let parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(parent_hsm_node_hw_component_count_vec);

  // Calculate initial scores for 'target' HSM group
  let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
    calculate_hsm_node_scores_from_final_hsm(
      target_hsm_node_hw_component_count_vec,
      &target_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

  // Calculate initial scores for 'parent' HSM group
  let mut parent_hsm_node_score_tuple_vec: Vec<(String, f32)> =
    calculate_hsm_node_scores_from_final_hsm(
      parent_hsm_node_hw_component_count_vec,
      &parent_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

  // Calculate hashmap to group nodes by score for 'target'
  let mut group_target_hsm_node_by_score_hashmap: HashMap<usize, Vec<String>> =
    HashMap::new();
  for (node, score) in &target_hsm_node_score_tuple_vec {
    group_target_hsm_node_by_score_hashmap
      .entry(*score as usize)
      .and_modify(|node_vec| node_vec.push(node.to_string()))
      .or_insert(vec![node.clone()]);
  }

  // Calculate hashmap to group nodes by score for 'parent'
  let mut group_parent_hsm_node_by_score_hashmap: HashMap<usize, Vec<String>> =
    HashMap::new();
  for (node, score) in &parent_hsm_node_score_tuple_vec {
    group_parent_hsm_node_by_score_hashmap
      .entry(*score as usize)
      .and_modify(|node_vec| node_vec.push(node.to_string()))
      .or_insert(vec![node.clone()]);
  }

  let mut nodes_migrated_from_combination_target_parent_hsm: Vec<(
    String,
    HashMap<String, usize>,
  )> = Vec::new();

  let (mut best_candidate, mut best_candidate_counters) =
    get_best_candidate_in_target_and_parent_hsm(
      &mut target_hsm_node_score_tuple_vec,
      &mut parent_hsm_node_score_tuple_vec,
      target_hsm_node_hw_component_count_vec,
      parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| Error::Message("No best candidate found.".to_string()))?;

  // Check if we need to keep iterating
  let mut work_to_do = keep_iterating_final_hsm(
    user_defined_hsm_hw_components_count_hashmap,
    &combination_target_parent_hsm_hw_component_summary_hashmap,
  );

  ////////////////////////////////
  // Iterate

  let mut iter = 0;

  while work_to_do {
    tracing::info!("----- ITERATION {} -----", iter);

    tracing::info!(
      "HSM group hw component counters: {:?}",
      combination_target_parent_hsm_hw_component_summary_hashmap
    );
    tracing::info!(
      "Final hw component counters the user wants: {:?}",
      user_defined_hsm_hw_components_count_hashmap
    );
    tracing::info!(
      "Best candidate is '{}' with score {} and hw \
       component counters {:?}",
      best_candidate.0,
      best_candidate.1,
      best_candidate_counters
    );

    // Print target hsm group hw configuration in table
    print_table_f32_score(
      user_defined_hw_component_vec,
      target_hsm_node_hw_component_count_vec,
      &target_hsm_node_score_tuple_vec,
    );

    // Print parent hsm group hw configuration in table
    print_table_f32_score(
      user_defined_hw_component_vec,
      parent_hsm_node_hw_component_count_vec,
      &parent_hsm_node_score_tuple_vec,
    );

    ////////////////////////////////
    // Apply changes - Migrate from target to parent HSM

    // Add best candidate to list of nodes migrated
    nodes_migrated_from_combination_target_parent_hsm
      .push((best_candidate.0.clone(), best_candidate_counters.clone()));

    // Remove best candidate from combined HSM group
    combination_target_parent_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    // Remove best candidate from target HSM group
    target_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    // Remove best candidate from parent HSM group
    parent_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    if combination_target_parent_hsm_node_hw_component_count_vec.is_empty() {
      break;
    }

    // Calculate hw component counters for the whole HSM
    combination_target_parent_hsm_hw_component_summary_hashmap =
      calculate_hsm_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    // Remove best candidate in target HSM group scores
    target_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    // Remove best candidate in parent HSM group scores
    parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    // Recalculate scores for 'target' HSM group
    let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
      calculate_hsm_node_scores_from_final_hsm(
        target_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    // Recalculate scores for 'parent' HSM group
    let mut parent_hsm_node_score_tuple_vec: Vec<(String, f32)> =
      calculate_hsm_node_scores_from_final_hsm(
        parent_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    // Calculate hashmap to group nodes by score
    let mut group_target_hsm_node_by_score_hashmap: HashMap<
      usize,
      Vec<String>,
    > = HashMap::new();
    for (node, score) in &target_hsm_node_score_tuple_vec {
      group_target_hsm_node_by_score_hashmap
        .entry(*score as usize)
        .and_modify(|node_vec| node_vec.push(node.to_string()))
        .or_insert(vec![node.clone()]);
    }

    // Calculate hashmap to group nodes by score
    let mut group_parent_hsm_node_by_score_hashmap: HashMap<
      usize,
      Vec<String>,
    > = HashMap::new();
    for (node, score) in &parent_hsm_node_score_tuple_vec {
      group_parent_hsm_node_by_score_hashmap
        .entry(*score as usize)
        .and_modify(|node_vec| node_vec.push(node.to_string()))
        .or_insert(vec![node.clone()]);
    }

    (best_candidate, best_candidate_counters) =
      get_best_candidate_in_target_and_parent_hsm(
        &mut target_hsm_node_score_tuple_vec,
        &mut parent_hsm_node_score_tuple_vec,
        target_hsm_node_hw_component_count_vec,
        parent_hsm_node_hw_component_count_vec,
      )
      .ok_or_else(|| Error::Message("No best candidate found.".to_string()))?;

    // Check if we need to keep iterating
    work_to_do = keep_iterating_final_hsm(
      user_defined_hsm_hw_components_count_hashmap,
      &combination_target_parent_hsm_hw_component_summary_hashmap,
    );

    iter += 1;
  }

  tracing::info!("----- FINAL RESULT -----");

  tracing::info!("No candidates found");

  // Print target hsm group hw configuration in table
  print_table_f32_score(
    user_defined_hw_component_vec,
    target_hsm_node_hw_component_count_vec,
    &target_hsm_node_score_tuple_vec,
  );

  // Print parent hsm group hw configuration in table
  print_table_f32_score(
    user_defined_hw_component_vec,
    parent_hsm_node_hw_component_count_vec,
    &parent_hsm_node_score_tuple_vec,
  );

  Ok(nodes_migrated_from_combination_target_parent_hsm)
}
