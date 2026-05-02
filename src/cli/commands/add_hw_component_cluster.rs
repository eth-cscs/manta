use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::group::GroupTrait,
  types::Group,
};

use crate::{
  cli::commands::hw_cluster_common::{
    MEMORY_CAPACITY_LCM,
    utils::{
      calculate_hsm_hw_component_summary,
      calculate_hw_component_scarcity_scores, fetch_hsm_hw_inventory,
      get_hsm_node_hw_component_counter, parse_hw_pattern,
      print_hsm_group_json, show_solution_and_confirm,
    },
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Result of an `add hw-component` operation.
pub struct AddHwResult {
  pub nodes_moved: Vec<String>,
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

/// Core logic for adding hardware components to a cluster group.
/// No terminal interaction — suitable for both CLI and HTTP callers.
pub async fn run(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_hsm_group: bool,
) -> Result<AddHwResult, Error> {
  ensure_target_group_exists(
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

  let mem_lcm = MEMORY_CAPACITY_LCM;
  let (
    _parent_member_vec,
    mut parent_hsm_node_hw_component_count_vec,
    parent_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_delta_hw_component_vec,
    parent_hsm_group_name,
    mem_lcm,
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
    crate::cli::commands::apply_hw_cluster_unpin::utils::calculate_target_hsm_unpin(
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
    .await
    .map_err(|e| {
      Error::Message(format!(
        "Failed to get member vec from target HSM group: {e}"
      ))
    })?;

  target_hsm_node_vec.extend(nodes_to_move.clone());
  target_hsm_node_vec.sort();

  if !dryrun {
    for xname in &nodes_to_move {
      backend
        .delete_member_from_group(shasta_token, parent_hsm_group_name, xname)
        .await
        .map_err(|e| {
          Error::Message(format!(
            "Failed to delete member from parent group: {e}"
          ))
        })?;

      let _ = backend
        .add_members_to_group(
          shasta_token,
          target_name,
          &[xname.as_str()],
        )
        .await
        .map_err(|e| {
          Error::Message(format!("Failed to add member to target group: {e}"))
        })?;
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

/// Add hardware components to a cluster group (CLI entry point).
pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_hsm_group: bool,
) -> Result<(), Error> {
  ensure_target_group_exists(
    backend,
    shasta_token,
    target_hsm_group_name,
    dryrun,
    create_hsm_group,
  )
  .await?;

  let pattern = format!("{}:{}", target_hsm_group_name, pattern);
  let pattern_lowercase = pattern.to_lowercase();
  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();
  let target_hsm_group_name = pattern_element_vec.remove(0);

  let (
    user_defined_delta_hw_component_vec,
    user_defined_delta_hw_component_count_hashmap,
  ) = parse_hw_pattern(&pattern_element_vec)?;

  tracing::info!(
    "User defined hw components with counters: {:?}",
    user_defined_delta_hw_component_count_hashmap
  );

  let mem_lcm = MEMORY_CAPACITY_LCM;
  let (
    _parent_member_vec,
    mut parent_hsm_node_hw_component_count_vec,
    parent_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    shasta_token,
    &user_defined_delta_hw_component_vec,
    parent_hsm_group_name,
    mem_lcm,
  )
  .await?;

  tracing::info!(
    "Parent group '{}' hw component summary: {:?}",
    parent_hsm_group_name,
    parent_hsm_hw_component_summary
  );

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
    crate::cli::commands::apply_hw_cluster_unpin::utils::calculate_target_hsm_unpin(
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
      &[target_hsm_group_name.to_string()],
    )
    .await
    .map_err(|e| {
      Error::Message(format!(
        "Failed to get member vec from target HSM group: {e}"
      ))
    })?;

  target_hsm_node_vec.extend(nodes_to_move.clone());
  target_hsm_node_vec.sort();

  let mut target_hsm_node_hw_component_count_vec =
    get_hsm_node_hw_component_counter(
      backend,
      shasta_token,
      &user_defined_delta_hw_component_vec,
      &target_hsm_node_vec,
      mem_lcm,
    )
    .await;

  target_hsm_node_hw_component_count_vec.sort_by(|a, b| a.0.cmp(&b.0));

  let target_hsm_hw_component_summary =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  show_solution_and_confirm(
    target_hsm_group_name,
    &user_defined_delta_hw_component_vec,
    &target_hsm_node_hw_component_count_vec,
    &target_hsm_hw_component_summary,
  )?;

  if dryrun {
    tracing::info!(
      "Dryrun enabled, not modifying the groups \
       on the system."
    )
  } else {
    for xname in &nodes_to_move {
      backend
        .delete_member_from_group(shasta_token, parent_hsm_group_name, xname)
        .await
        .map_err(|e| {
          Error::Message(format!(
            "Failed to delete member from parent group: {e}"
          ))
        })?;

      let _ = backend
        .add_members_to_group(
          shasta_token,
          target_hsm_group_name,
          &[xname.as_str()],
        )
        .await
        .map_err(|e| {
          Error::Message(format!("Failed to add member to target group: {e}"))
        })?;
    }
  }

  let parent_hsm_node_vec: Vec<String> = parent_hsm_node_hw_component_count_vec
    .iter()
    .map(|(xname, _)| xname.clone())
    .collect();

  print_hsm_group_json(target_hsm_group_name, &target_hsm_node_vec)?;
  print_hsm_group_json(parent_hsm_group_name, &parent_hsm_node_vec)?;

  Ok(())
}

/// Ensure the target HSM group exists, creating it if
/// `create_hsm_group` is set.
async fn ensure_target_group_exists(
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
        return Err(Error::Message(format!(
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
        return Err(Error::Message(
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
      backend
        .add_group(shasta_token, group)
        .await
        .map_err(|e| {
          Error::Message(format!("Unable to create new group: {e}"))
        })?;
      Ok(())
    }
  }
}

/// Compute the final parent HSM hw component summary after
/// subtracting the user-requested deltas. Returns an error
/// if the parent doesn't have enough of any component.
fn compute_final_parent_summary(
  current_summary: &HashMap<String, usize>,
  deltas: &HashMap<String, isize>,
  parent_group_name: &str,
) -> Result<HashMap<String, usize>, Error> {
  let mut final_summary: HashMap<String, usize> = HashMap::new();

  for (hw_component, counter) in deltas {
    let current = *current_summary.get(hw_component).unwrap_or(&0);
    if *counter > current as isize {
      return Err(Error::Message(format!(
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
