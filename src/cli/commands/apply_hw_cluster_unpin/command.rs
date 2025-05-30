use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};
use std::collections::HashMap;

use crate::{
  cli::commands::apply_hw_cluster_unpin::utils::{
    calculate_hsm_hw_component_summary, get_hsm_node_hw_component_counter,
    resolve_hw_description_to_xnames,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_target_hsm_group: bool,
  delete_empty_parent_hsm_group: bool,
) {
  // *********************************************************************************************************
  // PREPREQUISITES - FORMAT USER INPUT

  let pattern = format!("{}:{}", target_hsm_group_name, pattern);

  log::info!("pattern: {}", pattern);

  // lcm -> used to normalize and quantify memory capacity
  let mem_lcm = 16384; // 1024 * 16

  // Normalize text in lowercase and separate each HSM group hw inventory pattern
  let pattern_lowercase = pattern.to_lowercase();

  let (target_hsm_group_name, pattern_hw_component) =
    pattern_lowercase.split_once(':').unwrap();

  let pattern_element_vec: Vec<&str> =
    pattern_hw_component.split(':').collect();

  let mut user_defined_target_hsm_hw_component_count_hashmap: HashMap<
    String,
    usize,
  > = HashMap::new();

  // Check user input is correct
  for hw_component_counter in pattern_element_vec.chunks(2) {
    if hw_component_counter[0].parse::<String>().is_ok()
      && hw_component_counter[1].parse::<usize>().is_ok()
    {
      user_defined_target_hsm_hw_component_count_hashmap.insert(
        hw_component_counter[0].parse::<String>().unwrap(),
        hw_component_counter[1].parse::<usize>().unwrap(),
      );
    } else {
      log::error!("Error in pattern. Please make sure to follow <hsm name>:<hw component>:<counter>:... eg <tasna>:a100:4:epyc:10:instinct:8");
      std::process::exit(1);
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

  // *********************************************************************************************************
  // PREPREQUISITES - GET DATA - TARGET HSM

  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      log::debug!("Target HSM group '{}' exists, good.", target_hsm_group_name)
    }
    Err(_) => {
      if create_target_hsm_group {
        log::info!("Target HSM group '{}' does not exist, but the option to create the group has been selected, creating it now.", target_hsm_group_name.to_string());
        if dryrun {
          log::error!(
            "Dryrun selected, cannot create the new group and continue."
          );
        } else {
          std::process::exit(1);
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
            .expect("Unable to create new target HSM group");
        }
      } else {
        log::error!("Target HSM group '{}' does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_group_name.to_string());
        std::process::exit(1);
      }
    }
  };

  // Get target HSM group members
  let target_hsm_group_member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      vec![target_hsm_group_name.to_string()],
    )
    .await
    .unwrap();

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
  target_hsm_node_hw_component_count_vec.sort_by_key(
    |target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone(),
  );

  // Calculate hw component counters (summary) across all node within the HSM group
  let target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  log::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary_hashmap
  );

  // *********************************************************************************************************
  // PREPREQUISITES - GET DATA - PARENT HSM

  // Get target HSM group members
  let parent_hsm_group_member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      vec![parent_hsm_group_name.to_string()],
    )
    .await
    .unwrap();

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
  parent_hsm_node_hw_component_count_vec.sort_by_key(
    |parent_hsm_group_hw_component| parent_hsm_group_hw_component.0.clone(),
  );

  // *********************************************************************************************************
  // VALIDATE USER INPUT - CHECK HARDWARE REQUIREMENTS REQUESTED BY USER CAN BE FULFILLED
  // CHECK USER HAS ACCESS TO REQUESTED HW COMPONENTS
  // CHECK USER HAS ACCESS TO ENOUGH QUANTITY OF HW RESOURCES REQUESTED

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
      // We are ok, user has access to enough resources to fullfill its request
    } else {
      // There are not enough resources to fulfill the user request
      eprintln!(
        "ERROR - there are not enough resources to fulfill user request."
      );
      std::process::exit(1);
    }
  }

  // *********************************************************************************************************
  // CONVERT THE HARDWARE DESCRIPTION INTO A SET OF NODES IN TARGET HSM

  let (
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
  ) = resolve_hw_description_to_xnames(
    target_hsm_node_hw_component_count_vec,
    parent_hsm_node_hw_component_count_vec,
    user_defined_target_hsm_hw_component_count_hashmap,
  )
  .await;

  // Calculate hw component counters (summary) across all node within the HSM group
  let target_hsm_hw_component_summary_hashmap =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  // Calculate hw component counters (summary) across all node within the HSM group
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

  // *********************************************************************************************************
  // UPDATE TARGET HSM GROUP IN CSM
  log::info!(
    "Updating target HSM group '{}' members",
    target_hsm_group_name
  );
  if dryrun {
    log::info!("Dry run enabled, not modifying the HSM groups on the system.");
  } else {
    // The target HSM group will never be empty, the way the pattern works it'll always
    // contain at least one node, so there is no need to add code to delete it if it's empty.
    let _ = backend
      .update_group_members(
        shasta_token,
        target_hsm_group_name,
        &target_hsm_group_member_vec,
        &target_hsm_node_vec,
      )
      .await;
  }

  // *********************************************************************************************************
  // UPDATE PARENT GROUP IN CSM
  log::info!(
    "Updating parent HSM group '{}' members",
    parent_hsm_group_name
  );
  if dryrun {
    log::info!("Dry run enabled, not modifying the HSM groups on the system.");
  } else {
    // The parent group might be out of resources after applying this, so it's safe to check
    // if there are still nodes there and, delete it after moving out the resources.
    let parent_group_will_be_empty =
      &target_hsm_group_member_vec.len() == &parent_hsm_group_member_vec.len();
    let _ = backend
      .update_group_members(
        shasta_token,
        parent_hsm_group_name,
        &parent_hsm_group_member_vec,
        &parent_hsm_node_vec,
      )
      .await;
    if parent_group_will_be_empty {
      if delete_empty_parent_hsm_group {
        log::info!("Parent HSM group '{}' is now empty and the option to delete empty groups has been selected, removing it.",parent_hsm_group_name);
        match backend.delete_group(shasta_token,
                             &parent_hsm_group_name.to_string())
                    .await {
                    Ok(_) => log::info!("HSM group removed successfully."),
                    Err(e2) => log::debug!("Error removing the HSM group. This always fails, ignore please. Reported: {}", e2)
                };
      } else {
        log::debug!("Parent HSM group '{}' is now empty and the option to delete empty groups has NOT been selected, will not remove it.",parent_hsm_group_name)
      }
    }
  }
  // *********************************************************************************************************
  // RETURN VALUES

  // *********************************************************************************************************
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
    serde_json::to_string_pretty(&target_hsm_group_value).unwrap()
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
    serde_json::to_string_pretty(&parent_hsm_group_value).unwrap()
  );
}
