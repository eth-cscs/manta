//! High-level coordinators: `apply_hw_configuration` (pin/unpin),
//! `add_hw_component`, `delete_hw_component`. These are the functions
//! the server handlers call directly.

use std::collections::HashMap;

use manta_backend_dispatcher::{error::Error, types::Group};

use super::{
  AddHwResult, ApplyHwResult, DeleteHwResult, HwClusterMode,
  MEMORY_CAPACITY_LCM, pin_unpin, scoring,
};
use crate::server::common::app_context::InfraContext;

/// Pin or unpin nodes between `parent_group_name` and
/// `target_group_name` so the target group satisfies `pattern`.
///
/// The flow is parse → ensure → score → resolve → apply: the
/// component pattern is parsed into a counts map, the target group is
/// created on demand (or refused when `create_target_group` is false
/// and the group is missing), parent and target hardware inventories
/// are fetched, the resource-sufficiency check rejects patterns that
/// ask for more of a component than exists in the pool, and `mode`
/// (`Pin` / `Unpin`) picks which selection algorithm runs. `dryrun`
/// shortcuts every backend mutation but still returns the would-be
/// final memberships so the operator sees the plan.
/// Parameters for [`apply_hw_configuration`].
pub struct ApplyHwConfigurationParams<'a> {
  /// `Pin` (capacity-aware selection) or `Unpin` (release all).
  pub mode: HwClusterMode,
  /// Destination HSM group that will receive nodes matching `pattern`.
  pub target_group_name: &'a str,
  /// Source HSM group nodes are drawn from when honouring `pattern`.
  pub parent_group_name: &'a str,
  /// Hardware-component request string, e.g. `"a100:8,milan:2"`.
  pub pattern: &'a str,
  /// When `true`, plan the moves but skip every backend mutation; the
  /// returned `ApplyHwResult` still reflects the would-be membership.
  pub dryrun: bool,
  /// Create `target_group_name` if it doesn't already exist.
  pub create_target_group: bool,
  /// Delete the parent group when the move leaves it with no members.
  pub delete_empty_parent_group: bool,
}

/// Service entry point for `POST /hardware-clusters/{target}/configuration`.
pub async fn apply_hw_configuration(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  p: ApplyHwConfigurationParams<'_>,
) -> Result<ApplyHwResult, Error> {
  let ApplyHwConfigurationParams {
    mode,
    target_group_name,
    parent_group_name,
    pattern,
    dryrun,
    create_target_group,
    delete_empty_parent_group,
  } = p;
  let (user_defined_hw_component_vec, user_defined_hw_component_count_hashmap) =
    pin_unpin::parse_hw_pattern_usize(target_group_name, pattern)?;

  pin_unpin::ensure_target_group_exists(
    infra,
    shasta_token,
    target_group_name,
    dryrun,
    create_target_group,
  )
  .await?;

  let (
    target_hsm_group_member_vec,
    target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = scoring::fetch_group_hw_inventory(
    infra,
    shasta_token,
    &user_defined_hw_component_vec,
    target_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  tracing::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_group_name,
    target_hsm_hw_component_summary
  );

  let (
    parent_hsm_group_member_vec,
    parent_hsm_node_hw_component_count_vec,
    _parent_summary,
  ) = scoring::fetch_group_hw_inventory(
    infra,
    shasta_token,
    &user_defined_hw_component_vec,
    parent_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  pin_unpin::validate_resource_sufficiency(
    &target_hsm_node_hw_component_count_vec,
    &parent_hsm_node_hw_component_count_vec,
    &user_defined_hw_component_count_hashmap,
  )?;

  let (
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
  ) = scoring::resolve_hw_description_to_xnames(
    mode,
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
    &user_defined_hw_component_count_hashmap,
  )?;

  let target_hsm_node_vec: Vec<String> = target_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect();

  let parent_hsm_node_vec: Vec<String> = parent_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect();

  pin_unpin::apply_group_updates(
    infra,
    shasta_token,
    pin_unpin::GroupUpdate {
      target_group: target_group_name,
      parent_group: parent_group_name,
      old_target_members: &target_hsm_group_member_vec,
      old_parent_members: &parent_hsm_group_member_vec,
      new_target_members: &target_hsm_node_vec,
      new_parent_members: &parent_hsm_node_vec,
      dryrun,
      delete_empty_parent: delete_empty_parent_group,
    },
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
  infra: &InfraContext<'_>,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  create_hsm_group: bool,
) -> Result<(), Error> {
  if infra
    .get_group(shasta_token, target_hsm_group_name)
    .await
    .is_ok()
  {
    tracing::debug!("The group '{}' exists, good.", target_hsm_group_name);
    return Ok(());
  }
  if !create_hsm_group {
    return Err(Error::NotFound(format!(
      "Group '{target_hsm_group_name}' does not exist, but the \
       option to create the group was NOT \
       specified, cannot continue."
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
  infra.add_group(shasta_token, group).await?;
  Ok(())
}

/// Compute the final parent HSM hw component summary after subtracting user-requested deltas.
//
// `deltas` carries signed counters (`isize`) because callers compute
// the difference between current and target counts; in practice the
// values are non-negative HW component subtractions. The
// `*counter as usize` cast is guarded by the explicit
// `if *counter > current as isize` overflow check above each call site.
#[allow(clippy::cast_sign_loss)]
fn compute_final_parent_summary(
  current_summary: &HashMap<String, usize>,
  deltas: &HashMap<String, isize>,
  parent_group_name: &str,
) -> Result<HashMap<String, usize>, Error> {
  let mut final_summary: HashMap<String, usize> = HashMap::new();

  for (hw_component, counter) in deltas {
    let current = *current_summary.get(hw_component).unwrap_or(&0);
    if *counter > current.cast_signed() {
      return Err(Error::InsufficientResources(format!(
        "Cannot remove more hw component '{}' \
         ({}) than available in parent group \
         '{}' ({})",
        hw_component, *counter, parent_group_name, current
      )));
    }
    let new_counter = current - *counter as usize;
    final_summary.insert(hw_component.clone(), new_counter);
  }

  Ok(final_summary)
}

/// Move enough nodes out of `parent_group_name` into
/// `target_group_name` to add the components described by
/// `pattern` (`<component>:<delta>` pairs) to the target.
///
/// The target group is created on demand when `create_group` is
/// set; missing it otherwise yields `NotFound`. The parent group's
/// post-move hw component summary is computed up front so the
/// algorithm can reject patterns that would over-draw the parent
/// (`InsufficientResources`). Selection uses scarcity-weighted scores
/// so common components get pulled first and rare ones are preserved.
/// In `dryrun` mode the planned move is returned without any backend
/// mutation.
pub async fn add_hw_component(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  target_group_name: &str,
  parent_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_group: bool,
) -> Result<AddHwResult, Error> {
  ensure_add_target_group_exists(
    infra,
    shasta_token,
    target_group_name,
    dryrun,
    create_group,
  )
  .await?;

  let pattern_str = format!("{target_group_name}:{pattern}");
  let pattern_lowercase = pattern_str.to_lowercase();
  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();
  let target_name = pattern_element_vec.remove(0);

  let (
    user_defined_delta_hw_component_vec,
    user_defined_delta_hw_component_count_hashmap,
  ) = scoring::parse_hw_pattern(&pattern_element_vec)?;

  let (
    _parent_member_vec,
    mut parent_hsm_node_hw_component_count_vec,
    parent_hsm_hw_component_summary,
  ) = scoring::fetch_group_hw_inventory(
    infra,
    shasta_token,
    &user_defined_delta_hw_component_vec,
    parent_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  let final_parent_hsm_hw_component_summary = compute_final_parent_summary(
    &parent_hsm_hw_component_summary,
    &user_defined_delta_hw_component_count_hashmap,
    parent_group_name,
  )?;

  let scarcity_scores = scoring::calculate_hw_component_scarcity_scores(
    &parent_hsm_node_hw_component_count_vec,
  );

  let hw_counters_to_move = pin_unpin::calculate_target_group_unpin(
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

  let mut target_hsm_node_vec: Vec<String> = infra
    .get_member_vec_from_group_name_vec(
      shasta_token,
      &[target_name.to_string()],
    )
    .await?;

  target_hsm_node_vec.extend(nodes_to_move.clone());
  target_hsm_node_vec.sort();

  if !dryrun {
    for xname in &nodes_to_move {
      infra
        .delete_member_from_group(shasta_token, parent_group_name, xname)
        .await?;

      infra
        .add_members_to_group(shasta_token, target_name, &[xname.as_str()])
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
  infra: &InfraContext<'_>,
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
  match infra
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
  }
  Ok(())
}

/// Compute the final target HSM hw component summary after subtracting deltas.
//
// Same `isize → usize` cast rationale as `compute_final_parent_summary`:
// callers compute non-negative HW deltas; the cast preserves intent.
#[allow(clippy::cast_sign_loss)]
fn compute_delete_final_summary(
  current_summary: &HashMap<String, usize>,
  deltas: &HashMap<String, isize>,
) -> Result<HashMap<String, usize>, Error> {
  let mut final_summary: HashMap<String, usize> = HashMap::new();

  for (hw_component, counter) in deltas {
    let current = *current_summary.get(hw_component).ok_or_else(|| {
      Error::NotFound(format!(
        "hw component '{hw_component}' not found in target HSM \
           hw component summary"
      ))
    })?;

    final_summary.insert(hw_component.clone(), current - *counter as usize);
  }

  Ok(final_summary)
}

/// Move nodes between HSM groups: delete from target, add to parent.
async fn apply_node_moves(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  target_group: &str,
  parent_group: &str,
  nodes: &[String],
  target_will_be_empty: bool,
  delete_hsm_group: bool,
) -> Result<(), Error> {
  for xname in nodes {
    infra
      .delete_member_from_group(shasta_token, target_group, xname.as_str())
      .await?;

    infra
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
      match infra.delete_group(shasta_token, target_group).await {
        Ok(_) => tracing::info!("HSM group removed successfully."),
        Err(e) => tracing::debug!(
          "Error removing the HSM group. This always \
           fails, ignore please. Reported: {}",
          e
        ),
      }
    } else {
      tracing::debug!(
        "HSM group {} is now empty and the option to \
         delete empty groups has NOT been selected, \
         will not remove it.",
        target_group
      );
    }
  }

  Ok(())
}

/// Move enough nodes out of `target_group_name` back into
/// `parent_group_name` to remove the components described by
/// `pattern` from the target.
///
/// The target group must already exist (returns `NotFound`
/// otherwise). When the target is already empty the routine
/// short-circuits, optionally deleting the empty group if
/// `delete_group` is set. Selection scores combine both groups'
/// scarcity, so the move keeps the most scarce hardware in the target
/// group whenever possible. After moving, the function deletes the
/// target group if it ended up empty and `delete_group` is true.
/// `dryrun` returns the planned move without touching the backend.
pub async fn delete_hw_component(
  infra: &InfraContext<'_>,
  token: &str,
  target_group_name: &str,
  parent_group_name: &str,
  pattern: &str,
  dryrun: bool,
  delete_group: bool,
) -> Result<DeleteHwResult, Error> {
  match infra.get_group(token, target_group_name).await {
    Ok(_) => {}
    Err(_) => {
      return Err(Error::NotFound(format!(
        "HSM group {target_group_name} does not exist, cannot remove hw from it."
      )));
    }
  }

  let pattern_str = format!("{target_group_name}:{pattern}");
  let pattern_lowercase = pattern_str.to_lowercase();
  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();
  let target_name = pattern_element_vec.remove(0);

  let (
    user_defined_delta_hw_component_vec,
    user_defined_delta_hw_component_count_hashmap,
  ) = scoring::parse_hw_pattern(&pattern_element_vec)?;

  let (
    target_hsm_group_member_vec,
    mut target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = scoring::fetch_group_hw_inventory(
    infra,
    token,
    &user_defined_delta_hw_component_vec,
    target_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  if target_hsm_node_hw_component_count_vec.is_empty() {
    handle_empty_target(infra, token, target_name, dryrun, delete_group)
      .await?;
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
  ) = scoring::fetch_group_hw_inventory(
    infra,
    token,
    &user_defined_delta_hw_component_vec,
    parent_group_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  let combined = [
    target_hsm_node_hw_component_count_vec.clone(),
    parent_hsm_node_hw_component_count_vec.clone(),
  ]
  .concat();
  let scarcity_scores =
    scoring::calculate_hw_component_scarcity_scores(&combined);

  let final_target_summary = compute_delete_final_summary(
    &target_hsm_hw_component_summary,
    &user_defined_delta_hw_component_count_hashmap,
  )?;

  let hw_counters_to_move = pin_unpin::calculate_target_group_unpin(
    &final_target_summary,
    &final_target_summary
      .keys()
      .cloned()
      .collect::<Vec<String>>(),
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
      infra,
      token,
      target_name,
      parent_group_name,
      &nodes_to_move,
      target_hsm_group_member_vec.len() == nodes_to_move.len(),
      delete_group,
    )
    .await?;
  }

  Ok(DeleteHwResult {
    nodes_moved: nodes_to_move,
    target_nodes,
    parent_nodes,
  })
}
