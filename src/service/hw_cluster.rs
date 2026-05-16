//! Hardware cluster pin/unpin and hw-component add/delete service logic.

use std::{collections::HashMap, sync::Arc, time::Instant};

use comfy_table::Color;
use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::{group::GroupTrait, hardware_inventory::HardwareInventory},
  types::Group,
};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

/// LCM (Least Common Multiple) used to normalise memory capacity values.
/// Memory DIMMs come in multiples of 16 GiB (16384 MiB).
const MEMORY_CAPACITY_LCM: u64 = 16384;

/// Maximum number of concurrent hardware component queries.
const HW_COMPONENT_CONCURRENCY_LIMIT: usize = 5;

// ── Public types ────────────────────────────────────────────────────────────

pub use crate::shared::params::hw_cluster::HwClusterMode;

/// A list of nodes paired with their per-component counts.
pub type NodeHwCountVec = Vec<(String, HashMap<String, usize>)>;

/// Result of an `add hw-component` operation.
pub struct AddHwResult {
  pub nodes_moved: Vec<String>,
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

/// Result of a `delete hw-component` operation.
pub struct DeleteHwResult {
  pub nodes_moved: Vec<String>,
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

/// Result of an `apply hw-configuration` (pin/unpin) operation.
pub struct ApplyHwResult {
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

// ── Utility functions ────────────────────────────────────────────────────────

/// Compute a scarcity score for each hardware component type across all nodes.
pub async fn calculate_hw_component_scarcity_scores(
  hsm_node_hw_component_count: &[(String, HashMap<String, usize>)],
) -> HashMap<String, f32> {
  let total_num_hw_components: usize = hsm_node_hw_component_count
    .iter()
    .flat_map(|(_, hw_component_qty_hashmap)| hw_component_qty_hashmap.values())
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

  tracing::info!(
    "Hw component scarcity scores: {:?}",
    hw_component_scarcity_score_hashmap
  );

  hw_component_scarcity_score_hashmap
}

/// Calculates a normalised score for each node based on component scarcity.
pub fn calculate_hsm_node_scores_from_final_hsm(
  parent_hsm_node_hw_component_count_vec: &[(String, HashMap<String, usize>)],
  parent_hsm_hw_component_summary_hashmap: &HashMap<String, usize>,
  final_hsm_summary_hashmap: &HashMap<String, usize>,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f32>,
) -> Vec<(String, f32)> {
  let mut node_score_vec: Vec<(String, f32)> = Vec::new();

  for (xname, hw_component_count) in parent_hsm_node_hw_component_count_vec {
    let mut node_score: f32 = 0.0;
    for (hw_component, qty) in hw_component_count {
      let scarcity_score = hw_component_scarcity_scores_hashmap
        .get(hw_component)
        .copied()
        .unwrap_or(0.0);

      if final_hsm_summary_hashmap.get(hw_component).is_none() {
        node_score -= scarcity_score * *qty as f32;
      } else {
        let final_qty = final_hsm_summary_hashmap
          .get(hw_component)
          .copied()
          .unwrap_or(0);
        let parent_qty = parent_hsm_hw_component_summary_hashmap
          .get(hw_component)
          .copied()
          .unwrap_or(0);

        if final_qty < parent_qty {
          node_score += scarcity_score * *qty as f32;
        } else {
          node_score -= scarcity_score * *qty as f32;
        }
      }
    }
    node_score_vec.push((xname.to_string(), node_score));
  }

  node_score_vec
}

/// Check whether further iteration is needed to satisfy the target hw pattern.
pub fn keep_iterating_final_hsm(
  hsm_final_hw_component_summary_hashmap: &HashMap<String, usize>,
  hsm_current_hw_component_summary_hashmap: &HashMap<String, usize>,
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

/// Aggregate per-node hardware counters into a single summary map.
pub fn calculate_hsm_hw_component_summary(
  target_hsm_group_node_hw_component_vec: &[(String, HashMap<String, usize>)],
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

/// Returns properties from a hardware inventory value matching the given pattern.
fn get_node_hw_properties_from_value(
  node_hw_inventory_value: &Value,
  hw_component_pattern_list: &[String],
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

  let memory_vec =
    common::hw_inventory_utils::get_list_memory_capacity_from_hw_inventory_value(
      node_hw_inventory_value,
    )
    .unwrap_or_default();

  (node_hw_component_pattern_vec, memory_vec)
}

/// Fetch hw inventory for a single node.
async fn get_node_hw_component_count(
  backend: StaticBackendDispatcher,
  shasta_token: String,
  hsm_member: &str,
  user_defined_hw_profile_vec: Vec<String>,
) -> (String, Vec<String>, Vec<u64>) {
  let node_hw_inventory_value = match backend
    .get_inventory_hardware_query(
      &shasta_token,
      hsm_member,
      None,
      None,
      None,
      None,
      None,
    )
    .await
  {
    Ok(value) => value,
    Err(e) => {
      tracing::error!("Failed to get hw inventory for '{}': {}", hsm_member, e);
      return (hsm_member.to_string(), Vec::new(), Vec::new());
    }
  };

  let node_hw_profile = get_node_hw_properties_from_value(
    &node_hw_inventory_value,
    &user_defined_hw_profile_vec,
  );

  (hsm_member.to_string(), node_hw_profile.0, node_hw_profile.1)
}

/// Print a table of node hardware component scores with color-coded cells.
pub fn print_table_f32_score(
  user_defined_hw_component_vec: &[String],
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
    [hsm_hw_component_vec, user_defined_hw_component_vec.to_vec()].concat();

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
    let mut row: Vec<comfy_table::Cell> = Vec::new();
    row.push(
      comfy_table::Cell::new(xname.clone())
        .set_alignment(comfy_table::CellAlignment::Center),
    );
    for hw_component in &all_hw_component_vec {
      if user_defined_hw_component_vec.contains(hw_component)
        && node_pattern_hashmap.contains_key(hw_component)
      {
        let counter =
          node_pattern_hashmap.get(hw_component).copied().unwrap_or(0);
        row.push(
          comfy_table::Cell::new(format!("\u{1F7E2} ({})", counter))
            .fg(Color::Green)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else if node_pattern_hashmap.contains_key(hw_component) {
        let counter =
          node_pattern_hashmap.get(hw_component).copied().unwrap_or(0);
        row.push(
          comfy_table::Cell::new(format!("\u{1F7E1} ({})", counter))
            .fg(Color::Yellow)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else {
        row.push(
          comfy_table::Cell::new("\u{1F534}".to_string())
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      }
    }

    let default_score = (xname.to_string(), 0f32);
    let node_score = hsm_score_vec
      .iter()
      .find(|(node_name, _)| node_name.eq(xname))
      .unwrap_or(&default_score)
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

  tracing::info!("\n{table}\n");
}

/// Fetch hardware inventory for HSM group members and return per-node component counters.
pub async fn get_hsm_node_hw_component_counter(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  user_defined_hw_component_vec: &[String],
  hsm_group_member_vec: &[String],
  mem_lcm: u64,
) -> Vec<(String, HashMap<String, usize>)> {
  let start = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(HW_COMPONENT_CONCURRENCY_LIMIT));

  let mut target_hsm_node_hw_component_count_vec = Vec::new();

  for hsm_member in hsm_group_member_vec {
    let shasta_token_string = shasta_token.to_string();
    let user_defined_hw_component_vec =
      user_defined_hw_component_vec.to_owned();
    let backend_clone = backend.clone();
    let hsm_member = hsm_member.clone();

    let permit = Arc::clone(&sem).acquire_owned().await;

    tasks.spawn(async move {
      let _permit = permit;

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
      tracing::error!("Failed processing/fetching node hw information");
    }
  }

  let duration = start.elapsed();
  tracing::info!("Time elapsed to calculate hw components is: {:?}", duration);

  target_hsm_node_hw_component_count_vec
}

/// Selects the best candidate node by highest score, breaking ties by xname.
pub fn get_best_candidate_in_hsm(
  hsm_score_vec: &mut [(String, f32)],
  hsm_hw_component_vec: &[(String, HashMap<String, usize>)],
) -> Option<((String, f32), HashMap<String, usize>)> {
  if hsm_score_vec.is_empty() || hsm_hw_component_vec.is_empty() {
    return None;
  }

  hsm_score_vec.sort_by(|a, b| {
    b.1
      .partial_cmp(&a.1)
      .unwrap_or(std::cmp::Ordering::Equal)
      .then(a.0.cmp(&b.0))
  });

  let best_candidate: (String, f32) = hsm_score_vec.first()?.clone();

  hsm_hw_component_vec
    .iter()
    .find(|(node, _)| node.eq(&best_candidate.0))
    .map(|best_candidate_hw| (best_candidate, best_candidate_hw.1.clone()))
}

/// For PIN mode: selects best candidate preferring existing target nodes first.
pub fn get_best_candidate_in_target_and_parent_hsm(
  target_hsm_node_score_tuple_vec: &mut [(String, f32)],
  parent_hsm_node_score_tuple_vec: &mut [(String, f32)],
  target_hsm_node_hw_component_count_vec: &mut [(
    String,
    HashMap<String, usize>,
  )],
  parent_hsm_node_hw_component_count_vec: &[(String, HashMap<String, usize>)],
) -> Option<((String, f32), HashMap<String, usize>)> {
  let target_best_candidate_tuple = get_best_candidate_in_hsm(
    target_hsm_node_score_tuple_vec,
    target_hsm_node_hw_component_count_vec,
  );

  let parent_best_candidate_tuple = get_best_candidate_in_hsm(
    parent_hsm_node_score_tuple_vec,
    parent_hsm_node_hw_component_count_vec,
  );

  if target_best_candidate_tuple.is_some() {
    target_best_candidate_tuple
  } else if parent_best_candidate_tuple.is_some() {
    parent_best_candidate_tuple
  } else {
    None
  }
}

/// Resolves a hardware description pattern into concrete xnames.
/// Returns (new_target, remaining_parent).
pub async fn resolve_hw_description_to_xnames(
  mode: HwClusterMode,
  mut target_hsm_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )>,
  mut parent_hsm_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )>,
  user_defined_target_hsm_hw_component_count_hashmap: HashMap<String, usize>,
) -> Result<
  (
    Vec<(String, HashMap<String, usize>)>,
    Vec<(String, HashMap<String, usize>)>,
  ),
  Error,
> {
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

  let hw_component_scarcity_scores_hashmap: HashMap<String, f32> =
    calculate_hw_component_scarcity_scores(
      &combined_target_parent_hsm_node_hw_component_count_vec,
    )
    .await;

  let mut final_combined_target_parent_hsm_hw_component_summary =
    user_defined_target_hsm_hw_component_count_hashmap.clone();

  for (hw_component, qty) in
    combined_target_parent_hsm_hw_component_summary_hashmap
  {
    final_combined_target_parent_hsm_hw_component_summary
      .entry(hw_component)
      .and_modify(|current_qty| *current_qty = qty - *current_qty);
  }

  let hw_component_counters_to_move_out_from_combined_hsm =
    match mode {
      HwClusterMode::Pin => {
        calculate_target_hsm_pin(
          &final_combined_target_parent_hsm_hw_component_summary,
          &final_combined_target_parent_hsm_hw_component_summary
            .keys()
            .cloned()
            .collect::<Vec<String>>(),
          &mut combined_target_parent_hsm_node_hw_component_count_vec,
          &mut target_hsm_node_hw_component_count_vec,
          &mut parent_hsm_node_hw_component_count_vec,
          &hw_component_scarcity_scores_hashmap,
        )?
      }
      HwClusterMode::Unpin => {
        calculate_target_hsm_unpin(
          &final_combined_target_parent_hsm_hw_component_summary,
          &final_combined_target_parent_hsm_hw_component_summary
            .keys()
            .cloned()
            .collect::<Vec<String>>(),
          &mut combined_target_parent_hsm_node_hw_component_count_vec,
          &hw_component_scarcity_scores_hashmap,
        )?
      }
    };

  let new_target_hsm_node_hw_component_count_vec =
    hw_component_counters_to_move_out_from_combined_hsm;

  Ok((
    new_target_hsm_node_hw_component_count_vec,
    combined_target_parent_hsm_node_hw_component_count_vec,
  ))
}

/// Parse a hardware pattern string like `"a100:4:epyc:10"` into component names
/// and a hashmap of `{component -> isize count}`.
pub fn parse_hw_pattern(
  pattern_elements: &[&str],
) -> Result<(Vec<String>, HashMap<String, isize>), Error> {
  if !pattern_elements.len().is_multiple_of(2) {
    return Err(Error::InvalidPattern(
      "Error in pattern: odd number of elements \
       after group name. Expected pairs of \
       <hw component>:<count>. \
       eg tasna:a100:4:epyc:10:instinct:8"
        .to_string(),
    ));
  }

  let mut hw_component_count: HashMap<String, isize> = HashMap::new();

  for chunk in pattern_elements.chunks_exact(2) {
    if let Ok(count) = chunk[1].parse::<isize>() {
      hw_component_count.insert(chunk[0].to_string(), count);
    } else {
      return Err(Error::InvalidPattern(
        "Error in pattern. Please make sure to \
         follow <hsm name>:<hw component>:\
         <counter>:... \
         eg <tasna>:a100:4:epyc:10:instinct:8"
          .to_string(),
      ));
    }
  }

  let mut hw_component_vec: Vec<String> =
    hw_component_count.keys().cloned().collect();
  hw_component_vec.sort();

  Ok((hw_component_vec, hw_component_count))
}

/// Fetch HSM group members, compute per-node hw component counts, and return
/// the member list, per-node counts, and group summary.
pub async fn fetch_hsm_hw_inventory(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hw_components: &[String],
  group_name: &str,
  mem_lcm: u64,
) -> Result<(Vec<String>, NodeHwCountVec, HashMap<String, usize>), Error> {
  let member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(shasta_token, &[group_name.to_string()])
    .await
    .map_err(|e| {
      Error::NotFound(format!(
        "Failed to get members from HSM group '{}': {e}",
        group_name
      ))
    })?;

  let mut node_hw_count_vec = get_hsm_node_hw_component_counter(
    backend,
    shasta_token,
    hw_components,
    &member_vec,
    mem_lcm,
  )
  .await;

  node_hw_count_vec.sort_by(|a, b| a.0.cmp(&b.0));

  let summary = calculate_hsm_hw_component_summary(&node_hw_count_vec);

  Ok((member_vec, node_hw_count_vec, summary))
}

// ── Pin algorithm ────────────────────────────────────────────────────────────

/// Node selection algorithm for PIN mode — keeps as many existing target nodes
/// as possible, pulling from parent only when needed.
pub fn calculate_target_hsm_pin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>,
  user_defined_hw_component_vec: &[String],
  combination_target_parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  target_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f32>,
) -> Result<NodeHwCountVec, Error> {
  let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<
    String,
    usize,
  > = calculate_hsm_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );
  let target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(target_hsm_node_hw_component_count_vec);
  let parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(parent_hsm_node_hw_component_count_vec);

  let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
    calculate_hsm_node_scores_from_final_hsm(
      target_hsm_node_hw_component_count_vec,
      &target_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

  let mut parent_hsm_node_score_tuple_vec: Vec<(String, f32)> =
    calculate_hsm_node_scores_from_final_hsm(
      parent_hsm_node_hw_component_count_vec,
      &parent_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

  let mut group_target_hsm_node_by_score_hashmap: HashMap<usize, Vec<String>> =
    HashMap::new();
  for (node, score) in &target_hsm_node_score_tuple_vec {
    group_target_hsm_node_by_score_hashmap
      .entry(*score as usize)
      .and_modify(|node_vec| node_vec.push(node.to_string()))
      .or_insert(vec![node.clone()]);
  }

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
    .ok_or_else(|| Error::InsufficientResources("No best candidate found.".to_string()))?;

  let mut work_to_do = keep_iterating_final_hsm(
    user_defined_hsm_hw_components_count_hashmap,
    &combination_target_parent_hsm_hw_component_summary_hashmap,
  );

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

    print_table_f32_score(
      user_defined_hw_component_vec,
      target_hsm_node_hw_component_count_vec,
      &target_hsm_node_score_tuple_vec,
    );

    print_table_f32_score(
      user_defined_hw_component_vec,
      parent_hsm_node_hw_component_count_vec,
      &parent_hsm_node_score_tuple_vec,
    );

    nodes_migrated_from_combination_target_parent_hsm
      .push((best_candidate.0.clone(), best_candidate_counters.clone()));

    combination_target_parent_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    target_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    parent_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    if combination_target_parent_hsm_node_hw_component_count_vec.is_empty() {
      break;
    }

    combination_target_parent_hsm_hw_component_summary_hashmap =
      calculate_hsm_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    target_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
      calculate_hsm_node_scores_from_final_hsm(
        target_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    let mut parent_hsm_node_score_tuple_vec: Vec<(String, f32)> =
      calculate_hsm_node_scores_from_final_hsm(
        parent_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

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
      .ok_or_else(|| Error::InsufficientResources("No best candidate found.".to_string()))?;

    work_to_do = keep_iterating_final_hsm(
      user_defined_hsm_hw_components_count_hashmap,
      &combination_target_parent_hsm_hw_component_summary_hashmap,
    );

    iter += 1;
  }

  tracing::info!("----- FINAL RESULT -----");
  tracing::info!("No candidates found");

  print_table_f32_score(
    user_defined_hw_component_vec,
    target_hsm_node_hw_component_count_vec,
    &target_hsm_node_score_tuple_vec,
  );

  print_table_f32_score(
    user_defined_hw_component_vec,
    parent_hsm_node_hw_component_count_vec,
    &parent_hsm_node_score_tuple_vec,
  );

  Ok(nodes_migrated_from_combination_target_parent_hsm)
}

// ── Unpin algorithm ──────────────────────────────────────────────────────────

/// Node selection algorithm for UNPIN mode — merges target and parent, then
/// selects nodes to move back to parent.
pub fn calculate_target_hsm_unpin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>,
  user_defined_hw_component_vec: &[String],
  combination_target_parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f32>,
) -> Result<NodeHwCountVec, Error> {
  let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<
    String,
    usize,
  > = calculate_hsm_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );

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

  let (mut best_candidate, mut best_candidate_counters) =
    get_best_candidate_in_hsm(
      &mut combination_target_parent_hsm_node_score_tuple_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| Error::InsufficientResources("No best candidate found.".to_string()))?;

  let mut work_to_do = keep_iterating_final_hsm(
    user_defined_hsm_hw_components_count_hashmap,
    &combination_target_parent_hsm_hw_component_summary_hashmap,
  );

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

    print_table_f32_score(
      user_defined_hw_component_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
      &combination_target_parent_hsm_node_score_tuple_vec,
    );

    nodes_migrated_from_combination_target_parent_hsm
      .push((best_candidate.0.clone(), best_candidate_counters.clone()));

    combination_target_parent_hsm_node_hw_component_count_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    if combination_target_parent_hsm_node_hw_component_count_vec.is_empty() {
      break;
    }

    combination_target_parent_hsm_hw_component_summary_hashmap =
      calculate_hsm_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    combination_target_parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
      calculate_hsm_node_scores_from_final_hsm(
        combination_target_parent_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    (best_candidate, best_candidate_counters) = get_best_candidate_in_hsm(
      &mut target_hsm_node_score_tuple_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| Error::InsufficientResources("No best candidate found.".to_string()))?;

    work_to_do = keep_iterating_final_hsm(
      user_defined_hsm_hw_components_count_hashmap,
      &combination_target_parent_hsm_hw_component_summary_hashmap,
    );

    iter += 1;
  }

  tracing::info!("----- FINAL RESULT -----");
  tracing::info!("No candidates found");

  print_table_f32_score(
    user_defined_hw_component_vec,
    combination_target_parent_hsm_node_hw_component_count_vec,
    &combination_target_parent_hsm_node_score_tuple_vec,
  );

  Ok(nodes_migrated_from_combination_target_parent_hsm)
}

// ── apply_hw_configuration ───────────────────────────────────────────────────

/// Parse user pattern `"a100:4:epyc:10"` into hw component names and a hashmap
/// of `{component -> usize count}`.
fn parse_hw_pattern_usize(
  target_hsm_group_name: &str,
  pattern: &str,
) -> Result<(Vec<String>, HashMap<String, usize>), Error> {
  let pattern = format!("{}:{}", target_hsm_group_name, pattern);
  tracing::info!("pattern: {}", pattern);

  let pattern_lowercase = pattern.to_lowercase();

  let (_group_name, pattern_hw_component) =
    pattern_lowercase.split_once(':').ok_or_else(|| {
      Error::InvalidPattern(
        "Invalid pattern format: \
         expected 'group:component:count'"
          .to_string(),
      )
    })?;

  let pattern_element_vec: Vec<&str> =
    pattern_hw_component.split(':').collect();

  if !pattern_element_vec.len().is_multiple_of(2) {
    return Err(Error::InvalidPattern(
      "Error in pattern: odd number of elements. \
       Expected pairs of <hw component>:<count>. \
       eg a100:4:epyc:10:instinct:8"
        .to_string(),
    ));
  }

  let mut hw_component_count: HashMap<String, usize> = HashMap::new();

  for chunk in pattern_element_vec.chunks_exact(2) {
    if let Ok(count) = chunk[1].parse::<usize>() {
      hw_component_count.insert(chunk[0].to_string(), count);
    } else {
      return Err(Error::InvalidPattern(
        "Error in pattern. Please make sure to follow \
         <hsm name>:<hw component>:<counter>:... \
         eg <tasna>:a100:4:epyc:10:instinct:8"
          .to_string(),
      ));
    }
  }

  tracing::info!(
    "User defined hw components with counters: {:?}",
    hw_component_count
  );

  let mut hw_component_vec: Vec<String> =
    hw_component_count.keys().cloned().collect();
  hw_component_vec.sort();

  Ok((hw_component_vec, hw_component_count))
}

/// Ensure the target HSM group exists, creating it if `create_target_hsm_group` is set.
async fn ensure_target_group_exists(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
) -> Result<(), Error> {
  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      tracing::debug!("Target HSM group '{}' exists, good.", target_hsm_group_name);
      Ok(())
    }
    Err(_) => {
      if !create_target_hsm_group {
        return Err(Error::NotFound(format!(
          "Target HSM group '{}' does not exist, \
           but the option to create the group was \
           NOT specified, cannot continue.",
          target_hsm_group_name,
        )));
      }
      tracing::info!(
        "Target HSM group '{}' does not exist, \
         but the option to create the group has \
         been selected, creating it now.",
        target_hsm_group_name
      );
      if dryrun {
        return Err(Error::BadRequest(
          "Dryrun selected, cannot create the \
           new group and continue."
            .to_string(),
        ));
      }
      let group = Group {
        label: target_hsm_group_name.to_string(),
        description: None,
        tags: None,
        members: None,
        exclusive_group: Some("false".to_string()),
      };
      let _ = backend
        .add_group(shasta_token, group)
        .await
        .map_err(|e| {
          Error::BadRequest(format!(
            "Unable to create new target HSM group: {e}"
          ))
        })?;
      Ok(())
    }
  }
}

/// Validate that combined target+parent resources can fulfil the user request.
fn validate_resource_sufficiency(
  target_hw: &[(String, HashMap<String, usize>)],
  parent_hw: &[(String, HashMap<String, usize>)],
  requested: &HashMap<String, usize>,
) -> Result<(), Error> {
  let mut combined = parent_hw.to_vec();
  for elem in target_hw {
    if !parent_hw.iter().any(|(xname, _)| xname.eq(&elem.0)) {
      combined.push(elem.clone());
    }
  }

  let combined_summary = calculate_hsm_hw_component_summary(&combined);

  for (hw_component, qty) in requested {
    if combined_summary
      .get(hw_component)
      .is_none_or(|value| value < qty)
    {
      return Err(Error::InsufficientResources(
        "There are not enough resources \
         to fulfil user request."
          .to_string(),
      ));
    }
  }

  Ok(())
}

/// Apply group membership updates to both target and parent HSM groups.
#[allow(clippy::too_many_arguments)]
async fn apply_group_updates(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_group: &str,
  parent_group: &str,
  old_target_members: &[String],
  old_parent_members: &[String],
  new_target_members: &[String],
  new_parent_members: &[String],
  dryrun: bool,
  delete_empty_parent: bool,
) -> Result<(), Error> {
  tracing::info!("Updating target HSM group '{}' members", target_group);
  if dryrun {
    tracing::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    backend
      .update_group_members(
        shasta_token,
        target_group,
        &old_target_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
        &new_target_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      )
      .await
      .map_err(|e| {
        Error::BadRequest(format!(
          "Failed to update target HSM group members: {e}"
        ))
      })?;
  }

  tracing::info!("Updating parent HSM group '{}' members", parent_group);
  if dryrun {
    tracing::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    let parent_will_be_empty =
      old_target_members.len() == old_parent_members.len();
    backend
      .update_group_members(
        shasta_token,
        parent_group,
        &old_parent_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
        &new_parent_members
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      )
      .await
      .map_err(|e| {
        Error::BadRequest(format!(
          "Failed to update parent HSM group members: {e}"
        ))
      })?;

    if parent_will_be_empty && delete_empty_parent {
      tracing::info!(
        "Parent HSM group '{}' is now empty and \
         the option to delete empty groups has \
         been selected, removing it.",
        parent_group
      );
      match backend.delete_group(shasta_token, parent_group).await {
        Ok(_) => tracing::info!("HSM group removed successfully."),
        Err(e) => tracing::debug!(
          "Error removing the HSM group. \
           This always fails, ignore please. \
           Reported: {}",
          e
        ),
      };
    } else if parent_will_be_empty {
      tracing::debug!(
        "Parent HSM group '{}' is now empty and \
         the option to delete empty groups has \
         NOT been selected, will not remove it.",
        parent_group
      )
    }
  }

  Ok(())
}

/// Core logic for hardware cluster pin/unpin — no terminal interaction.
#[allow(clippy::too_many_arguments)]
pub async fn apply_hw_configuration(
  backend: &StaticBackendDispatcher,
  mode: HwClusterMode,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
  delete_empty_parent_hsm_group: bool,
) -> Result<ApplyHwResult, Error> {
  let (user_defined_hw_component_vec, user_defined_hw_component_count_hashmap) =
    parse_hw_pattern_usize(target_hsm_group_name, pattern)?;

  ensure_target_group_exists(
    backend,
    shasta_token,
    target_hsm_group_name,
    dryrun,
    create_target_hsm_group,
  )
  .await?;

  let (
    target_hsm_group_member_vec,
    target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_hw_component_vec,
    target_hsm_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  tracing::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary
  );

  let (
    parent_hsm_group_member_vec,
    parent_hsm_node_hw_component_count_vec,
    _parent_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_hw_component_vec,
    parent_hsm_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  validate_resource_sufficiency(
    &target_hsm_node_hw_component_count_vec,
    &parent_hsm_node_hw_component_count_vec,
    &user_defined_hw_component_count_hashmap,
  )?;

  let (
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
  ) = resolve_hw_description_to_xnames(
    mode,
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
    user_defined_hw_component_count_hashmap,
  )
  .await?;

  let target_hsm_node_vec: Vec<String> = target_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect();

  let parent_hsm_node_vec: Vec<String> = parent_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect();

  apply_group_updates(
    backend,
    shasta_token,
    target_hsm_group_name,
    parent_hsm_group_name,
    &target_hsm_group_member_vec,
    &parent_hsm_group_member_vec,
    &target_hsm_node_vec,
    &parent_hsm_node_vec,
    dryrun,
    delete_empty_parent_hsm_group,
  )
  .await?;

  Ok(ApplyHwResult {
    target_nodes: target_hsm_node_vec,
    parent_nodes: parent_hsm_node_vec,
  })
}

// ── add_hw_component ─────────────────────────────────────────────────────────

/// Ensure the target HSM group exists for add-hw-component, creating it if needed.
async fn ensure_add_target_group_exists(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  create_hsm_group: bool,
) -> Result<(), Error> {
  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      tracing::debug!("The group '{}' exists, good.", target_hsm_group_name);
      Ok(())
    }
    Err(_) => {
      if !create_hsm_group {
        return Err(Error::NotFound(format!(
          "Group '{}' does not exist, but the \
           option to create the group was NOT \
           specified, cannot continue.",
          target_hsm_group_name
        )));
      }
      tracing::info!(
        "Group '{}' does not exist, but the option \
         to create the group has been selected, \
         creating it now.",
        target_hsm_group_name
      );
      if dryrun {
        return Err(Error::BadRequest(
          "Dryrun selected, cannot create \
           the new group and continue."
            .to_string(),
        ));
      }
      let group = Group {
        label: target_hsm_group_name.to_string(),
        description: None,
        tags: None,
        members: None,
        exclusive_group: Some("false".to_string()),
      };
      backend.add_group(shasta_token, group).await?;
      Ok(())
    }
  }
}

/// Compute the final parent HSM hw component summary after subtracting user-requested deltas.
fn compute_final_parent_summary(
  current_summary: &HashMap<String, usize>,
  deltas: &HashMap<String, isize>,
  parent_group_name: &str,
) -> Result<HashMap<String, usize>, Error> {
  let mut final_summary: HashMap<String, usize> = HashMap::new();

  for (hw_component, counter) in deltas {
    let current = *current_summary.get(hw_component).unwrap_or(&0);
    if *counter > current as isize {
      return Err(Error::InsufficientResources(format!(
        "Cannot remove more hw component '{}' \
         ({}) than available in parent group \
         '{}' ({})",
        hw_component, *counter, parent_group_name, current
      )));
    }
    let new_counter = current - *counter as usize;
    final_summary.insert(hw_component.to_string(), new_counter);
  }

  Ok(final_summary)
}

/// Core logic for adding hardware components to a cluster group.
/// No terminal interaction — suitable for both CLI and HTTP callers.
pub async fn add_hw_component(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_hsm_group: bool,
) -> Result<AddHwResult, Error> {
  ensure_add_target_group_exists(
    backend,
    shasta_token,
    target_hsm_group_name,
    dryrun,
    create_hsm_group,
  )
  .await?;

  let pattern_str = format!("{}:{}", target_hsm_group_name, pattern);
  let pattern_lowercase = pattern_str.to_lowercase();
  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();
  let target_name = pattern_element_vec.remove(0);

  let (
    user_defined_delta_hw_component_vec,
    user_defined_delta_hw_component_count_hashmap,
  ) = parse_hw_pattern(&pattern_element_vec)?;

  let (
    _parent_member_vec,
    mut parent_hsm_node_hw_component_count_vec,
    parent_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_delta_hw_component_vec,
    parent_hsm_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  let final_parent_hsm_hw_component_summary = compute_final_parent_summary(
    &parent_hsm_hw_component_summary,
    &user_defined_delta_hw_component_count_hashmap,
    parent_hsm_group_name,
  )?;

  let scarcity_scores = calculate_hw_component_scarcity_scores(
    &parent_hsm_node_hw_component_count_vec,
  )
  .await;

  let hw_counters_to_move =
    calculate_target_hsm_unpin(
      &final_parent_hsm_hw_component_summary,
      &final_parent_hsm_hw_component_summary
        .keys()
        .cloned()
        .collect::<Vec<String>>(),
      &mut parent_hsm_node_hw_component_count_vec,
      &scarcity_scores,
    )?;

  let nodes_to_move: Vec<String> = hw_counters_to_move
    .iter()
    .map(|(xname, _)| xname.clone())
    .collect();

  let mut target_hsm_node_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      &[target_name.to_string()],
    )
    .await?;

  target_hsm_node_vec.extend(nodes_to_move.clone());
  target_hsm_node_vec.sort();

  if !dryrun {
    for xname in &nodes_to_move {
      backend
        .delete_member_from_group(shasta_token, parent_hsm_group_name, xname)
        .await?;

      let _ = backend
        .add_members_to_group(
          shasta_token,
          target_name,
          &[xname.as_str()],
        )
        .await?;
    }
  }

  let parent_nodes: Vec<String> = parent_hsm_node_hw_component_count_vec
    .iter()
    .map(|(xname, _)| xname.clone())
    .collect();

  Ok(AddHwResult {
    nodes_moved: nodes_to_move,
    target_nodes: target_hsm_node_vec,
    parent_nodes,
  })
}

// ── delete_hw_component ──────────────────────────────────────────────────────

/// Handle the case when target HSM group is already empty.
async fn handle_empty_target(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  delete_hsm_group: bool,
) -> Result<(), Error> {
  tracing::info!(
    "The target HSM group {} is already empty, cannot \
     remove hardware from it.",
    target_hsm_group_name
  );

  if dryrun || !delete_hsm_group {
    tracing::info!(
      "The option to delete empty groups has NOT been \
       selected, or the dryrun has been enabled. We \
       are done with this action."
    );
    return Ok(());
  }

  tracing::info!(
    "The option to delete empty groups has been \
     selected, removing it."
  );
  match backend
    .delete_group(shasta_token, target_hsm_group_name)
    .await
  {
    Ok(_) => {
      tracing::info!(
        "HSM group removed successfully, we are \
         done with this action."
      );
    }
    Err(e) => tracing::debug!(
      "Error removing the HSM group. This always \
       fails, ignore please. Reported: {}",
      e
    ),
  };
  Ok(())
}

/// Compute the final target HSM hw component summary after subtracting deltas.
fn compute_delete_final_summary(
  current_summary: &HashMap<String, usize>,
  deltas: &HashMap<String, isize>,
) -> Result<HashMap<String, usize>, Error> {
  let mut final_summary: HashMap<String, usize> = HashMap::new();

  for (hw_component, counter) in deltas {
    let current = *current_summary.get(hw_component).ok_or_else(|| {
      Error::NotFound(format!(
        "hw component '{}' not found in target HSM \
           hw component summary",
        hw_component
      ))
    })?;

    final_summary.insert(hw_component.to_string(), current - *counter as usize);
  }

  Ok(final_summary)
}

/// Move nodes between HSM groups: delete from target, add to parent.
async fn apply_node_moves(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_group: &str,
  parent_group: &str,
  nodes: &[String],
  target_will_be_empty: bool,
  delete_hsm_group: bool,
) -> Result<(), Error> {
  for xname in nodes {
    backend
      .delete_member_from_group(shasta_token, target_group, xname.as_str())
      .await?;

    backend
      .add_members_to_group(shasta_token, parent_group, &[xname.as_str()])
      .await?;
  }

  if target_will_be_empty {
    if delete_hsm_group {
      tracing::info!(
        "HSM group {} is now empty and the option to \
         delete empty groups has been selected, \
         removing it.",
        target_group
      );
      match backend.delete_group(shasta_token, target_group).await {
        Ok(_) => tracing::info!("HSM group removed successfully."),
        Err(e) => tracing::debug!(
          "Error removing the HSM group. This always \
           fails, ignore please. Reported: {}",
          e
        ),
      };
    } else {
      tracing::debug!(
        "HSM group {} is now empty and the option to \
         delete empty groups has NOT been selected, \
         will not remove it.",
        target_group
      )
    }
  }

  Ok(())
}

/// Core logic for removing hardware components from a cluster group.
/// No terminal interaction — suitable for both CLI and HTTP callers.
pub async fn delete_hw_component(
  backend: &StaticBackendDispatcher,
  token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  delete_hsm_group: bool,
) -> Result<DeleteHwResult, Error> {
  match backend.get_group(token, target_hsm_group_name).await {
    Ok(_) => {}
    Err(_) => {
      return Err(Error::NotFound(format!(
        "HSM group {} does not exist, cannot remove hw from it.",
        target_hsm_group_name
      )));
    }
  }

  let pattern_str = format!("{}:{}", target_hsm_group_name, pattern);
  let pattern_lowercase = pattern_str.to_lowercase();
  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();
  let target_name = pattern_element_vec.remove(0);

  let (
    user_defined_delta_hw_component_vec,
    user_defined_delta_hw_component_count_hashmap,
  ) = parse_hw_pattern(&pattern_element_vec)?;

  let (
    target_hsm_group_member_vec,
    mut target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    token,
    &user_defined_delta_hw_component_vec,
    target_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  if target_hsm_node_hw_component_count_vec.is_empty() {
    handle_empty_target(backend, token, target_name, dryrun, delete_hsm_group).await?;
    return Ok(DeleteHwResult {
      nodes_moved: vec![],
      target_nodes: vec![],
      parent_nodes: vec![],
    });
  }

  let (
    parent_hsm_group_member_vec,
    parent_hsm_node_hw_component_count_vec,
    _parent_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    token,
    &user_defined_delta_hw_component_vec,
    parent_hsm_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  let combined = [
    target_hsm_node_hw_component_count_vec.clone(),
    parent_hsm_node_hw_component_count_vec.clone(),
  ]
  .concat();
  let scarcity_scores = calculate_hw_component_scarcity_scores(&combined).await;

  let final_target_summary =
    compute_delete_final_summary(&target_hsm_hw_component_summary, &user_defined_delta_hw_component_count_hashmap)?;

  let hw_counters_to_move =
    calculate_target_hsm_unpin(
      &final_target_summary,
      &final_target_summary.keys().cloned().collect::<Vec<String>>(),
      &mut target_hsm_node_hw_component_count_vec,
      &scarcity_scores,
    )?;

  let nodes_to_move: Vec<String> = hw_counters_to_move
    .iter()
    .map(|(xname, _)| xname.clone())
    .collect();

  let mut parent_nodes: Vec<String> = parent_hsm_group_member_vec;
  parent_nodes.extend(nodes_to_move.clone());
  parent_nodes.sort();

  let target_nodes: Vec<String> = target_hsm_node_hw_component_count_vec
    .iter()
    .map(|(xname, _)| xname.clone())
    .collect();

  if !dryrun {
    apply_node_moves(
      backend,
      token,
      target_name,
      parent_hsm_group_name,
      &nodes_to_move,
      target_hsm_group_member_vec.len() == nodes_to_move.len(),
      delete_hsm_group,
    )
    .await?;
  }

  Ok(DeleteHwResult { nodes_moved: nodes_to_move, target_nodes, parent_nodes })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  // ---- parse_hw_pattern ----

  #[test]
  fn parse_hw_pattern_valid() {
    let input = vec!["a100", "4", "epyc", "10"];
    let (names, counts) = parse_hw_pattern(&input).unwrap();
    assert_eq!(names, vec!["a100", "epyc"]);
    assert_eq!(counts.get("a100"), Some(&4));
    assert_eq!(counts.get("epyc"), Some(&10));
  }

  #[test]
  fn parse_hw_pattern_single_pair() {
    let input = vec!["instinct", "8"];
    let (names, counts) = parse_hw_pattern(&input).unwrap();
    assert_eq!(names, vec!["instinct"]);
    assert_eq!(counts.get("instinct"), Some(&8));
  }

  #[test]
  fn parse_hw_pattern_empty() {
    let input: Vec<&str> = vec![];
    let (names, counts) = parse_hw_pattern(&input).unwrap();
    assert!(names.is_empty());
    assert!(counts.is_empty());
  }

  #[test]
  fn parse_hw_pattern_odd_elements_errors() {
    let input = vec!["a100", "4", "epyc"];
    assert!(parse_hw_pattern(&input).is_err());
  }

  #[test]
  fn parse_hw_pattern_non_numeric_count_errors() {
    let input = vec!["a100", "four"];
    assert!(parse_hw_pattern(&input).is_err());
  }

  #[test]
  fn parse_hw_pattern_negative_count() {
    let input = vec!["a100", "-3"];
    let (_, counts) = parse_hw_pattern(&input).unwrap();
    assert_eq!(counts.get("a100"), Some(&-3));
  }

  #[test]
  fn parse_hw_pattern_sorted_output() {
    let input = vec!["zebra", "1", "alpha", "2", "mid", "3"];
    let (names, _) = parse_hw_pattern(&input).unwrap();
    assert_eq!(names, vec!["alpha", "mid", "zebra"]);
  }

  // ---- calculate_hsm_hw_component_summary ----

  #[test]
  fn summary_empty_input() {
    let input: Vec<(String, HashMap<String, usize>)> = vec![];
    let result = calculate_hsm_hw_component_summary(&input);
    assert!(result.is_empty());
  }

  #[test]
  fn summary_single_node() {
    let mut hw = HashMap::new();
    hw.insert("a100".to_string(), 4);
    hw.insert("epyc".to_string(), 2);
    let input = vec![("x1000c0s0b0n0".to_string(), hw)];
    let result = calculate_hsm_hw_component_summary(&input);
    assert_eq!(result.get("a100"), Some(&4));
    assert_eq!(result.get("epyc"), Some(&2));
  }

  #[test]
  fn summary_multiple_nodes() {
    let mut hw1 = HashMap::new();
    hw1.insert("a100".to_string(), 4);
    hw1.insert("epyc".to_string(), 2);
    let mut hw2 = HashMap::new();
    hw2.insert("a100".to_string(), 2);
    hw2.insert("instinct".to_string(), 8);
    let input = vec![
      ("x1000c0s0b0n0".to_string(), hw1),
      ("x1000c0s1b0n0".to_string(), hw2),
    ];
    let result = calculate_hsm_hw_component_summary(&input);
    assert_eq!(result.get("a100"), Some(&6));
    assert_eq!(result.get("epyc"), Some(&2));
    assert_eq!(result.get("instinct"), Some(&8));
  }

  // ---- keep_iterating_final_hsm ----

  #[test]
  fn keep_iterating_when_current_exceeds_final() {
    let final_summary = HashMap::from([("a100".to_string(), 4)]);
    let current_summary = HashMap::from([("a100".to_string(), 6)]);
    assert!(keep_iterating_final_hsm(&final_summary, &current_summary));
  }

  #[test]
  fn stop_iterating_when_current_equals_final() {
    let final_summary = HashMap::from([("a100".to_string(), 4)]);
    let current_summary = HashMap::from([("a100".to_string(), 4)]);
    assert!(!keep_iterating_final_hsm(&final_summary, &current_summary));
  }

  #[test]
  fn stop_iterating_when_current_below_final() {
    let final_summary = HashMap::from([("a100".to_string(), 4)]);
    let current_summary = HashMap::from([("a100".to_string(), 2)]);
    assert!(!keep_iterating_final_hsm(&final_summary, &current_summary));
  }

  #[test]
  fn stop_iterating_when_component_missing_from_current() {
    let final_summary = HashMap::from([("a100".to_string(), 4)]);
    let current_summary = HashMap::new();
    assert!(!keep_iterating_final_hsm(&final_summary, &current_summary));
  }

  #[test]
  fn keep_iterating_mixed_components() {
    let final_summary =
      HashMap::from([("a100".to_string(), 4), ("epyc".to_string(), 10)]);
    let current_summary =
      HashMap::from([("a100".to_string(), 4), ("epyc".to_string(), 12)]);
    assert!(keep_iterating_final_hsm(&final_summary, &current_summary));
  }

  // ---- get_best_candidate_in_hsm ----

  #[test]
  fn best_candidate_empty_inputs() {
    let mut scores: Vec<(String, f32)> = vec![];
    let hw: Vec<(String, HashMap<String, usize>)> = vec![];
    assert!(get_best_candidate_in_hsm(&mut scores, &hw).is_none());
  }

  #[test]
  fn best_candidate_highest_score_wins() {
    let mut scores = vec![
      ("x1000c0s0b0n0".to_string(), 2.0),
      ("x1000c0s1b0n0".to_string(), 5.0),
      ("x1000c0s2b0n0".to_string(), 3.0),
    ];
    let hw = vec![
      (
        "x1000c0s0b0n0".to_string(),
        HashMap::from([("a100".to_string(), 4)]),
      ),
      (
        "x1000c0s1b0n0".to_string(),
        HashMap::from([("a100".to_string(), 2)]),
      ),
      (
        "x1000c0s2b0n0".to_string(),
        HashMap::from([("a100".to_string(), 1)]),
      ),
    ];
    let result = get_best_candidate_in_hsm(&mut scores, &hw).unwrap();
    assert_eq!(result.0.0, "x1000c0s1b0n0");
    assert_eq!(result.0.1, 5.0);
    assert_eq!(result.1.get("a100"), Some(&2));
  }

  // ---- parse_hw_pattern_usize ----

  #[test]
  fn parse_hw_pattern_usize_valid() {
    let (names, counts) =
      parse_hw_pattern_usize("tasna", "a100:4:epyc:10").unwrap();
    assert_eq!(names, vec!["a100", "epyc"]);
    assert_eq!(counts.get("a100"), Some(&4));
    assert_eq!(counts.get("epyc"), Some(&10));
  }

  #[test]
  fn parse_hw_pattern_usize_single_pair() {
    let (names, counts) =
      parse_hw_pattern_usize("group1", "instinct:8").unwrap();
    assert_eq!(names, vec!["instinct"]);
    assert_eq!(counts.get("instinct"), Some(&8));
  }

  #[test]
  fn parse_hw_pattern_usize_odd_elements_errors() {
    assert!(parse_hw_pattern_usize("g", "a100:4:epyc").is_err());
  }

  #[test]
  fn parse_hw_pattern_usize_non_numeric_count_errors() {
    assert!(parse_hw_pattern_usize("g", "a100:four").is_err());
  }

  #[test]
  fn parse_hw_pattern_usize_negative_count_errors() {
    assert!(parse_hw_pattern_usize("g", "a100:-3").is_err());
  }

  #[test]
  fn parse_hw_pattern_usize_sorted_output() {
    let (names, _) =
      parse_hw_pattern_usize("g", "zebra:1:alpha:2:mid:3").unwrap();
    assert_eq!(names, vec!["alpha", "mid", "zebra"]);
  }

  #[test]
  fn parse_hw_pattern_usize_lowercased() {
    let (names, counts) = parse_hw_pattern_usize("GROUP", "A100:4").unwrap();
    assert_eq!(names, vec!["a100"]);
    assert_eq!(counts.get("a100"), Some(&4));
  }

  // ---- validate_resource_sufficiency ----

  #[test]
  fn validate_sufficiency_passes() {
    let target_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let parent_hw = vec![(
      "x1000c0s1b0n0".to_string(),
      HashMap::from([("a100".to_string(), 8)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 10)]);
    assert!(validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_ok());
  }

  #[test]
  fn validate_sufficiency_fails_insufficient() {
    let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 2)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 10)]);
    assert!(validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_err());
  }

  #[test]
  fn validate_sufficiency_fails_missing_component() {
    let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("epyc".to_string(), 10)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 1)]);
    assert!(validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_err());
  }

  #[test]
  fn validate_sufficiency_exact_match() {
    let target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 4)]);
    assert!(validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_ok());
  }

  #[test]
  fn validate_sufficiency_combines_target_and_parent() {
    let target_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 3)]),
    )];
    let parent_hw = vec![(
      "x1000c0s1b0n0".to_string(),
      HashMap::from([("a100".to_string(), 3)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 6)]);
    assert!(validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_ok());
  }

  #[test]
  fn validate_sufficiency_no_double_count_overlap() {
    let target_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let parent_hw = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4)]),
    )];
    let requested = HashMap::from([("a100".to_string(), 5)]);
    assert!(validate_resource_sufficiency(&target_hw, &parent_hw, &requested).is_err());
  }

  // ---- resolve_hw_description_to_xnames (pin / unpin integration) ----

  #[tokio::test]
  pub async fn test_hsm_hw_management_pin_1() {
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
        HwClusterMode::Pin,
        hsm_zinal_hw_counters,
        hsm_nodes_free_hw_conters,
        user_request_hw_summary.clone(),
      )
      .await
      .unwrap();

    let target_hsm_hw_summary: HashMap<String, usize> =
      calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    let mut success = true;
    for (hw_component, qty) in user_request_hw_summary {
      if target_hsm_hw_summary.get(&hw_component).is_none()
        || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
      {
        success = false;
      }
    }

    assert!(success)
  }

  #[tokio::test]
  pub async fn test_hsm_hw_management_pin_2() {
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
        HwClusterMode::Pin,
        hsm_zinal_hw_counters.clone(),
        hsm_nodes_free_hw_conters,
        user_request_hw_summary.clone(),
      )
      .await
      .unwrap();

    let target_hsm_hw_summary: HashMap<String, usize> =
      calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    let mut success = true;
    for (hw_component, qty) in &user_request_hw_summary {
      if target_hsm_hw_summary.get(hw_component).is_none()
        || *qty > *target_hsm_hw_summary.get(hw_component).unwrap()
      {
        success = false;
      }
    }

    // Pinning: new target should maximise nodes from old target
    success = success
      && target_hsm_node_hw_component_count_vec.iter().all(
        |(new_target_xname, _)| {
          hsm_zinal_hw_counters
            .iter()
            .any(|(old_target_xname, _)| old_target_xname == new_target_xname)
        },
      );

    assert!(success)
  }

  #[tokio::test]
  pub async fn test_hsm_hw_management_unpin_1() {
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
        HwClusterMode::Unpin,
        hsm_zinal_hw_counters,
        hsm_nodes_free_hw_conters,
        user_request_hw_summary.clone(),
      )
      .await
      .unwrap();

    let target_hsm_hw_summary: HashMap<String, usize> =
      calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    let mut success = true;
    for (hw_component, qty) in user_request_hw_summary {
      if target_hsm_hw_summary.get(&hw_component).is_none()
        || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
      {
        success = false;
      }
    }

    assert!(success)
  }

  #[tokio::test]
  pub async fn test_hsm_hw_management_unpin_2() {
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
        HwClusterMode::Unpin,
        hsm_zinal_hw_counters,
        hsm_nodes_free_hw_conters,
        user_request_hw_summary.clone(),
      )
      .await
      .unwrap();

    let target_hsm_hw_summary: HashMap<String, usize> =
      calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    let mut success = true;
    for (hw_component, qty) in user_request_hw_summary {
      if target_hsm_hw_summary.get(&hw_component).is_none()
        || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
      {
        success = false;
      }
    }

    assert!(success)
  }
}
