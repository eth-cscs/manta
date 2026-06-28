//! Pin / Unpin node-selection algorithms and their shared infrastructure
//! (pattern parsing, target-group existence check, resource-sufficiency
//! validation, and group-membership update orchestration).
//!
//! # Algorithm shape
//!
//! Both [`calculate_target_group_pin`] and
//! [`calculate_target_group_unpin`] are greedy iterators over candidate
//! nodes. Each iteration:
//!
//! 1. Recomputes per-node scores from the current combined inventory
//!    (see `scoring::calculate_group_node_scores_from_final_hsm`).
//! 2. Picks the highest-scoring candidate — Pin prefers existing
//!    target-group members so memberships are kept stable; Unpin
//!    treats target and parent as a single pool.
//! 3. Records the move and removes the node from the working sets.
//! 4. Repeats until `keep_iterating_final_hsm` reports the combined
//!    summary has converged on the user-requested counts.
//!
//! # Rollback semantics
//!
//! Selection itself is in-memory and side-effect-free: the working
//! `NodeHwCountVec`s are mutated, but no backend call is issued until
//! [`apply_group_updates`] runs. If selection fails mid-flight (e.g.
//! `InsufficientResources` because no candidate scores positively for
//! the remaining pattern) no group membership is touched — the caller
//! gets the error and the cluster is untouched.
//!
//! Backend mutation in [`apply_group_updates`] is *not* transactional:
//! it issues the target-group update first, then the parent. A failure
//! on the parent leaves the target update in place. The pattern is
//! tolerated because target updates are idempotent and operators
//! retry — there is no shared-state corruption.
//!
//! # Dry-run
//!
//! `dryrun` short-circuits every backend mutation in
//! [`apply_group_updates`] but otherwise walks the full plan, so the
//! returned `ApplyHwResult` still reflects the would-be membership.

use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait, types::Group,
};

use super::{NodeHwCountVec, scoring};
use crate::server::common::app_context::InfraContext;

// ── Pin algorithm ────────────────────────────────────────────────────────────

/// Node selection algorithm for PIN mode — keeps as many existing
/// target nodes as possible, pulling from parent only when needed.
///
/// Greedy: each iteration picks the node with the highest current
/// score, prefers existing target members on ties (see
/// `scoring::get_best_candidate_in_target_and_parent_hsm`), records the
/// move, and recomputes scores against the smaller combined pool.
/// Terminates when the running combined summary matches
/// `user_defined_hsm_hw_components_count_hashmap`.
///
/// # Errors
///
/// Returns [`Error::InsufficientResources`] if at any iteration no
/// candidate can be selected (e.g. all remaining nodes have already
/// been moved or the working sets are empty before the pattern is
/// satisfied).
//
// Scores are HW-component scarcity ratios — always non-negative, always
// well within `usize` range. The `f64 as usize` casts used as hashmap
// keys for bucketing nodes are intentional truncation; Rust's saturating
// `as` semantics handle any non-finite edge case.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn calculate_target_group_pin(
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
  > = scoring::calculate_group_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );
  let target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    scoring::calculate_group_hw_component_summary(
      target_hsm_node_hw_component_count_vec,
    );
  let parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    scoring::calculate_group_hw_component_summary(
      parent_hsm_node_hw_component_count_vec,
    );

  let mut target_hsm_node_score_tuple_vec: Vec<(String, f64)> =
    scoring::calculate_group_node_scores_from_final_hsm(
      target_hsm_node_hw_component_count_vec,
      &target_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

  let mut parent_hsm_node_score_tuple_vec: Vec<(String, f64)> =
    scoring::calculate_group_node_scores_from_final_hsm(
      parent_hsm_node_hw_component_count_vec,
      &parent_hsm_hw_component_summary_hashmap,
      user_defined_hsm_hw_components_count_hashmap,
      hw_component_scarcity_scores_hashmap,
    );

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
      scoring::calculate_group_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    target_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    let mut target_hsm_node_score_tuple_vec: Vec<(String, f64)> =
      scoring::calculate_group_node_scores_from_final_hsm(
        target_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

    let mut parent_hsm_node_score_tuple_vec: Vec<(String, f64)> =
      scoring::calculate_group_node_scores_from_final_hsm(
        parent_hsm_node_hw_component_count_vec,
        &combination_target_parent_hsm_hw_component_summary_hashmap,
        user_defined_hsm_hw_components_count_hashmap,
        hw_component_scarcity_scores_hashmap,
      );

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

/// Node selection algorithm for UNPIN mode — treats target and
/// parent as a single combined pool and picks nodes to move back to
/// parent until the user-requested counts are met.
///
/// Unlike pin, there's no preference for keeping target members in
/// place: a single `get_best_candidate_in_hsm` call picks the best
/// candidate from the merged pool each iteration.
///
/// # Errors
///
/// Returns [`Error::InsufficientResources`] if at any iteration no
/// candidate can be selected before the pattern is satisfied.
pub fn calculate_target_group_unpin(
  user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>,
  user_defined_hw_component_vec: &[String],
  combination_target_parent_hsm_node_hw_component_count_vec: &mut NodeHwCountVec,
  hw_component_scarcity_scores_hashmap: &HashMap<String, f64>,
) -> Result<NodeHwCountVec, Error> {
  let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<
    String,
    usize,
  > = scoring::calculate_group_hw_component_summary(
    combination_target_parent_hsm_node_hw_component_count_vec,
  );

  let mut combination_target_parent_hsm_node_score_tuple_vec: Vec<(
    String,
    f64,
  )> = scoring::calculate_group_node_scores_from_final_hsm(
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
      scoring::calculate_group_hw_component_summary(
        combination_target_parent_hsm_node_hw_component_count_vec,
      );

    combination_target_parent_hsm_node_score_tuple_vec
      .retain(|(node, _)| !node.eq(&best_candidate.0));

    let mut target_hsm_node_score_tuple_vec: Vec<(String, f64)> =
      scoring::calculate_group_node_scores_from_final_hsm(
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

/// Parse user pattern `"a100:4:epyc:10"` into hw component names and a
/// hashmap of `{component -> usize count}`. The target group name is
/// prepended internally before splitting, so the input pattern itself
/// must *not* include a leading group prefix.
///
/// # Errors
///
/// Returns [`Error::InvalidPattern`] if the pattern lacks a colon
/// separator, has an odd number of component:count pairs, or a count
/// element fails to parse as `usize`.
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

/// Ensure the target HSM group exists, creating it if
/// `create_target_hsm_group` is set. The created group has no members
/// and `exclusive_group = false`; population happens later in
/// [`apply_group_updates`].
///
/// # Errors
///
/// - [`Error::NotFound`] if the group is missing and
///   `create_target_hsm_group` is `false`.
/// - [`Error::BadRequest`] if `dryrun` is `true` and the group needs
///   to be created (creation isn't simulated).
/// - [`Error::BadRequest`] if the backend `add_group` call fails.
pub async fn ensure_target_group_exists(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
) -> Result<(), Error> {
  if infra
    .backend
    .get_group(shasta_token, target_hsm_group_name)
    .await
    .is_ok()
  {
    tracing::debug!(
      "Target HSM group '{}' exists, good.",
      target_hsm_group_name
    );
    return Ok(());
  }
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
  infra
    .backend
    .add_group(shasta_token, group)
    .await
    .map_err(|e| {
      Error::BadRequest(format!("Unable to create new target HSM group: {e}"))
    })?;
  Ok(())
}

/// Validate that combined target+parent resources can fulfil the user
/// request. Run before the selection algorithm to fail fast with a
/// caller-facing error rather than mid-way through scoring.
///
/// # Errors
///
/// Returns [`Error::InsufficientResources`] if any requested component
/// count exceeds the union of target and parent supply.
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

  let combined_summary =
    scoring::calculate_group_hw_component_summary(&combined);

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

/// Inputs to [`apply_group_updates`] bundled to avoid a ten-arg
/// positional call. Pairs the old and new membership lists per
/// group; the function uses the pair to decide whether the parent
/// will be left empty by the move.
pub struct GroupUpdate<'a> {
  /// Destination group label.
  pub target_group: &'a str,
  /// Source group label.
  pub parent_group: &'a str,
  /// Membership of the target group before the update.
  pub old_target_members: &'a [String],
  /// Membership of the parent group before the update.
  pub old_parent_members: &'a [String],
  /// Membership of the target group after the update.
  pub new_target_members: &'a [String],
  /// Membership of the parent group after the update.
  pub new_parent_members: &'a [String],
  /// When `true`, skip every backend mutation but still walk the plan.
  pub dryrun: bool,
  /// When `true` and the parent group has no remaining members after
  /// the update, delete the parent group too.
  pub delete_empty_parent: bool,
}

/// Apply group membership updates to both target and parent HSM
/// groups: target first, then parent. Optionally deletes the parent if
/// it ends up empty and `delete_empty_parent` is set. Each call is a
/// remove-then-add against the backend's `update_group_members`
/// endpoint.
///
/// Not transactional: a failure on the parent update leaves the target
/// update in place. See the module docs for the rollback contract.
///
/// # Errors
///
/// - [`Error::BadRequest`] if either `update_group_members` call
///   fails on the backend.
/// - The empty-parent `delete_group` call is best-effort; failures are
///   only logged (this matches CSM's quirky delete-group behaviour;
///   see the inline comment).
pub async fn apply_group_updates(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  u: GroupUpdate<'_>,
) -> Result<(), Error> {
  tracing::info!("Updating target HSM group '{}' members", u.target_group);
  if u.dryrun {
    tracing::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    let target_remove_ref: Vec<&str> =
      u.old_target_members.iter().map(String::as_str).collect();
    let target_add_ref: Vec<&str> =
      u.new_target_members.iter().map(String::as_str).collect();
    infra
      .backend
      .update_group_members(
        shasta_token,
        u.target_group,
        &target_remove_ref,
        &target_add_ref,
      )
      .await
      .map_err(|e| {
        Error::BadRequest(format!(
          "Failed to update target HSM group members: {e}"
        ))
      })?;
  }

  tracing::info!("Updating parent HSM group '{}' members", u.parent_group);
  if u.dryrun {
    tracing::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    let parent_will_be_empty =
      u.old_target_members.len() == u.old_parent_members.len();
    let parent_remove_ref: Vec<&str> =
      u.old_parent_members.iter().map(String::as_str).collect();
    let parent_add_ref: Vec<&str> =
      u.new_parent_members.iter().map(String::as_str).collect();
    infra
      .backend
      .update_group_members(
        shasta_token,
        u.parent_group,
        &parent_remove_ref,
        &parent_add_ref,
      )
      .await
      .map_err(|e| {
        Error::BadRequest(format!(
          "Failed to update parent HSM group members: {e}"
        ))
      })?;

    if parent_will_be_empty && u.delete_empty_parent {
      tracing::info!(
        "Parent HSM group '{}' is now empty and \
         the option to delete empty groups has \
         been selected, removing it.",
        u.parent_group
      );
      match infra
        .backend
        .delete_group(shasta_token, u.parent_group)
        .await
      {
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
        u.parent_group
      );
    }
  }

  Ok(())
}
