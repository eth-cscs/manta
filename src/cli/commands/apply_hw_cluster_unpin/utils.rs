use std::collections::HashMap;

use anyhow::Error;

use crate::cli::commands::hw_cluster_common::utils::{
  NodeHwCountVec, calculate_hsm_hw_component_summary,
  calculate_hsm_node_scores_from_final_hsm, get_best_candidate_in_hsm,
  keep_iterating_final_hsm, print_table_f32_score,
};

/// Generates a list of tuples with xnames and the hardware
/// summary for each node. This method merges both target and
/// parent HSM groups into a single one, this is a good
/// practice to get rid of fragmentation and also when
/// calculating new target HSM group based on deltas.
/// Returns a list of tuples, the first element is the xname
/// and the last element is a hardware summary of the node.
pub fn calculate_target_hsm_unpin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>,
  user_defined_hw_component_vec: &[String],
  combination_target_parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
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

  // Calculate initial scores
  let mut combination_target_parent_hsm_node_score_tuple_vec: Vec<(
    String,
    f32,
  )> = calculate_hsm_node_scores_from_final_hsm(
    combination_target_parent_hsm_node_hw_component_count_vec,
    &combination_target_parent_hsm_hw_component_summary_hashmap,
    user_defined_hsm_hw_components_count_hashmap,
    hw_component_scarcity_scores_hashmap,
  );

  let mut nodes_migrated_from_combination_target_parent_hsm: Vec<(
    String,
    HashMap<String, usize>,
  )> = Vec::new();

  // Get best candidate
  let (mut best_candidate, mut best_candidate_counters) =
    get_best_candidate_in_hsm(
      &mut combination_target_parent_hsm_node_score_tuple_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| Error::msg("No best candidate found."))?;

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
      combination_target_parent_hsm_node_score_tuple_vec
        .iter()
        .find(|(node, _score)| node.eq(&best_candidate.0))
        .map(|(_, score)| *score)
        .unwrap_or(0.0),
      best_candidate_counters
    );

    // Print target hsm group hw configuration in table
    print_table_f32_score(
      user_defined_hw_component_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
      &combination_target_parent_hsm_node_score_tuple_vec,
    );

    ////////////////////////////////
    // Apply changes - Migrate from target to parent HSM

    // Add best candidate to list of nodes migrated
    nodes_migrated_from_combination_target_parent_hsm
      .push((best_candidate.0.clone(), best_candidate_counters.clone()));

    // Remove best candidate from target HSM group
    combination_target_parent_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    if combination_target_parent_hsm_node_hw_component_count_vec.is_empty() {
      break;
    }

    // Calculate hw component counters for the whole HSM group
    combination_target_parent_hsm_hw_component_summary_hashmap =
      calculate_hsm_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    // Remove best candidate from scores
    combination_target_parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    // Recalculate scores
    let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
      calculate_hsm_node_scores_from_final_hsm(
        combination_target_parent_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    // Get best candidate
    (best_candidate, best_candidate_counters) = get_best_candidate_in_hsm(
      &mut target_hsm_node_score_tuple_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| Error::msg("No best candidate found."))?;

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
    combination_target_parent_hsm_node_hw_component_count_vec,
    &combination_target_parent_hsm_node_score_tuple_vec,
  );

  Ok(nodes_migrated_from_combination_target_parent_hsm)
}
