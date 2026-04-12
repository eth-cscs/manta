use std::collections::HashMap;

use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  cli::commands::hw_cluster_common::{
    MEMORY_CAPACITY_LCM,
    utils::{
      calculate_hsm_hw_component_summary,
      calculate_hw_component_scarcity_scores, fetch_hsm_hw_inventory,
      parse_hw_pattern, print_hsm_group_json, show_solution_and_confirm,
    },
  },
  common::{
    app_context::AppContext, authentication::get_api_token,
    authorization::get_groups_names_available,
  },
};

/// Remove hardware components from a cluster group.
pub async fn exec(
  ctx: &AppContext<'_>,
  target_hsm_group_name_arg_opt: Option<&str>,
  parent_hsm_group_name_arg_opt: Option<&str>,
  pattern: &str,
  dryrun: bool,
  delete_hsm_group: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;
  let shasta_token = get_api_token(backend, site_name).await?;
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    target_hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;
  let parent_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    parent_hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  let target_hsm_group_name = target_hsm_group_vec
    .first()
    .context("Target HSM group vec is empty")?;
  let parent_hsm_group_name = parent_hsm_group_vec
    .first()
    .context("Parent HSM group vec is empty")?;

  match backend
    .get_group(&shasta_token, target_hsm_group_name)
    .await
  {
    Ok(_) => {
      log::debug!("The HSM group {} exists, good.", target_hsm_group_name)
    }
    Err(_) => {
      bail!(
        "HSM group {} does not exist, cannot remove hw \
         from it and cannot continue.",
        target_hsm_group_name
      );
    }
  }

  // Parse the hardware pattern
  let pattern = format!("{}:{}", target_hsm_group_name, pattern);
  let pattern_lowercase = pattern.to_lowercase();
  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();
  let target_hsm_group_name = pattern_element_vec.remove(0);

  let (
    user_defined_delta_hw_component_vec,
    user_defined_delta_hw_component_count_hashmap,
  ) = parse_hw_pattern(&pattern_element_vec)?;

  log::info!(
    "User defined hw components with counters: {:?}",
    user_defined_delta_hw_component_count_hashmap
  );

  // Fetch target HSM inventory
  let mem_lcm = MEMORY_CAPACITY_LCM;
  let (
    target_hsm_group_member_vec,
    mut target_hsm_node_hw_component_count_vec,
    target_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    &shasta_token,
    &user_defined_delta_hw_component_vec,
    target_hsm_group_name,
    mem_lcm,
  )
  .await?;

  if target_hsm_node_hw_component_count_vec.is_empty() {
    return handle_empty_target(
      backend,
      &shasta_token,
      target_hsm_group_name,
      dryrun,
      delete_hsm_group,
    )
    .await;
  }

  log::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary
  );

  // Fetch parent HSM inventory
  let (
    parent_hsm_group_member_vec,
    parent_hsm_node_hw_component_count_vec,
    _parent_hsm_hw_component_summary,
  ) = fetch_hsm_hw_inventory(
    backend,
    &shasta_token,
    &user_defined_delta_hw_component_vec,
    parent_hsm_group_name,
    mem_lcm,
  )
  .await?;

  // Calculate combined scarcity scores
  let combined = [
    target_hsm_node_hw_component_count_vec.clone(),
    parent_hsm_node_hw_component_count_vec.clone(),
  ]
  .concat();

  let scarcity_scores = calculate_hw_component_scarcity_scores(&combined).await;

  // Calculate final target HSM hw component summary
  let final_target_hsm_hw_component_summary = compute_final_summary(
    &target_hsm_hw_component_summary,
    &user_defined_delta_hw_component_count_hashmap,
  )?;

  // Find nodes to move out of target
  let hw_counters_to_move =
    crate::cli::commands::apply_hw_cluster_unpin::utils::calculate_target_hsm_unpin(
      &final_target_hsm_hw_component_summary,
      &final_target_hsm_hw_component_summary
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

  // Prepare display data
  let mut parent_hsm_node_vec: Vec<String> = parent_hsm_group_member_vec;
  parent_hsm_node_vec.extend(nodes_to_move.clone());
  parent_hsm_node_vec.sort();

  let target_hsm_hw_component_summary =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  let target_hsm_node_vec: Vec<String> = target_hsm_node_hw_component_count_vec
    .iter()
    .map(|(xname, _)| xname.clone())
    .collect();

  // Show solution and confirm
  show_solution_and_confirm(
    target_hsm_group_name,
    &user_defined_delta_hw_component_vec,
    &target_hsm_node_hw_component_count_vec,
    &target_hsm_hw_component_summary,
  )?;

  // Apply changes
  if dryrun {
    log::info!(
      "Dry run enabled, not modifying the HSM groups \
       on the system."
    )
  } else {
    apply_node_moves(
      backend,
      &shasta_token,
      target_hsm_group_name,
      parent_hsm_group_name,
      &nodes_to_move,
      target_hsm_group_member_vec.len() == nodes_to_move.len(),
      delete_hsm_group,
    )
    .await?;
  }

  print_hsm_group_json(target_hsm_group_name, &target_hsm_node_vec)?;
  print_hsm_group_json(parent_hsm_group_name, &parent_hsm_node_vec)?;

  Ok(())
}

/// Handle the case when target HSM group is already empty.
async fn handle_empty_target(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  dryrun: bool,
  delete_hsm_group: bool,
) -> Result<(), Error> {
  log::info!(
    "The target HSM group {} is already empty, cannot \
     remove hardware from it.",
    target_hsm_group_name
  );

  if dryrun || !delete_hsm_group {
    log::info!(
      "The option to delete empty groups has NOT been \
       selected, or the dryrun has been enabled. We \
       are done with this action."
    );
    return Ok(());
  }

  log::info!(
    "The option to delete empty groups has been \
     selected, removing it."
  );
  match backend
    .delete_group(shasta_token, target_hsm_group_name)
    .await
  {
    Ok(_) => {
      log::info!(
        "HSM group removed successfully, we are \
         done with this action."
      );
    }
    Err(e) => log::debug!(
      "Error removing the HSM group. This always \
       fails, ignore please. Reported: {}",
      e
    ),
  };
  Ok(())
}

/// Compute the final target HSM hw component summary after
/// subtracting the user-defined deltas.
fn compute_final_summary(
  current_summary: &HashMap<String, usize>,
  deltas: &HashMap<String, isize>,
) -> Result<HashMap<String, usize>, Error> {
  let mut final_summary: HashMap<String, usize> = HashMap::new();

  for (hw_component, counter) in deltas {
    let current = *current_summary.get(hw_component).ok_or_else(|| {
      Error::msg(format!(
        "hw component '{}' not found in target HSM \
           hw component summary",
        hw_component
      ))
    })?;

    final_summary.insert(hw_component.to_string(), current - *counter as usize);
  }

  Ok(final_summary)
}

/// Move nodes between HSM groups: delete from target, add
/// to parent. Optionally delete the target group if empty.
async fn apply_node_moves(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
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
      .await
      .context("Failed to add node to parent group")?;
  }

  if target_will_be_empty {
    if delete_hsm_group {
      log::info!(
        "HSM group {} is now empty and the option to \
         delete empty groups has been selected, \
         removing it.",
        target_group
      );
      match backend.delete_group(shasta_token, target_group).await {
        Ok(_) => {
          log::info!("HSM group removed successfully.")
        }
        Err(e) => log::debug!(
          "Error removing the HSM group. This always \
           fails, ignore please. Reported: {}",
          e
        ),
      };
    } else {
      log::debug!(
        "HSM group {} is now empty and the option to \
         delete empty groups has NOT been selected, \
         will not remove it.",
        target_group
      )
    }
  }

  Ok(())
}
