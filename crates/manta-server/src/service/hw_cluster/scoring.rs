//! Hardware component scoring + inventory helpers.
//!
//! Pure-computation functions plus the parallel hw-inventory fetcher.
//! `resolve_hw_description_to_xnames` lives here as a higher-level
//! coordinator that picks between the pin/unpin algorithms; it calls
//! into `super::pin_unpin`.
//!
//! # Scoring rubric
//!
//! A node's score is the sum of its components' contributions, each
//! weighted by component scarcity (`total_components / supply_of_kind`,
//! so rarer components carry more weight).
//!
//! For each `(component, qty)` pair on the node:
//!
//! - If the user-requested pattern doesn't mention the component at
//!   all, contribute *negatively* — the node has hw the workload
//!   doesn't care about, so moving it is cheap.
//! - If the user wants *less* of this component than the current
//!   parent supply provides, contribute *positively* — keeping this
//!   node in the target satisfies that part of the pattern.
//! - Otherwise contribute *negatively* — the node has hw the workload
//!   doesn't need (yet), so it's a cheap candidate to release.
//!
//! Highest score wins. Pin mode breaks ties in favour of nodes already
//! in the target group so memberships are stable across reruns; Unpin
//! treats target and parent as a single pool. See
//! [`get_best_candidate_in_hsm`] and
//! [`get_best_candidate_in_target_and_parent_hsm`].
//!
//! # Concurrency
//!
//! [`get_group_node_hw_component_counter`] fans out per-node inventory
//! fetches under a `tokio::sync::Semaphore` bounded by
//! [`HW_COMPONENT_CONCURRENCY_LIMIT`]. Memory DIMM capacities are
//! normalised against `MEMORY_CAPACITY_LCM` (16 GiB) so the scorer
//! sees integer DIMM counts rather than raw MiB.

use std::{collections::HashMap, sync::Arc, time::Instant};

use comfy_table::Color;
use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::{group::GroupTrait, hardware_inventory::HardwareInventory},
};
use serde_json::Value;
use tokio::sync::Semaphore;

use super::{
  HW_COMPONENT_CONCURRENCY_LIMIT, HwClusterMode, NodeHwCountVec,
  hw_inventory_utils, pin_unpin,
};
use crate::dispatcher::StaticBackendDispatcher;
use crate::server::common::app_context::InfraContext;

/// Compute a scarcity score for each hardware component type across
/// all nodes: `total_components_in_pool / supply_of_this_kind`. Rarer
/// components score higher and dominate the per-node weights used by
/// [`calculate_group_node_scores_from_final_hsm`].
//
// The `usize -> f64` casts below would trigger `clippy::cast_precision_loss`
// because f64's 52-bit mantissa can't represent every 64-bit usize. In
// practice these values are hardware-component counts (low thousands at
// the largest realistic site); 2^52 ≈ 4.5e15 covers any plausible fleet
// size by many orders of magnitude.
#[allow(clippy::cast_precision_loss)]
pub fn calculate_hw_component_scarcity_scores(
  group_node_hw_component_count: &[(String, HashMap<String, usize>)],
) -> HashMap<String, f64> {
  let total_num_hw_components: usize = group_node_hw_component_count
    .iter()
    .flat_map(|(_, hw_component_qty_hashmap)| hw_component_qty_hashmap.values())
    .sum();

  let mut hw_component_vec: Vec<&String> = group_node_hw_component_count
    .iter()
    .flat_map(|(_, hw_component_counter_hashmap)| {
      hw_component_counter_hashmap.keys()
    })
    .collect();

  hw_component_vec.sort();
  hw_component_vec.dedup();

  let mut hw_component_scarcity_score_hashmap: HashMap<String, f64> =
    HashMap::new();
  for hw_component in hw_component_vec {
    let mut group_hw_component_count = 0;

    for (_, hw_component_counter_hashmap) in group_node_hw_component_count {
      if let Some(hw_component_qty) =
        hw_component_counter_hashmap.get(hw_component)
      {
        group_hw_component_count += hw_component_qty;
      }
    }

    hw_component_scarcity_score_hashmap.insert(
      hw_component.clone(),
      (total_num_hw_components as f64) / (group_hw_component_count as f64),
    );
  }

  tracing::info!(
    "Hw component scarcity scores: {:?}",
    hw_component_scarcity_score_hashmap
  );

  hw_component_scarcity_score_hashmap
}

/// Score each node in the input vector against the user-requested
/// pattern. Implements the rubric in this module's header: components
/// the user wants pull positively when the parent has surplus,
/// negatively otherwise; components the user doesn't ask for always
/// pull negatively. Weighted by the precomputed scarcity scores so
/// rare hw dominates.
//
// Same `cast_precision_loss` justification as above — qty is a per-node
// component count, never large enough to overflow f64's mantissa.
#[allow(clippy::cast_precision_loss)]
pub fn calculate_group_node_scores_from_final_hsm(
  parent_group_node_hw_component_count_vec: &[(
    String,
    HashMap<String, usize>,
  )],
  parent_group_hw_component_summary_hashmap: &HashMap<String, usize>,
  final_group_summary_hashmap: &HashMap<String, usize>,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f64>,
) -> Vec<(String, f64)> {
  let mut node_score_vec: Vec<(String, f64)> = Vec::new();

  for (xname, hw_component_count) in parent_group_node_hw_component_count_vec {
    let mut node_score: f64 = 0.0;
    for (hw_component, qty) in hw_component_count {
      let scarcity_score = hw_component_scarcity_scores_hashmap
        .get(hw_component)
        .copied()
        .unwrap_or(0.0);

      if final_group_summary_hashmap.get(hw_component).is_none() {
        node_score -= scarcity_score * *qty as f64;
      } else {
        let final_qty = final_group_summary_hashmap
          .get(hw_component)
          .copied()
          .unwrap_or(0);
        let parent_qty = parent_group_hw_component_summary_hashmap
          .get(hw_component)
          .copied()
          .unwrap_or(0);

        if final_qty < parent_qty {
          node_score += scarcity_score * *qty as f64;
        } else {
          node_score -= scarcity_score * *qty as f64;
        }
      }
    }
    node_score_vec.push((xname.clone(), node_score));
  }

  node_score_vec
}

/// Check whether further iteration is needed to satisfy the target hw
/// pattern. Returns `true` while any user-requested component still has
/// more supply in the current pool than the user asked for — i.e.
/// there's still slack to drain.
pub fn keep_iterating_final_hsm(
  group_final_hw_component_summary_hashmap: &HashMap<String, usize>,
  group_current_hw_component_summary_hashmap: &HashMap<String, usize>,
) -> bool {
  for (hw_component, final_qty) in group_final_hw_component_summary_hashmap {
    if group_current_hw_component_summary_hashmap
      .get(hw_component)
      .is_some_and(|current_qty| current_qty > final_qty)
    {
      return true;
    }
  }

  false
}

/// Aggregate per-node hardware counters into a single summary map.
pub fn calculate_group_hw_component_summary(
  target_group_node_hw_component_vec: &[(String, HashMap<String, usize>)],
) -> HashMap<String, usize> {
  let mut group_hw_component_count_hashmap = HashMap::new();

  for (_xname, node_hw_component_count_hashmap) in
    target_group_node_hw_component_vec
  {
    for (hw_component, &qty) in node_hw_component_count_hashmap {
      group_hw_component_count_hashmap
        .entry(hw_component.clone())
        .and_modify(|qty_aux| *qty_aux += qty)
        .or_insert(qty);
    }
  }

  group_hw_component_count_hashmap
}

/// Returns properties from a hardware inventory value matching the given pattern.
fn get_node_hw_properties_from_value(
  node_hw_inventory_value: &Value,
  hw_component_pattern_list: &[String],
) -> (Vec<String>, Vec<u64>) {
  let processor_vec =
    hw_inventory_utils::get_list_processor_model_from_hw_inventory_value(
      node_hw_inventory_value,
    )
    .unwrap_or_default();

  let accelerator_vec =
    hw_inventory_utils::get_list_accelerator_model_from_hw_inventory_value(
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
      node_hw_component_pattern_vec.push(hw_component_pattern.clone());
    } else {
      node_hw_component_pattern_vec.push(actual_hw_component_pattern);
    }
  }

  let memory_vec =
    hw_inventory_utils::get_list_memory_capacity_from_hw_inventory_value(
      node_hw_inventory_value,
    )
    .unwrap_or_default();

  (node_hw_component_pattern_vec, memory_vec)
}

/// Fetch hw inventory for a single node.
async fn get_node_hw_component_count(
  backend: StaticBackendDispatcher,
  shasta_token: String,
  group_member: &str,
  user_defined_hw_profile_vec: Vec<String>,
) -> (String, Vec<String>, Vec<u64>) {
  let hw_inventory_typed = match backend
    .get_inventory_hardware_query(
      &shasta_token,
      group_member,
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
      tracing::error!(
        "Failed to get hw inventory for '{}': {}",
        group_member,
        e
      );
      return (group_member.to_string(), Vec::new(), Vec::new());
    }
  };

  // `get_node_hw_properties_from_value` parses JSON paths out of a
  // Value; re-serialize the typed `HWInventory` here. Future work could
  // refactor that helper to take `&HWInventory` directly.
  let node_hw_inventory_value =
    serde_json::to_value(&hw_inventory_typed).unwrap_or_default();
  let node_hw_profile = get_node_hw_properties_from_value(
    &node_hw_inventory_value,
    &user_defined_hw_profile_vec,
  );

  (
    group_member.to_string(),
    node_hw_profile.0,
    node_hw_profile.1,
  )
}

/// Print a table of node hardware component scores with color-coded cells.
pub fn print_score_table(
  user_defined_hw_component_vec: &[String],
  group_hw_pattern_vec: &[(String, HashMap<String, usize>)],
  group_score_vec: &[(String, f64)],
) {
  let group_hw_component_vec: Vec<String> = group_hw_pattern_vec
    .iter()
    .flat_map(|(_xname, node_pattern_hashmap)| {
      node_pattern_hashmap.keys().cloned()
    })
    .collect();

  let mut all_hw_component_vec = [
    group_hw_component_vec,
    user_defined_hw_component_vec.to_vec(),
  ]
  .concat();

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

  for (xname, node_pattern_hashmap) in group_hw_pattern_vec {
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
          comfy_table::Cell::new(format!("\u{1F7E2} ({counter})"))
            .fg(Color::Green)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else if node_pattern_hashmap.contains_key(hw_component) {
        let counter =
          node_pattern_hashmap.get(hw_component).copied().unwrap_or(0);
        row.push(
          comfy_table::Cell::new(format!("\u{1F7E1} ({counter})"))
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

    let default_score = (xname.clone(), 0.0);
    let node_score = group_score_vec
      .iter()
      .find(|(node_name, _)| node_name.eq(xname))
      .unwrap_or(&default_score)
      .1;
    let node_score_table_cell = if node_score <= 0.0 {
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
pub async fn get_group_node_hw_component_counter(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  user_defined_hw_component_vec: &[String],
  group_member_vec: &[String],
  mem_lcm: u64,
) -> Vec<(String, HashMap<String, usize>)> {
  let start = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(HW_COMPONENT_CONCURRENCY_LIMIT));

  let mut target_group_node_hw_component_count_vec = Vec::new();

  for group_member in group_member_vec {
    let shasta_token_string = shasta_token.to_string();
    let user_defined_hw_component_vec =
      user_defined_hw_component_vec.to_owned();
    // Owned clone needed for `tokio::spawn` below — see
    // `InfraContext::backend_clone`'s docstring for the lifetime story.
    let backend_clone = infra.backend_clone();
    let group_member = group_member.clone();

    let permit = Arc::clone(&sem).acquire_owned().await;

    tasks.spawn(async move {
      let _permit = permit;

      get_node_hw_component_count(
        backend_clone,
        shasta_token_string,
        &group_member,
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

      target_group_node_hw_component_count_vec.push((
        node_hw_component_vec_tuple.0,
        node_hw_component_count_hashmap,
      ));
    } else {
      tracing::error!("Failed processing/fetching node hw information");
    }
  }

  let duration = start.elapsed();
  tracing::info!("Time elapsed to calculate hw components is: {:?}", duration);

  target_group_node_hw_component_count_vec
}

/// Select the best candidate node by highest score, breaking ties by
/// xname (lexicographic ascending) for determinism. Returns `None` if
/// either input is empty. Used directly by Unpin; Pin layers
/// [`get_best_candidate_in_target_and_parent_hsm`] on top.
pub fn get_best_candidate_in_hsm(
  group_score_vec: &mut [(String, f64)],
  group_hw_component_vec: &[(String, HashMap<String, usize>)],
) -> Option<((String, f64), HashMap<String, usize>)> {
  if group_score_vec.is_empty() || group_hw_component_vec.is_empty() {
    return None;
  }

  group_score_vec.sort_by(|a, b| {
    b.1
      .partial_cmp(&a.1)
      .unwrap_or(std::cmp::Ordering::Equal)
      .then(a.0.cmp(&b.0))
  });

  let best_candidate: (String, f64) = group_score_vec.first()?.clone();

  group_hw_component_vec
    .iter()
    .find(|(node, _)| node.eq(&best_candidate.0))
    .map(|best_candidate_hw| (best_candidate, best_candidate_hw.1.clone()))
}

/// For PIN mode: select the best candidate, preferring existing target
/// nodes whenever the target pool still has one to offer. Falls back to
/// the parent pool only when the target is exhausted. This is what
/// makes Pin "stable" — runs converge to the same target memberships
/// when the inputs don't change.
pub fn get_best_candidate_in_target_and_parent_hsm(
  target_group_node_score_tuple_vec: &mut [(String, f64)],
  parent_group_node_score_tuple_vec: &mut [(String, f64)],
  target_group_node_hw_component_count_vec: &mut [(
    String,
    HashMap<String, usize>,
  )],
  parent_group_node_hw_component_count_vec: &[(
    String,
    HashMap<String, usize>,
  )],
) -> Option<((String, f64), HashMap<String, usize>)> {
  let target_best_candidate_tuple = get_best_candidate_in_hsm(
    target_group_node_score_tuple_vec,
    target_group_node_hw_component_count_vec,
  );

  let parent_best_candidate_tuple = get_best_candidate_in_hsm(
    parent_group_node_score_tuple_vec,
    parent_group_node_hw_component_count_vec,
  );

  if target_best_candidate_tuple.is_some() {
    target_best_candidate_tuple
  } else if parent_best_candidate_tuple.is_some() {
    parent_best_candidate_tuple
  } else {
    None
  }
}

/// Resolve a hardware description pattern into concrete xnames by
/// running the Pin or Unpin selection algorithm against the supplied
/// inventories. Returns `(new_target, remaining_parent)` — the
/// post-move membership lists ready to feed to
/// `super::pin_unpin::apply_group_updates`.
///
/// # Errors
///
/// Propagates `InsufficientResources` from
/// [`super::pin_unpin::calculate_target_group_pin`] /
/// [`super::pin_unpin::calculate_target_group_unpin`] when no valid
/// selection plan exists.
//
// `type_complexity`: the tuple-of-vecs-of-tuples shape is exactly
// what this function negotiates and renaming the parts behind a
// `type` alias would just push the same shape one level down. Keeping
// the structural type at the signature is more honest about the data
// flow.
#[allow(clippy::type_complexity)]
pub fn resolve_hw_description_to_xnames(
  mode: HwClusterMode,
  mut target_group_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )>,
  mut parent_group_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )>,
  user_defined_target_group_hw_component_count_hashmap: &HashMap<String, usize>,
) -> Result<
  (
    Vec<(String, HashMap<String, usize>)>,
    Vec<(String, HashMap<String, usize>)>,
  ),
  Error,
> {
  let mut combined_target_parent_group_node_hw_component_count_vec =
    parent_group_node_hw_component_count_vec.clone();

  for elem in &target_group_node_hw_component_count_vec {
    if !parent_group_node_hw_component_count_vec
      .iter()
      .any(|(xname, _)| xname.eq(&elem.0))
    {
      combined_target_parent_group_node_hw_component_count_vec
        .push(elem.clone());
    }
  }

  let combined_target_parent_group_hw_component_summary_hashmap =
    calculate_group_hw_component_summary(
      &combined_target_parent_group_node_hw_component_count_vec,
    );

  let hw_component_scarcity_scores_hashmap: HashMap<String, f64> =
    calculate_hw_component_scarcity_scores(
      &combined_target_parent_group_node_hw_component_count_vec,
    );

  let mut final_combined_target_parent_group_hw_component_summary =
    user_defined_target_group_hw_component_count_hashmap.clone();

  for (hw_component, qty) in
    combined_target_parent_group_hw_component_summary_hashmap
  {
    final_combined_target_parent_group_hw_component_summary
      .entry(hw_component)
      .and_modify(|current_qty| *current_qty = qty - *current_qty);
  }

  let hw_component_counters_to_move_out_from_combined_hsm = match mode {
    HwClusterMode::Pin => pin_unpin::calculate_target_group_pin(
      &final_combined_target_parent_group_hw_component_summary,
      &final_combined_target_parent_group_hw_component_summary
        .keys()
        .cloned()
        .collect::<Vec<String>>(),
      &mut combined_target_parent_group_node_hw_component_count_vec,
      &mut target_group_node_hw_component_count_vec,
      &mut parent_group_node_hw_component_count_vec,
      &hw_component_scarcity_scores_hashmap,
    )?,
    HwClusterMode::Unpin => pin_unpin::calculate_target_group_unpin(
      &final_combined_target_parent_group_hw_component_summary,
      &final_combined_target_parent_group_hw_component_summary
        .keys()
        .cloned()
        .collect::<Vec<String>>(),
      &mut combined_target_parent_group_node_hw_component_count_vec,
      &hw_component_scarcity_scores_hashmap,
    )?,
  };

  let new_target_group_node_hw_component_count_vec =
    hw_component_counters_to_move_out_from_combined_hsm;

  Ok((
    new_target_group_node_hw_component_count_vec,
    combined_target_parent_group_node_hw_component_count_vec,
  ))
}

/// Parse a hardware pattern string like `"a100:4:epyc:10"` into
/// component names and a hashmap of `{component -> isize count}`.
/// Counts are signed (`isize`) so callers can express deltas, but in
/// practice they're non-negative; see `apply::compute_final_*_summary`
/// for the explicit overflow guards.
///
/// # Errors
///
/// Returns [`Error::InvalidPattern`] if the pattern has an odd number
/// of elements or a count fails to parse as `isize`.
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

/// Fetch HSM group members, compute per-node hw component counts, and
/// return the member list, per-node counts (sorted by xname), and
/// group-wide summary. Inventory fetches run concurrently — see
/// [`get_group_node_hw_component_counter`].
///
/// # Errors
///
/// Returns [`Error::NotFound`] when the group lookup fails — usually
/// because the group doesn't exist or the token lacks access.
pub async fn fetch_group_hw_inventory(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  hw_components: &[String],
  group_name: &str,
  mem_lcm: u64,
) -> Result<(Vec<String>, NodeHwCountVec, HashMap<String, usize>), Error> {
  let member_vec: Vec<String> = infra
    .backend
    .get_member_vec_from_group_name_vec(shasta_token, &[group_name.to_string()])
    .await
    .map_err(|e| {
      Error::NotFound(format!(
        "Failed to get members from HSM group '{group_name}': {e}"
      ))
    })?;

  let mut node_hw_count_vec = get_group_node_hw_component_counter(
    infra,
    shasta_token,
    hw_components,
    &member_vec,
    mem_lcm,
  )
  .await;

  node_hw_count_vec.sort_by(|a, b| a.0.cmp(&b.0));

  let summary = calculate_group_hw_component_summary(&node_hw_count_vec);

  Ok((member_vec, node_hw_count_vec, summary))
}
