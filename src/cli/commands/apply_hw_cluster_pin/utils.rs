use std::{collections::HashMap, sync::Arc, time::Instant};

use comfy_table::Color;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

// Returns a tuple (target_hsm, parent_hsm) with 2 list of nodes and its hardware components.
// The left tuple element are the nodes moved from the
pub async fn resolve_hw_description_to_xnames(
  mut target_hsm_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )>,
  mut parent_hsm_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )>,
  user_defined_target_hsm_hw_component_count_hashmap: HashMap<String, usize>,
) -> (
  Vec<(String, HashMap<String, usize>)>,
  Vec<(String, HashMap<String, usize>)>,
) {
  // *********************************************************************************************************
  // CALCULATE 'COMBINED HSM' WITH TARGET HSM AND PARENT HSM ELEMENTS COMBINED
  // NOTE: PARENT HSM may contain elements in TARGET HSM, we need to only add those xnames
  // which are not part of PARENT HSM already

  let mut combined_target_parent_hsm_node_hw_component_count_vec =
    parent_hsm_node_hw_component_count_vec.clone();

  for elem in &target_hsm_node_hw_component_count_vec {
    if !parent_hsm_node_hw_component_count_vec
      .iter()
      .any(|(xname, _)| xname.eq(&elem.0))
    {
      combined_target_parent_hsm_node_hw_component_count_vec.push(elem.clone());
    }
  }

  let combined_target_parent_hsm_hw_component_summary_hashmap =
    calculate_hsm_hw_component_summary(
      &combined_target_parent_hsm_node_hw_component_count_vec,
    );

  // *********************************************************************************************************
  // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY

  // Get parent HSM group members
  // Calculate nomarlized score for each hw component type in as much HSM groups as possible
  // related to the stakeholders using these nodes
  let hw_component_scarcity_scores_hashmap: HashMap<String, f32> =
    calculate_hw_component_scarcity_scores(
      &combined_target_parent_hsm_node_hw_component_count_vec,
    )
    .await;

  // *********************************************************************************************************
  // CALCULATE FINAL HSM SUMMARY COUNTERS AFTER REMOVING THE NODES THAT NEED TO GO TO TARGET
  // HSM (SUBSTRACT USER INPUT SUMMARY FROM INITIAL COMBINED HSM SUMMARY)
  let mut final_combined_target_parent_hsm_hw_component_summary =
    user_defined_target_hsm_hw_component_count_hashmap.clone();

  for (hw_component, qty) in
    combined_target_parent_hsm_hw_component_summary_hashmap
  {
    final_combined_target_parent_hsm_hw_component_summary
      .entry(hw_component)
      .and_modify(|current_qty| *current_qty = qty - *current_qty);
  }

  // Calculate new target HSM group
  let hw_component_counters_to_move_out_from_combined_hsm =
    calculate_target_hsm_pin(
      &final_combined_target_parent_hsm_hw_component_summary.clone(),
      &final_combined_target_parent_hsm_hw_component_summary
        .into_keys()
        .collect::<Vec<String>>(),
      &mut combined_target_parent_hsm_node_hw_component_count_vec,
      &mut target_hsm_node_hw_component_count_vec,
      &mut parent_hsm_node_hw_component_count_vec,
      &hw_component_scarcity_scores_hashmap,
    );

  let new_target_hsm_node_hw_component_count_vec =
    hw_component_counters_to_move_out_from_combined_hsm;
  (
    new_target_hsm_node_hw_component_count_vec,
    combined_target_parent_hsm_node_hw_component_count_vec,
  )
}

/// Pin means this function should be used when the user wants to keep as much nodes in
/// original target HSM group as possible. Use case for this:
///  - cluster upscaling or downscaling in same site and want to minimize the impact on running
///  applications
///  - defining final state for a cluster
pub fn get_best_candidate_in_hsm_pin(
  hsm_score_vec: &mut [(String, f32)],
  hsm_hw_component_vec: &[(String, HashMap<String, usize>)],
) -> Option<((String, f32), HashMap<String, usize>)> {
  if hsm_score_vec.is_empty() || hsm_hw_component_vec.is_empty() {
    return None;
  }

  hsm_score_vec.sort_by_key(|elem| elem.0.clone());
  hsm_score_vec.sort_by(|b, a| a.1.partial_cmp(&b.1).unwrap());

  // Get node with highest normalized score (best candidate)
  let best_candidate: (String, f32) = hsm_score_vec.first().unwrap().clone();

  if let Some(best_candiate) = hsm_hw_component_vec
    .iter()
    .find(|(node, _)| node.eq(&best_candidate.0))
  {
    Some((best_candidate, best_candiate.1.clone()))
  } else {
    None
  }
}

pub fn get_best_candidate_in_target_and_parent_hsm_pin(
  target_hsm_node_score_tuple_vec: &mut [(String, f32)],
  parent_hsm_node_score_tuple_vec: &mut [(String, f32)],
  target_hsm_node_hw_component_count_vec: &mut Vec<(
    String,
    HashMap<String, usize>,
  )>,
  parent_hsm_node_hw_component_count_vec: &Vec<(
    String,
    HashMap<String, usize>,
  )>,
) -> Option<((String, f32), HashMap<String, usize>)> {
  // Get best candidate in 'target' HSM group
  let target_best_candidate_tuple = get_best_candidate_in_hsm_pin(
    target_hsm_node_score_tuple_vec,
    target_hsm_node_hw_component_count_vec,
  );

  // Get best candidate in 'parent' HSM group
  let parent_best_candidate_tuple = get_best_candidate_in_hsm_pin(
    parent_hsm_node_score_tuple_vec,
    parent_hsm_node_hw_component_count_vec,
  );

  // If best candidate exists (in 'target' HSM group), then use it. Otherwise, use the one in 'parent' HSM group
  if target_best_candidate_tuple.is_some() {
    target_best_candidate_tuple
  } else if parent_best_candidate_tuple.is_some() {
    parent_best_candidate_tuple
  } else {
    None
  }

  /* if let Some(target_best_candidate) = target_best_candidate_tuple {
      target_best_candidate
  } else if let Some(parent_best_candidate) = parent_best_candidate_tuple {
      parent_best_candidate
  } else {
      eprintln!("ERROR - No best candidate found.");
      std::process::exit(1);
  } */
}

/// Generates a list of tuples with xnames and the hardware summary for each node. This method
/// keeps as much nodes from the target HSM group as it can, this is good to minimize the
/// number of nodes being changed in the cluster
/// Returns a list of tuples, the first element is the xname and the last element is a hardware
/// summary of the node
pub fn calculate_target_hsm_pin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>, // hw
  // components summary the target hsm group should have according to user requests (this is
  // equivalent to target_hsm_node_hw_component_count_vec minus
  // hw_components_deltas_from_target_hsm_to_parent_hsm). Note hw componets needs to be grouped/filtered
  // based on user input
  user_defined_hw_component_vec: &[String], // list of hw components the user is asking
  // for
  combination_target_parent_hsm_node_hw_component_count_vec: &mut Vec<(
    String,
    HashMap<String, usize>,
  )>, // list
  // of hw component counters in target HSM group
  target_hsm_node_hw_component_count_vec: &mut Vec<(
    String,
    HashMap<String, usize>,
  )>,
  parent_hsm_node_hw_component_count_vec: &mut Vec<(
    String,
    HashMap<String, usize>,
  )>,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f32>, // hw
                                                               // component type score for as much hsm groups related to the stakeholders using these
                                                               // nodes
) -> Vec<(String, HashMap<String, usize>)> {
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

  // Calculate hashmap to group nodes by score for 'target' HSM group
  let mut group_target_hsm_node_by_score_hashmap: HashMap<usize, Vec<String>> =
    HashMap::new();
  for (node, score) in &target_hsm_node_score_tuple_vec {
    group_target_hsm_node_by_score_hashmap
      .entry(*score as usize)
      .and_modify(|node_vec| node_vec.push(node.to_string()))
      .or_insert(vec![node.clone()]);
  }

  // Calculate hashmap to group nodes by score for 'parent' HSM group
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

  /* // Get best candidate in 'target' HSM group
  let target_best_candidate_tuple =
      get_best_candidate_in_hsm_based_on_scarcity_node_score_pin(
          &mut target_hsm_node_score_tuple_vec,
          target_hsm_node_hw_component_count_vec,
      );

  // Get best candidate in 'parent' HSM group
  let parent_best_candidate_tuple =
      get_best_candidate_in_hsm_based_on_scarcity_node_score_pin(
          &mut parent_hsm_node_score_tuple_vec,
          parent_hsm_node_hw_component_count_vec,
      );

  // If best candidate exists (in 'target' HSM group), then use it. Otherwise, use the one in 'parent' HSM group
  let (mut best_candidate, mut best_candidate_counters) =
      if let Some(target_best_candidate) = target_best_candidate_tuple {
          target_best_candidate
      } else if let Some(parent_best_candidate) = parent_best_candidate_tuple {
          parent_best_candidate
      } else {
          eprintln!("ERROR - No best candidate found.");
          std::process::exit(1);
      }; */

  let (mut best_candidate, mut best_candidate_counters) =
    get_best_candidate_in_target_and_parent_hsm_pin(
      &mut target_hsm_node_score_tuple_vec,
      &mut parent_hsm_node_score_tuple_vec,
      target_hsm_node_hw_component_count_vec,
      parent_hsm_node_hw_component_count_vec,
    )
    .unwrap_or_else(|| {
      eprintln!("ERROR - No best candidate found.");
      std::process::exit(1);
    });

  // Check if we need to keep iterating
  let mut work_to_do = keep_iterating_final_hsm(
    user_defined_hsm_hw_components_count_hashmap,
    &combination_target_parent_hsm_hw_component_summary_hashmap,
  );

  ////////////////////////////////
  // Iterate

  let mut iter = 0;

  while work_to_do {
    log::info!("----- ITERATION {} -----", iter);

    log::info!(
      "HSM group hw component counters: {:?}",
      combination_target_parent_hsm_hw_component_summary_hashmap
    );
    log::info!(
      "Final hw component counters the user wants: {:?}",
      user_defined_hsm_hw_components_count_hashmap
    );
    log::info!(
      "Best candidate is '{}' with score {} and hw component counters {:?}",
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

    // Print target hsm group hw configuration in table
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

    // Calculate hw component couters for the whole HSM group
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

    /* // Get best candidate in 'target' HSM group
    let target_best_candidate_tuple =
        get_best_candidate_in_hsm_based_on_scarcity_node_score_pin(
            &mut target_hsm_node_score_tuple_vec,
            target_hsm_node_hw_component_count_vec,
        );

    // Get best candidate in 'parent' HSM group
    let parent_best_candidate_tuple =
        get_best_candidate_in_hsm_based_on_scarcity_node_score_pin(
            &mut parent_hsm_node_score_tuple_vec,
            parent_hsm_node_hw_component_count_vec,
        );

    // If best candidate exists (in 'target' HSM group), then use it. Otherwise, use the one in 'parent' HSM group
    (best_candidate, best_candidate_counters) =
        if let Some(target_best_candidate) = target_best_candidate_tuple {
            target_best_candidate
        } else if let Some(parent_best_candidate) = parent_best_candidate_tuple {
            parent_best_candidate
        } else {
            eprintln!("ERROR - No best candidate found.");
            std::process::exit(1);
        }; */

    (best_candidate, best_candidate_counters) =
      get_best_candidate_in_target_and_parent_hsm_pin(
        &mut target_hsm_node_score_tuple_vec,
        &mut parent_hsm_node_score_tuple_vec,
        target_hsm_node_hw_component_count_vec,
        parent_hsm_node_hw_component_count_vec,
      )
      .expect("ERROR - No best candidate found.");

    // Check if we need to keep iterating
    work_to_do = keep_iterating_final_hsm(
      user_defined_hsm_hw_components_count_hashmap,
      // &best_candidate_counters,
      // &downscale_deltas,
      &combination_target_parent_hsm_hw_component_summary_hashmap,
    );

    iter += 1;
  }

  log::info!("----- FINAL RESULT -----");

  log::info!("No candidates found");

  // Print target hsm group hw configuration in table
  print_table_f32_score(
    user_defined_hw_component_vec,
    target_hsm_node_hw_component_count_vec,
    &target_hsm_node_score_tuple_vec,
  );

  // Print target hsm group hw configuration in table
  print_table_f32_score(
    user_defined_hw_component_vec,
    parent_hsm_node_hw_component_count_vec,
    &parent_hsm_node_score_tuple_vec,
  );

  nodes_migrated_from_combination_target_parent_hsm
}

pub async fn calculate_hw_component_scarcity_scores(
  hsm_node_hw_component_count: &Vec<(String, HashMap<String, usize>)>,
) -> HashMap<String, f32> {
  let total_num_hw_components: usize = hsm_node_hw_component_count
    .iter()
    .flat_map(|(_, hw_component_qty_hashmap)| {
      hw_component_qty_hashmap
        .iter()
        .map(|(_, hw_component_qty)| hw_component_qty)
    })
    .sum();

  let mut hw_component_vec: Vec<&String> = hsm_node_hw_component_count
    .iter()
    .flat_map(|(_, hw_component_counter_hashmap)| {
      hw_component_counter_hashmap.keys()
    })
    .collect();

  hw_component_vec.sort();
  hw_component_vec.dedup();

  let mut hw_component_scarcity_score_hashmap: HashMap<String, f32> =
    HashMap::new();
  for hw_component in hw_component_vec {
    let mut hsm_hw_component_count = 0;

    for (_, hw_component_counter_hashmap) in hsm_node_hw_component_count {
      if let Some(hw_component_qty) =
        hw_component_counter_hashmap.get(hw_component)
      {
        hsm_hw_component_count += hw_component_qty;
      }
    }

    hw_component_scarcity_score_hashmap.insert(
      hw_component.to_string(),
      (total_num_hw_components as f32) / (hsm_hw_component_count as f32),
    );
  }

  log::info!(
    "Hw component scarcity scores: {:?}",
    hw_component_scarcity_score_hashmap
  );

  hw_component_scarcity_score_hashmap
}
/// Calculates a normalized score for each hw component in HSM group based on component
/// scarcity.
pub fn calculate_hsm_node_scores_from_final_hsm(
  parent_hsm_node_hw_component_count_vec: &Vec<(
    String,
    HashMap<String, usize>,
  )>,
  parent_hsm_hw_component_summary_hashmap: &HashMap<String, usize>,
  final_hsm_summary_hashmap: &HashMap<String, usize>,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f32>,
) -> Vec<(String, f32)> {
  let mut node_score_vec: Vec<(String, f32)> = Vec::new();

  for (xname, hw_component_count) in parent_hsm_node_hw_component_count_vec {
    let mut node_score: f32 = 0.0;
    for (hw_component, qty) in hw_component_count {
      if final_hsm_summary_hashmap.get(hw_component).is_none() {
        // final/user request does NOT contain hw component
        // negative - current hw component counter in HSM group is not requested by the user therefor we should
        // penalize this node
        node_score -= hw_component_scarcity_scores_hashmap
          .get(hw_component)
          .unwrap()
          * *qty as f32;
      } else {
        // final/user request does contain hw component
        if final_hsm_summary_hashmap.get(hw_component).unwrap()
          < parent_hsm_hw_component_summary_hashmap
            .get(hw_component)
            .unwrap()
        {
          // positive - current hw component counter in parent/combined HSM group are higher than
          // final (user requested) hw component counter therefore we remove this node
          node_score += hw_component_scarcity_scores_hashmap
            .get(hw_component)
            .unwrap()
            * *qty as f32;
        } else {
          // negative - current hw component counter in parent/combined HSM group is lower or
          // equal than final (user requested) hw component counter therefor we should
          // penalize this node
          node_score -= hw_component_scarcity_scores_hashmap
            .get(hw_component)
            .unwrap()
            * *qty as f32;
        }
      }
    }
    node_score_vec.push((xname.to_string(), node_score));
  }

  node_score_vec
}

pub fn keep_iterating_final_hsm(
  hsm_final_hw_component_summary_hashmap: &HashMap<String, usize>, // hw components in
  // the target hsm group asked by the user (this is the minimum boundary, we can't provide
  // less than this)
  // best_candidate_counters: &HashMap<String, usize>,
  // hw_components_deltas_from_target_hsm_to_parent_hsm: &HashMap<String, isize>, // minimum boundaries (we
  // can't provide less that this)
  hsm_current_hw_component_summary_hashmap: &HashMap<String, usize>, // list of nodes
                                                                     // and its scores
) -> bool {
  for (hw_component, final_qty) in hsm_final_hw_component_summary_hashmap {
    if hsm_current_hw_component_summary_hashmap
      .get(hw_component)
      .is_some_and(|current_qty| current_qty > final_qty)
    {
      return true;
    }
  }

  false
}

/// Returns a triple like (<xname>, <list of hw components>, <list of memory capacity>)
/// Note: list of hw components can be either the hw componentn pattern provided by user or the
/// description from the HSM API
/// NOTE: backend it not borrowed because we need to clone it in order to use it across threads
pub async fn get_node_hw_component_count(
  backend: StaticBackendDispatcher,
  shasta_token: String,
  hsm_member: &str,
  user_defined_hw_profile_vec: Vec<String>,
) -> (String, Vec<String>, Vec<u64>) {
  let node_hw_inventory_value = backend
    .get_inventory_hardware_query(
      &shasta_token,
      /* &shasta_base_url,
      &shasta_root_cert, */
      hsm_member,
      None,
      None,
      None,
      None,
      None,
    )
    .await
    .unwrap();
  /* hsm::hw_inventory::hw_component::http_client::get_hw_inventory(
      &shasta_token,
      &shasta_base_url,
      &shasta_root_cert,
      hsm_member,
  )
  .await
  .unwrap(); */

  let node_hw_profile = get_node_hw_properties_from_value(
    &node_hw_inventory_value,
    user_defined_hw_profile_vec.clone(),
  );

  (hsm_member.to_string(), node_hw_profile.0, node_hw_profile.1)
}

// Calculate/groups hw component counters
pub fn calculate_hsm_hw_component_summary(
  target_hsm_group_node_hw_component_vec: &Vec<(
    String,
    HashMap<String, usize>,
  )>,
) -> HashMap<String, usize> {
  let mut hsm_hw_component_count_hashmap = HashMap::new();

  for (_xname, node_hw_component_count_hashmap) in
    target_hsm_group_node_hw_component_vec
  {
    for (hw_component, &qty) in node_hw_component_count_hashmap {
      hsm_hw_component_count_hashmap
        .entry(hw_component.to_string())
        .and_modify(|qty_aux| *qty_aux += qty)
        .or_insert(qty);
    }
  }

  hsm_hw_component_count_hashmap
}

/// Returns the properties in hw_property_list found in the node_hw_inventory_value which is
/// HSM hardware inventory API json response
pub fn get_node_hw_properties_from_value(
  node_hw_inventory_value: &Value,
  hw_component_pattern_list: Vec<String>,
) -> (Vec<String>, Vec<u64>) {
  let processor_vec =
        common::hw_inventory_utils::get_list_processor_model_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

  let accelerator_vec =
        common::hw_inventory_utils::get_list_accelerator_model_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

  let processor_and_accelerator = [processor_vec, accelerator_vec].concat();

  let processor_and_accelerator_lowercase = processor_and_accelerator
    .iter()
    .map(|hw_component| hw_component.to_lowercase());

  let mut node_hw_component_pattern_vec = Vec::new();

  for actual_hw_component_pattern in processor_and_accelerator_lowercase {
    if let Some(hw_component_pattern) = hw_component_pattern_list
      .iter()
      .find(|&hw_component| actual_hw_component_pattern.contains(hw_component))
    {
      node_hw_component_pattern_vec.push(hw_component_pattern.to_string());
    } else {
      node_hw_component_pattern_vec.push(actual_hw_component_pattern);
    }
  }

  let memory_vec = common::hw_inventory_utils::get_list_memory_capacity_from_hw_inventory_value(
        node_hw_inventory_value,
    )
    .unwrap_or_default();

  (node_hw_component_pattern_vec, memory_vec)
}

pub fn print_table_f32_score(
  user_defined_hw_componet_vec: &[String],
  hsm_hw_pattern_vec: &[(String, HashMap<String, usize>)],
  hsm_score_vec: &[(String, f32)],
) {
  let hsm_hw_component_vec: Vec<String> = hsm_hw_pattern_vec
    .iter()
    .flat_map(|(_xname, node_pattern_hashmap)| {
      node_pattern_hashmap.keys().cloned()
    })
    .collect();

  let mut all_hw_component_vec =
    [hsm_hw_component_vec, user_defined_hw_componet_vec.to_vec()].concat();

  all_hw_component_vec.sort();
  all_hw_component_vec.dedup();

  let mut table = comfy_table::Table::new();

  table.set_header(
    [
      vec!["Node".to_string()],
      all_hw_component_vec.clone(),
      vec!["Score".to_string()],
    ]
    .concat(),
  );

  for (xname, node_pattern_hashmap) in hsm_hw_pattern_vec {
    // println!("node_pattern_hashmap: {:?}", node_pattern_hashmap);

    let mut row: Vec<comfy_table::Cell> = Vec::new();
    // Node xname table cell
    row.push(
      comfy_table::Cell::new(xname.clone())
        .set_alignment(comfy_table::CellAlignment::Center),
    );
    // User hw components table cell
    for hw_component in &all_hw_component_vec {
      if user_defined_hw_componet_vec.contains(hw_component)
        && node_pattern_hashmap.contains_key(hw_component)
      {
        let counter = node_pattern_hashmap.get(hw_component).unwrap();
        row.push(
          comfy_table::Cell::new(format!("✅ ({})", counter,))
            .fg(Color::Green)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else if node_pattern_hashmap.contains_key(hw_component) {
        let counter = node_pattern_hashmap.get(hw_component).unwrap();
        row.push(
          comfy_table::Cell::new(format!("⚠️  ({})", counter)) // NOTE: emojis
            // can also be printed using unicode like \u{26A0}
            .fg(Color::Yellow)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else {
        // node does not contain hardware but it was requested by the user
        row.push(
          comfy_table::Cell::new("❌".to_string())
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      }
    }

    // Node score table cell
    let node_score = hsm_score_vec
      .iter()
      .find(|(node_name, _)| node_name.eq(xname))
      .unwrap_or(&(xname.to_string(), 0f32))
      .1;
    let node_score_table_cell = if node_score <= 0f32 {
      comfy_table::Cell::new(node_score)
        .set_alignment(comfy_table::CellAlignment::Center)
        .fg(Color::Red)
    } else {
      comfy_table::Cell::new(node_score)
        .set_alignment(comfy_table::CellAlignment::Center)
        .fg(Color::Green)
    };
    row.push(node_score_table_cell);
    table.add_row(row);
  }

  log::info!("\n{table}\n");
}

pub async fn get_hsm_node_hw_component_counter(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  user_defined_hw_component_vec: &[String],
  hsm_group_member_vec: &[String],
  mem_lcm: u64,
) -> Vec<(String, HashMap<String, usize>)> {
  // Get HSM group members hw configurfation based on user input

  let start = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(5)); // CSM 1.3.1 higher
                                         // number of concurrent tasks won't make it faster

  // Calculate HSM group hw component counters
  // List of node hw component counters belonging to target hsm group
  let mut target_hsm_node_hw_component_count_vec = Vec::new();

  // Get HW inventory details for parent HSM group
  for hsm_member in hsm_group_member_vec.to_owned() {
    let shasta_token_string = shasta_token.to_string(); // TODO: make it static
    let user_defined_hw_component_vec =
      user_defined_hw_component_vec.to_owned();
    let backend_clone = backend.clone();

    let permit = Arc::clone(&sem).acquire_owned().await;

    // println!("user_defined_hw_profile_vec_aux: {:?}", user_defined_hw_profile_vec_aux);
    tasks.spawn(async move {
      let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885

      get_node_hw_component_count(
        backend_clone,
        shasta_token_string,
        &hsm_member,
        user_defined_hw_component_vec,
      )
      .await
    });
  }

  while let Some(message) = tasks.join_next().await {
    if let Ok(mut node_hw_component_vec_tuple) = message {
      node_hw_component_vec_tuple.1.sort();

      let mut node_hw_component_count_hashmap: HashMap<String, usize> =
        HashMap::new();

      for node_hw_property_vec in node_hw_component_vec_tuple.1 {
        let count = node_hw_component_count_hashmap
          .entry(node_hw_property_vec)
          .or_insert(0);
        *count += 1;
      }

      let node_memory_total_capacity: u64 =
        node_hw_component_vec_tuple.2.iter().sum();

      node_hw_component_count_hashmap.insert(
        "memory".to_string(),
        (node_memory_total_capacity / mem_lcm)
          .try_into()
          .unwrap_or(0),
      );

      target_hsm_node_hw_component_count_vec.push((
        node_hw_component_vec_tuple.0,
        node_hw_component_count_hashmap,
      ));
    } else {
      log::error!("Failed procesing/fetching node hw information");
    }
  }

  let duration = start.elapsed();
  log::info!("Time elapsed to calculate hw components is: {:?}", duration);

  target_hsm_node_hw_component_count_vec
}
