//! High-level coordinators: `apply_hw_configuration` (pin/unpin),
//! `add_hw_component`, `delete_hw_component`. These are the functions
//! the server handlers call directly.

use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait, types::Group,
};

use super::{
  AddHwResult, ApplyHwResult, DeleteHwResult, HwClusterMode,
  MEMORY_CAPACITY_LCM, pin_unpin, scoring,
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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
    pin_unpin::parse_hw_pattern_usize(target_hsm_group_name, pattern)?;

  pin_unpin::ensure_target_group_exists(
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
  ) = scoring::fetch_hsm_hw_inventory(
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
  ) = scoring::fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_hw_component_vec,
    parent_hsm_group_name,
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

  pin_unpin::apply_group_updates(
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
  ) = scoring::parse_hw_pattern(&pattern_element_vec)?;

  let (
    _parent_member_vec,
    mut parent_hsm_node_hw_component_count_vec,
    parent_hsm_hw_component_summary,
  ) = scoring::fetch_hsm_hw_inventory(
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

  let scarcity_scores = scoring::calculate_hw_component_scarcity_scores(
    &parent_hsm_node_hw_component_count_vec,
  )
  .await;

  let hw_counters_to_move = pin_unpin::calculate_target_hsm_unpin(
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
  ) = scoring::parse_hw_pattern(&pattern_element_vec)?;

  let (
    target_hsm_group_member_vec,
    mut target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = scoring::fetch_hsm_hw_inventory(
    backend,
    token,
    &user_defined_delta_hw_component_vec,
    target_name,
    MEMORY_CAPACITY_LCM,
  )
  .await?;

  if target_hsm_node_hw_component_count_vec.is_empty() {
    handle_empty_target(backend, token, target_name, dryrun, delete_hsm_group)
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
  ) = scoring::fetch_hsm_hw_inventory(
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
  let scarcity_scores =
    scoring::calculate_hw_component_scarcity_scores(&combined).await;

  let final_target_summary = compute_delete_final_summary(
    &target_hsm_hw_component_summary,
    &user_defined_delta_hw_component_count_hashmap,
  )?;

  let hw_counters_to_move = pin_unpin::calculate_target_hsm_unpin(
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

  Ok(DeleteHwResult {
    nodes_moved: nodes_to_move,
    target_nodes,
    parent_nodes,
  })
}
