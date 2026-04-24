use std::{collections::HashMap, sync::Arc, time::Instant};

use anyhow::Error;
use comfy_table::Color;
use manta_backend_dispatcher::interfaces::hsm::{
  group::GroupTrait, hardware_inventory::HardwareInventory,
};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

use super::command::HwClusterMode;

/// Maximum number of concurrent hardware component queries.
const HW_COMPONENT_CONCURRENCY_LIMIT: usize = 5;

/// A list of nodes with their hardware component counts.
pub type NodeHwCountVec = Vec<(String, HashMap<String, usize>)>;

/// Compute a scarcity score for each hardware component
/// type across all nodes in the HSM group.
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

/// Calculates a normalized score for each hw component
/// in HSM group based on component scarcity.
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
        // final/user request does NOT contain hw component
        // negative - current hw component counter in HSM
        // group is not requested by the user therefor we
        // should penalize this node
        node_score -= scarcity_score * *qty as f32;
      } else {
        // final/user request does contain hw component
        let final_qty = final_hsm_summary_hashmap
          .get(hw_component)
          .copied()
          .unwrap_or(0);
        let parent_qty = parent_hsm_hw_component_summary_hashmap
          .get(hw_component)
          .copied()
          .unwrap_or(0);

        if final_qty < parent_qty {
          // positive - current hw component counter in
          // parent/combined HSM group are higher than
          // final (user requested) hw component counter
          // therefore we remove this node
          node_score += scarcity_score * *qty as f32;
        } else {
          // negative - current hw component counter in
          // parent/combined HSM group is lower or equal
          // than final (user requested) hw component
          // counter therefor we should penalize this node
          node_score -= scarcity_score * *qty as f32;
        }
      }
    }
    node_score_vec.push((xname.to_string(), node_score));
  }

  node_score_vec
}

/// Check whether further iteration is needed to satisfy
/// the target hardware pattern.
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

/// Returns a triple like (<xname>, <list of hw components>,
/// <list of memory capacity>)
/// Note: list of hw components can be either the hw component
/// pattern provided by user or the description from the HSM
/// API.
/// NOTE: backend is not borrowed because we need to clone it
/// in order to use it across threads.
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

/// Aggregate per-node hardware counters into a single
/// summary map for the HSM group.
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

/// Returns the properties in hw_property_list found in the
/// node_hw_inventory_value which is HSM hardware inventory
/// API json response.
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

/// Print a table of node hardware component scores with
/// color-coded cells.
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
    // Node xname table cell
    row.push(
      comfy_table::Cell::new(xname.clone())
        .set_alignment(comfy_table::CellAlignment::Center),
    );
    // User hw components table cell
    for hw_component in &all_hw_component_vec {
      if user_defined_hw_component_vec.contains(hw_component)
        && node_pattern_hashmap.contains_key(hw_component)
      {
        let counter =
          node_pattern_hashmap.get(hw_component).copied().unwrap_or(0);
        row.push(
          comfy_table::Cell::new(format!("\u{1F7E2} ({})", counter,))
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
        // node does not contain hardware but it was
        // requested by the user
        row.push(
          comfy_table::Cell::new("\u{1F534}".to_string())
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      }
    }

    // Node score table cell
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

/// Fetch hardware inventory for HSM group members and
/// return per-node component counters.
pub async fn get_hsm_node_hw_component_counter(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  user_defined_hw_component_vec: &[String],
  hsm_group_member_vec: &[String],
  mem_lcm: u64,
) -> Vec<(String, HashMap<String, usize>)> {
  // Get HSM group members hw configuration based on user
  // input

  let start = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(HW_COMPONENT_CONCURRENCY_LIMIT)); // CSM 1.3.1 higher
  // number of concurrent tasks won't make it faster

  // Calculate HSM group hw component counters
  // List of node hw component counters belonging to target
  // hsm group
  let mut target_hsm_node_hw_component_count_vec = Vec::new();

  // Get HW inventory details for parent HSM group
  for hsm_member in hsm_group_member_vec {
    let shasta_token_string = shasta_token.to_string();
    let user_defined_hw_component_vec =
      user_defined_hw_component_vec.to_owned();
    let backend_clone = backend.clone();
    let hsm_member = hsm_member.clone();

    let permit = Arc::clone(&sem).acquire_owned().await;

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
      tracing::error!("Failed processing/fetching node hw information");
    }
  }

  let duration = start.elapsed();
  tracing::info!("Time elapsed to calculate hw components is: {:?}", duration);

  target_hsm_node_hw_component_count_vec
}

/// Selects the best candidate from the given score/hw
/// vectors. Sorts by highest score first, then by xname
/// for deterministic tie-breaking.
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

  // Get node with highest normalized score (best candidate)
  let best_candidate: (String, f32) = hsm_score_vec.first()?.clone();

  hsm_hw_component_vec
    .iter()
    .find(|(node, _)| node.eq(&best_candidate.0))
    .map(|best_candidate_hw| (best_candidate, best_candidate_hw.1.clone()))
}

/// For PIN mode: selects the best candidate preferring
/// existing target HSM nodes first, falling back to parent.
pub fn get_best_candidate_in_target_and_parent_hsm(
  target_hsm_node_score_tuple_vec: &mut [(String, f32)],
  parent_hsm_node_score_tuple_vec: &mut [(String, f32)],
  target_hsm_node_hw_component_count_vec: &mut [(
    String,
    HashMap<String, usize>,
  )],
  parent_hsm_node_hw_component_count_vec: &[(String, HashMap<String, usize>)],
) -> Option<((String, f32), HashMap<String, usize>)> {
  // Get best candidate in 'target' HSM group
  let target_best_candidate_tuple = get_best_candidate_in_hsm(
    target_hsm_node_score_tuple_vec,
    target_hsm_node_hw_component_count_vec,
  );

  // Get best candidate in 'parent' HSM group
  let parent_best_candidate_tuple = get_best_candidate_in_hsm(
    parent_hsm_node_score_tuple_vec,
    parent_hsm_node_hw_component_count_vec,
  );

  // If best candidate exists (in 'target' HSM group),
  // then use it. Otherwise, use the one in 'parent'
  if target_best_candidate_tuple.is_some() {
    target_best_candidate_tuple
  } else if parent_best_candidate_tuple.is_some() {
    parent_best_candidate_tuple
  } else {
    None
  }
}

/// Resolves a hardware description pattern into concrete
/// xnames by combining target and parent HSM groups,
/// computing scarcity scores, and applying the appropriate
/// node selection strategy (Pin or Unpin).
///
/// Returns a tuple (new_target, remaining_parent) with the
/// resolved node lists.
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
  // *******************************************************
  // CALCULATE 'COMBINED HSM' WITH TARGET HSM AND PARENT
  // HSM ELEMENTS COMBINED
  // NOTE: PARENT HSM may contain elements in TARGET HSM,
  // we need to only add those xnames which are not part
  // of PARENT HSM already

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

  // *******************************************************
  // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY

  let hw_component_scarcity_scores_hashmap: HashMap<String, f32> =
    calculate_hw_component_scarcity_scores(
      &combined_target_parent_hsm_node_hw_component_count_vec,
    )
    .await;

  // *******************************************************
  // CALCULATE FINAL HSM SUMMARY COUNTERS AFTER REMOVING
  // THE NODES THAT NEED TO GO TO TARGET HSM (SUBTRACT
  // USER INPUT SUMMARY FROM INITIAL COMBINED HSM SUMMARY)
  let mut final_combined_target_parent_hsm_hw_component_summary =
    user_defined_target_hsm_hw_component_count_hashmap.clone();

  for (hw_component, qty) in
    combined_target_parent_hsm_hw_component_summary_hashmap
  {
    final_combined_target_parent_hsm_hw_component_summary
      .entry(hw_component)
      .and_modify(|current_qty| *current_qty = qty - *current_qty);
  }

  // Calculate new target HSM group using mode-specific
  // strategy
  let hw_component_counters_to_move_out_from_combined_hsm =
    match mode {
      HwClusterMode::Pin => {
        crate::cli::commands::apply_hw_cluster_pin::utils::calculate_target_hsm_pin(
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
        crate::cli::commands::apply_hw_cluster_unpin::utils::calculate_target_hsm_unpin(
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

/// Parse a hardware pattern string like `"a100:4:epyc:10"`
/// into a sorted list of hw component names and a hashmap
/// of `{component -> count}`.
///
/// The input `pattern_elements` must contain an even number
/// of colon-separated tokens (pairs of name:count).
pub fn parse_hw_pattern(
  pattern_elements: &[&str],
) -> Result<(Vec<String>, HashMap<String, isize>), Error> {
  use anyhow::bail;

  if !pattern_elements.len().is_multiple_of(2) {
    bail!(
      "Error in pattern: odd number of elements \
       after group name. Expected pairs of \
       <hw component>:<count>. \
       eg tasna:a100:4:epyc:10:instinct:8",
    );
  }

  let mut hw_component_count: HashMap<String, isize> = HashMap::new();

  for chunk in pattern_elements.chunks_exact(2) {
    if let Ok(count) = chunk[1].parse::<isize>() {
      hw_component_count.insert(chunk[0].to_string(), count);
    } else {
      bail!(
        "Error in pattern. Please make sure to \
         follow <hsm name>:<hw component>:\
         <counter>:... \
         eg <tasna>:a100:4:epyc:10:instinct:8",
      );
    }
  }

  let mut hw_component_vec: Vec<String> =
    hw_component_count.keys().cloned().collect();
  hw_component_vec.sort();

  Ok((hw_component_vec, hw_component_count))
}

/// Fetch HSM group member xnames, compute per-node hw
/// component counts, sort by xname, and compute the group
/// summary.
pub async fn fetch_hsm_hw_inventory(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hw_components: &[String],
  group_name: &str,
  mem_lcm: u64,
) -> Result<(Vec<String>, NodeHwCountVec, HashMap<String, usize>), Error> {
  use anyhow::Context;

  let member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(shasta_token, &[group_name.to_string()])
    .await
    .with_context(|| {
      format!("Failed to get members from HSM group '{}'", group_name)
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

/// Display the hw configuration table and prompt the user
/// for confirmation.
pub fn show_solution_and_confirm(
  group_name: &str,
  hw_component_vec: &[String],
  node_hw_component_count_vec: &[(String, HashMap<String, usize>)],
  hw_component_summary: &HashMap<String, usize>,
) -> Result<(), Error> {
  use anyhow::bail;

  tracing::info!("----- SOLUTION -----");
  tracing::info!("Hw components in HSM '{}'", group_name);
  tracing::info!(
    "hsm '{}' hw component counters: {:?}",
    group_name,
    node_hw_component_count_vec
  );

  let table = crate::cli::output::hardware::get_table(
    hw_component_vec,
    node_hw_component_count_vec,
  );

  tracing::info!("\n{table}");

  let confirm_message = format!(
    "Please check and confirm new hw summary for \
     cluster '{}': {}",
    group_name,
    hw_component_summary
      .iter()
      .map(|(k, v)| format!("{}: {}", k, v))
      .collect::<Vec<_>>()
      .join(", ")
  );

  if !crate::common::user_interaction::confirm(&confirm_message, false) {
    bail!("Operation cancelled by user");
  }

  Ok(())
}

/// Print a JSON representation of an HSM group to stdout.
pub fn print_hsm_group_json(
  label: &str,
  members: &[String],
) -> Result<(), Error> {
  use anyhow::Context;

  let value = serde_json::json!({
    "label": label,
    "description": "",
    "members": members,
    "tags": []
  });

  println!(
    "{}",
    serde_json::to_string_pretty(&value).with_context(|| format!(
      "Failed to serialize HSM group '{}' to JSON",
      label
    ))?
  );

  Ok(())
}

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

  #[test]
  fn best_candidate_tie_breaking_by_xname() {
    let mut scores = vec![
      ("x1000c0s1b0n0".to_string(), 5.0),
      ("x1000c0s0b0n0".to_string(), 5.0),
    ];
    let hw = vec![
      (
        "x1000c0s0b0n0".to_string(),
        HashMap::from([("a100".to_string(), 1)]),
      ),
      (
        "x1000c0s1b0n0".to_string(),
        HashMap::from([("a100".to_string(), 2)]),
      ),
    ];
    let result = get_best_candidate_in_hsm(&mut scores, &hw).unwrap();
    assert_eq!(result.0.0, "x1000c0s0b0n0");
  }

  // ---- calculate_hsm_node_scores_from_final_hsm ----

  #[test]
  fn scores_penalize_unrequested_components() {
    let nodes = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("unwanted".to_string(), 2usize)]),
    )];
    let parent_summary = HashMap::from([("unwanted".to_string(), 2)]);
    let final_summary = HashMap::new();
    let scarcity = HashMap::from([("unwanted".to_string(), 1.0f32)]);
    let scores = calculate_hsm_node_scores_from_final_hsm(
      &nodes,
      &parent_summary,
      &final_summary,
      &scarcity,
    );
    assert_eq!(scores.len(), 1);
    assert!(scores[0].1 < 0.0);
    assert_eq!(scores[0].1, -2.0);
  }

  #[test]
  fn scores_reward_excess_components() {
    let nodes = vec![(
      "x1000c0s0b0n0".to_string(),
      HashMap::from([("a100".to_string(), 4usize)]),
    )];
    let parent_summary = HashMap::from([("a100".to_string(), 8)]);
    let final_summary = HashMap::from([("a100".to_string(), 4)]);
    let scarcity = HashMap::from([("a100".to_string(), 2.0f32)]);
    let scores = calculate_hsm_node_scores_from_final_hsm(
      &nodes,
      &parent_summary,
      &final_summary,
      &scarcity,
    );
    assert!(scores[0].1 > 0.0);
    assert_eq!(scores[0].1, 8.0);
  }

  // ---- calculate_hw_component_scarcity_scores ----

  #[tokio::test]
  async fn scarcity_scores_single_component_type() {
    let input = vec![
      (
        "x1000c0s0b0n0".to_string(),
        HashMap::from([("a100".to_string(), 4usize)]),
      ),
      (
        "x1000c0s1b0n0".to_string(),
        HashMap::from([("a100".to_string(), 2usize)]),
      ),
    ];
    let scores = calculate_hw_component_scarcity_scores(&input).await;
    // total = 6, a100 total = 6 → score = 6/6 = 1.0
    assert_eq!(scores.len(), 1);
    assert!((scores["a100"] - 1.0).abs() < f32::EPSILON);
  }

  #[tokio::test]
  async fn scarcity_scores_multiple_component_types() {
    let input = vec![
      (
        "x1000c0s0b0n0".to_string(),
        HashMap::from([
          ("a100".to_string(), 4usize),
          ("epyc".to_string(), 2usize),
        ]),
      ),
      (
        "x1000c0s1b0n0".to_string(),
        HashMap::from([("a100".to_string(), 2usize)]),
      ),
    ];
    let scores = calculate_hw_component_scarcity_scores(&input).await;
    // total = 4 + 2 + 2 = 8
    // a100 total = 6 → score = 8/6 ≈ 1.333
    // epyc total = 2 → score = 8/2 = 4.0
    assert_eq!(scores.len(), 2);
    assert!((scores["a100"] - 8.0 / 6.0).abs() < 0.001);
    assert!((scores["epyc"] - 4.0).abs() < f32::EPSILON);
  }

  #[tokio::test]
  async fn scarcity_scores_empty_input() {
    let input: Vec<(String, HashMap<String, usize>)> = vec![];
    let scores = calculate_hw_component_scarcity_scores(&input).await;
    assert!(scores.is_empty());
  }

  #[tokio::test]
  async fn scarcity_scores_scarce_component_gets_higher_score() {
    let input = vec![
      (
        "n1".to_string(),
        HashMap::from([
          ("common".to_string(), 10usize),
          ("rare".to_string(), 1usize),
        ]),
      ),
      (
        "n2".to_string(),
        HashMap::from([("common".to_string(), 10usize)]),
      ),
    ];
    let scores = calculate_hw_component_scarcity_scores(&input).await;
    assert!(scores["rare"] > scores["common"]);
  }

  // ---- get_best_candidate_in_target_and_parent_hsm ----

  #[test]
  fn target_and_parent_prefers_target() {
    let mut target_scores = vec![("t1".to_string(), 3.0f32)];
    let mut parent_scores = vec![("p1".to_string(), 5.0f32)];
    let mut target_hw = vec![(
      "t1".to_string(),
      HashMap::from([("a100".to_string(), 4usize)]),
    )];
    let parent_hw = vec![(
      "p1".to_string(),
      HashMap::from([("a100".to_string(), 2usize)]),
    )];
    let result = get_best_candidate_in_target_and_parent_hsm(
      &mut target_scores,
      &mut parent_scores,
      &mut target_hw,
      &parent_hw,
    );
    let (candidate, hw) = result.unwrap();
    assert_eq!(candidate.0, "t1");
    assert_eq!(hw.get("a100"), Some(&4));
  }

  #[test]
  fn target_and_parent_falls_back_to_parent_when_target_empty() {
    let mut target_scores: Vec<(String, f32)> = vec![];
    let mut parent_scores = vec![("p1".to_string(), 5.0f32)];
    let mut target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw = vec![(
      "p1".to_string(),
      HashMap::from([("a100".to_string(), 2usize)]),
    )];
    let result = get_best_candidate_in_target_and_parent_hsm(
      &mut target_scores,
      &mut parent_scores,
      &mut target_hw,
      &parent_hw,
    );
    let (candidate, _) = result.unwrap();
    assert_eq!(candidate.0, "p1");
  }

  #[test]
  fn target_and_parent_returns_none_when_both_empty() {
    let mut target_scores: Vec<(String, f32)> = vec![];
    let mut parent_scores: Vec<(String, f32)> = vec![];
    let mut target_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let parent_hw: Vec<(String, HashMap<String, usize>)> = vec![];
    let result = get_best_candidate_in_target_and_parent_hsm(
      &mut target_scores,
      &mut parent_scores,
      &mut target_hw,
      &parent_hw,
    );
    assert!(result.is_none());
  }
}
