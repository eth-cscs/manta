//! Pin / Unpin node-selection algorithms and their shared infrastructure
//! (pattern parsing, target-group existence check, resource-sufficiency
//! validation, and group-membership update orchestration).

use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait, types::Group,
};

use super::{NodeHwCountVec, scoring};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

// ── Pin algorithm ────────────────────────────────────────────────────────────

/// Node selection algorithm for PIN mode — keeps as many existing target nodes
/// as possible, pulling from parent only when needed.
pub fn calculate_target_hsm_pin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>,
  user_defined_hw_component_vec: &[String],
  combination_target_parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  target_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f64>,
) -> Result<NodeHwCountVec, Error> {
  let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<
    String,
    usize,
  > = scoring::calculate_hsm_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );
  let target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    scoring::calculate_hsm_hw_component_summary(
      target_hsm_node_hw_component_count_vec,
    );
  let parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    scoring::calculate_hsm_hw_component_summary(
      parent_hsm_node_hw_component_count_vec,
    );

  let mut target_hsm_node_score_tuple_vec: Vec<(String, f64)> =
    scoring::calculate_hsm_node_scores_from_final_hsm(
      target_hsm_node_hw_component_count_vec,
      &target_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

  let mut parent_hsm_node_score_tuple_vec: Vec<(String, f64)> =
    scoring::calculate_hsm_node_scores_from_final_hsm(
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
    scoring::get_best_candidate_in_target_and_parent_hsm(
      &mut target_hsm_node_score_tuple_vec,
      &mut parent_hsm_node_score_tuple_vec,
      target_hsm_node_hw_component_count_vec,
      parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| {
      Error::InsufficientResources("No best candidate found".to_string())
    })?;

  let mut work_to_do = scoring::keep_iterating_final_hsm(
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

    scoring::print_score_table(
      user_defined_hw_component_vec,
      target_hsm_node_hw_component_count_vec,
      &target_hsm_node_score_tuple_vec,
    );

    scoring::print_score_table(
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
      scoring::calculate_hsm_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    target_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    let mut target_hsm_node_score_tuple_vec: Vec<(String, f64)> =
      scoring::calculate_hsm_node_scores_from_final_hsm(
        target_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    let mut parent_hsm_node_score_tuple_vec: Vec<(String, f64)> =
      scoring::calculate_hsm_node_scores_from_final_hsm(
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
      scoring::get_best_candidate_in_target_and_parent_hsm(
        &mut target_hsm_node_score_tuple_vec,
        &mut parent_hsm_node_score_tuple_vec,
        target_hsm_node_hw_component_count_vec,
        parent_hsm_node_hw_component_count_vec,
      )
      .ok_or_else(|| {
        Error::InsufficientResources("No best candidate found".to_string())
      })?;

    work_to_do = scoring::keep_iterating_final_hsm(
      user_defined_hsm_hw_components_count_hashmap,
      &combination_target_parent_hsm_hw_component_summary_hashmap,
    );

    iter += 1;
  }

  tracing::info!("----- FINAL RESULT -----");
  tracing::info!("No candidates found");

  scoring::print_score_table(
    user_defined_hw_component_vec,
    target_hsm_node_hw_component_count_vec,
    &target_hsm_node_score_tuple_vec,
  );

  scoring::print_score_table(
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
  hw_component_scarcity_scores_hashmap: &HashMap<String, f64>,
) -> Result<NodeHwCountVec, Error> {
  let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<
    String,
    usize,
  > = scoring::calculate_hsm_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );

  let mut combination_target_parent_hsm_node_score_tuple_vec: Vec<(
    String,
    f64,
  )> = scoring::calculate_hsm_node_scores_from_final_hsm(
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
    scoring::get_best_candidate_in_hsm(
      &mut combination_target_parent_hsm_node_score_tuple_vec,
      combination_target_parent_hsm_node_hw_component_count_vec,
    )
    .ok_or_else(|| {
      Error::InsufficientResources("No best candidate found".to_string())
    })?;

  let mut work_to_do = scoring::keep_iterating_final_hsm(
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
        .map_or(0.0, |(_, score)| *score),
      best_candidate_counters
    );

    scoring::print_score_table(
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
      scoring::calculate_hsm_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    combination_target_parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    let mut target_hsm_node_score_tuple_vec: Vec<(String, f64)> =
      scoring::calculate_hsm_node_scores_from_final_hsm(
        combination_target_parent_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    (best_candidate, best_candidate_counters) =
      scoring::get_best_candidate_in_hsm(
        &mut target_hsm_node_score_tuple_vec,
        combination_target_parent_hsm_node_hw_component_count_vec,
      )
      .ok_or_else(|| {
        Error::InsufficientResources("No best candidate found".to_string())
      })?;

    work_to_do = scoring::keep_iterating_final_hsm(
      user_defined_hsm_hw_components_count_hashmap,
      &combination_target_parent_hsm_hw_component_summary_hashmap,
    );

    iter += 1;
  }

  tracing::info!("----- FINAL RESULT -----");
  tracing::info!("No candidates found");

  scoring::print_score_table(
    user_defined_hw_component_vec,
    combination_target_parent_hsm_node_hw_component_count_vec,
    &combination_target_parent_hsm_node_score_tuple_vec,
  );

  Ok(nodes_migrated_from_combination_target_parent_hsm)
}

// ── apply_hw_configuration support ───────────────────────────────────────────

/// Parse user pattern `"a100:4:epyc:10"` into hw component names and a hashmap
/// of `{component -> usize count}`.
pub fn parse_hw_pattern_usize(
  target_hsm_group_name: &str,
  pattern: &str,
) -> Result<(Vec<String>, HashMap<String, usize>), Error> {
  let pattern = format!("{target_hsm_group_name}:{pattern}");
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
pub async fn ensure_target_group_exists(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
) -> Result<(), Error> {
  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      tracing::debug!(
        "Target HSM group '{}' exists, good.",
        target_hsm_group_name
      );
      Ok(())
    }
    Err(_) => {
      if !create_target_hsm_group {
        return Err(Error::NotFound(format!(
          "Target HSM group '{target_hsm_group_name}' does not exist, \
           but the option to create the group was \
           NOT specified, cannot continue.",
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
      let _ = backend.add_group(shasta_token, group).await.map_err(|e| {
        Error::BadRequest(format!("Unable to create new target HSM group: {e}"))
      })?;
      Ok(())
    }
  }
}

/// Validate that combined target+parent resources can fulfil the user request.
pub fn validate_resource_sufficiency(
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

  let combined_summary = scoring::calculate_hsm_hw_component_summary(&combined);

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
pub async fn apply_group_updates(
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
      }
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
