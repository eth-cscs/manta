use std::collections::HashMap;

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};

use crate::{
  cli::commands::hw_cluster_common::utils::{
    calculate_hsm_hw_component_summary, get_hsm_node_hw_component_counter,
    resolve_hw_description_to_xnames,
  },
  common::app_context::AppContext,
};

/// Determines whether the hw cluster operation moves nodes
/// into the target (Pin) or releases them back (Unpin).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwClusterMode {
  Pin,
  Unpin,
}

#[allow(clippy::too_many_arguments)]
pub async fn exec(
  mode: HwClusterMode,
  ctx: &AppContext<'_>,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
  delete_empty_parent_hsm_group: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;

  // ********************************************************
  // PREREQUISITES - FORMAT USER INPUT

  let pattern = format!("{}:{}", target_hsm_group_name, pattern);

  log::info!("pattern: {}", pattern);

  // lcm -> used to normalize and quantify memory capacity
  let mem_lcm = 16384; // 1024 * 16

  // Normalize text in lowercase and separate each HSM group
  // hw inventory pattern
  let pattern_lowercase = pattern.to_lowercase();

  let (target_hsm_group_name, pattern_hw_component) =
    pattern_lowercase.split_once(':').context(
      "Invalid pattern format: \
       expected 'group:component:count'",
    )?;

  let pattern_element_vec: Vec<&str> =
    pattern_hw_component.split(':').collect();

  if !pattern_element_vec.len().is_multiple_of(2) {
    bail!(
      "Error in pattern: odd number of elements. \
       Expected pairs of <hw component>:<count>. \
       eg a100:4:epyc:10:instinct:8",
    );
  }

  let mut user_defined_target_hsm_hw_component_count_hashmap: HashMap<
    String,
    usize,
  > = HashMap::new();

  // Check user pattern is of format
  // <hw component>:<quantity> where `hw component` is a
  // string and `quantity` is a number.
  for hw_component_counter in pattern_element_vec.chunks_exact(2) {
    if let Ok(count) = hw_component_counter[1].parse::<usize>() {
      user_defined_target_hsm_hw_component_count_hashmap
        .insert(hw_component_counter[0].to_string(), count);
    } else {
      bail!(
        "Error in pattern. Please make sure to follow \
         <hsm name>:<hw component>:<counter>:... \
         eg <tasna>:a100:4:epyc:10:instinct:8",
      );
    }
  }

  log::info!(
    "User defined hw components with counters: {:?}",
    user_defined_target_hsm_hw_component_count_hashmap
  );

  let mut user_defined_target_hsm_hw_component_vec: Vec<String> =
    user_defined_target_hsm_hw_component_count_hashmap
      .keys()
      .cloned()
      .collect();

  user_defined_target_hsm_hw_component_vec.sort();

  // ********************************************************
  // PREREQUISITES - GET DATA - TARGET HSM

  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      log::debug!("Target HSM group '{}' exists, good.", target_hsm_group_name)
    }
    Err(_) => {
      if create_target_hsm_group {
        log::info!(
          "Target HSM group '{}' does not exist, \
           but the option to create the group has \
           been selected, creating it now.",
          target_hsm_group_name
        );
        if dryrun {
          bail!(
            "Dryrun selected, cannot create the \
             new group and continue.",
          );
        } else {
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
            .context("Unable to create new target HSM group")?;
        }
      } else {
        bail!(
          "Target HSM group '{}' does not exist, \
           but the option to create the group was \
           NOT specified, cannot continue.",
          target_hsm_group_name,
        );
      }
    }
  };

  // Get target HSM group members
  let target_hsm_group_member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      &[target_hsm_group_name.to_string()],
    )
    .await
    .context("Failed to get target HSM group members")?;

  // Get HSM hw component counters for target HSM
  let mut target_hsm_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )> = get_hsm_node_hw_component_counter(
    backend,
    shasta_token,
    &user_defined_target_hsm_hw_component_vec,
    &target_hsm_group_member_vec,
    mem_lcm,
  )
  .await;

  // Sort nodes hw counters by node name
  target_hsm_node_hw_component_count_vec.sort_by(|a, b| a.0.cmp(&b.0));

  // Calculate hw component counters (summary) across all
  // nodes within the HSM group
  let target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  log::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary_hashmap
  );

  // ********************************************************
  // PREREQUISITES - GET DATA - PARENT HSM

  // Get parent HSM group members
  let parent_hsm_group_member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      &[parent_hsm_group_name.to_string()],
    )
    .await
    .context("Failed to get parent HSM group members")?;

  // Get HSM hw component counters for parent HSM
  let mut parent_hsm_node_hw_component_count_vec: Vec<(
    String,
    HashMap<String, usize>,
  )> = get_hsm_node_hw_component_counter(
    backend,
    shasta_token,
    &user_defined_target_hsm_hw_component_vec,
    &parent_hsm_group_member_vec,
    mem_lcm,
  )
  .await;

  // Sort nodes hw counters by node name
  parent_hsm_node_hw_component_count_vec.sort_by(|a, b| a.0.cmp(&b.0));

  // ********************************************************
  // VALIDATE USER INPUT
  // CHECK HARDWARE REQUIREMENTS CAN BE FULFILLED
  // CHECK USER HAS ACCESS TO REQUESTED HW COMPONENTS
  // CHECK USER HAS ENOUGH QUANTITY OF HW RESOURCES

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

  for (hw_component, qty) in &user_defined_target_hsm_hw_component_count_hashmap
  {
    if combined_target_parent_hsm_hw_component_summary_hashmap
      .get(hw_component)
      .is_some_and(|value| value >= qty)
    {
      // User has access to enough resources
    } else {
      bail!(
        "ERROR - there are not enough resources \
         to fulfill user request.",
      );
    }
  }

  // ********************************************************
  // CONVERT HW DESCRIPTION INTO A SET OF NODES IN TARGET HSM

  let (
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
  ) = resolve_hw_description_to_xnames(
    mode,
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
    user_defined_target_hsm_hw_component_count_hashmap,
  )
  .await?;

  // Calculate hw component counters (summary) across all
  // nodes within the HSM group
  let target_hsm_hw_component_summary_hashmap =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  // Calculate hw component counters (summary) across all
  // nodes within the HSM group
  let parent_hsm_hw_component_summary_hashmap =
    calculate_hsm_hw_component_summary(&parent_hsm_node_hw_component_count_vec);

  let target_hsm_node_vec = target_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect::<Vec<String>>();

  let parent_hsm_node_vec = parent_hsm_node_hw_component_count_vec
    .into_iter()
    .map(|(xname, _)| xname)
    .collect::<Vec<String>>();

  // ********************************************************
  // UPDATE TARGET HSM GROUP IN CSM
  log::info!(
    "Updating target HSM group '{}' members",
    target_hsm_group_name
  );
  if dryrun {
    log::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    // The target HSM group will never be empty — the
    // pattern always yields at least one node, so no
    // need to add code to delete it if empty.
    let _ = backend
      .update_group_members(
        shasta_token,
        target_hsm_group_name,
        &target_hsm_group_member_vec
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
        &target_hsm_node_vec
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      )
      .await;
  }

  // ********************************************************
  // UPDATE PARENT GROUP IN CSM
  log::info!(
    "Updating parent HSM group '{}' members",
    parent_hsm_group_name
  );
  if dryrun {
    log::info!(
      "Dry run enabled, not modifying the \
       HSM groups on the system."
    );
  } else {
    // The parent group might be out of resources after
    // this, so check if there are still nodes there
    // and delete it after moving out the resources.
    let parent_group_will_be_empty =
      target_hsm_group_member_vec.len() == parent_hsm_group_member_vec.len();
    let _ = backend
      .update_group_members(
        shasta_token,
        parent_hsm_group_name,
        &parent_hsm_group_member_vec
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
        &parent_hsm_node_vec
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      )
      .await;
    if parent_group_will_be_empty {
      if delete_empty_parent_hsm_group {
        log::info!(
          "Parent HSM group '{}' is now empty and \
           the option to delete empty groups has \
           been selected, removing it.",
          parent_hsm_group_name
        );
        match backend
          .delete_group(shasta_token, parent_hsm_group_name)
          .await
        {
          Ok(_) => {
            log::info!("HSM group removed successfully.")
          }
          Err(e2) => log::debug!(
            "Error removing the HSM group. \
             This always fails, ignore please. \
             Reported: {}",
            e2
          ),
        };
      } else {
        log::debug!(
          "Parent HSM group '{}' is now empty and \
           the option to delete empty groups has \
           NOT been selected, will not remove it.",
          parent_hsm_group_name
        )
      }
    }
  }

  // ********************************************************
  // PRINT SOLUTIONS

  // Print target HSM data
  log::info!(
    "HSM '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary_hashmap
  );

  let target_hsm_group_value = serde_json::json!({
      "label": target_hsm_group_name,
      "decription": "",
      "members": target_hsm_node_vec,
      "tags": []
  });

  println!(
    "{}",
    serde_json::to_string_pretty(&target_hsm_group_value)
      .context("Failed to serialize target HSM group")?
  );

  // Print parent HSM data
  log::info!(
    "HSM '{}' hw component summary: {:?}",
    parent_hsm_group_name,
    parent_hsm_hw_component_summary_hashmap
  );

  let parent_hsm_group_value = serde_json::json!({
      "label": parent_hsm_group_name,
      "decription": "",
      "members": parent_hsm_node_vec,
      "tags": []
  });

  println!(
    "{}",
    serde_json::to_string_pretty(&parent_hsm_group_value)
      .context("Failed to serialize parent HSM group")?
  );

  Ok(())
}
