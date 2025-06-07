use std::{collections::HashMap, sync::Arc, time::Instant};

use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::interfaces::hsm::{
  group::GroupTrait, hardware_inventory::HardwareInventory,
};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::{
  cli::commands::apply_hw_cluster_pin::utils::{
    calculate_hsm_hw_component_summary, calculate_hw_component_scarcity_scores,
    get_hsm_node_hw_component_counter,
  },
  common,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  delete_hsm_group: bool,
) {
  match backend.get_group(shasta_token, target_hsm_group_name).await {
    /* match hsm::group::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_group_name.to_string()),
    )
    .await
    { */
    Ok(_) => {
      log::debug!("The HSM group {} exists, good.", target_hsm_group_name)
    }
    Err(_) => {
      log::error!(
                "HSM group {} does not exist, cannot remove hw from it and cannot continue.",
                target_hsm_group_name.to_string()
            );
      std::process::exit(1);
    }
  }
  let pattern = format!("{}:{}", target_hsm_group_name, pattern);

  log::info!("pattern: {}", pattern);

  // lcm -> used to normalize and quantify memory capacity
  let mem_lcm = 16384; // 1024 * 16

  // Normalize text in lowercase and separate each HSM group hw inventory pattern
  let pattern_lowercase = pattern.to_lowercase();

  let mut pattern_element_vec: Vec<&str> =
    pattern_lowercase.split(':').collect();

  let target_hsm_group_name = pattern_element_vec.remove(0);

  let mut user_defined_delta_hw_component_count_hashmap: HashMap<
    String,
    isize,
  > = HashMap::new();

  // Check user input is correct
  for hw_component_counter in pattern_element_vec.chunks(2) {
    if hw_component_counter[0].parse::<String>().is_ok()
      && hw_component_counter[1].parse::<isize>().is_ok()
    {
      user_defined_delta_hw_component_count_hashmap.insert(
        hw_component_counter[0].parse::<String>().unwrap(),
        hw_component_counter[1].parse::<isize>().unwrap(),
      );
    } else {
      log::error!("Error in pattern. Please make sure to follow <hsm name>:<hw component>:<counter>:... eg <tasna>:a100:4:epyc:10:instinct:8");
      std::process::exit(1);
    }
  }

  log::info!(
    "User defined hw components with counters: {:?}",
    user_defined_delta_hw_component_count_hashmap
  );

  let mut user_defined_delta_hw_component_vec: Vec<String> =
    user_defined_delta_hw_component_count_hashmap
      .keys()
      .cloned()
      .collect();

  user_defined_delta_hw_component_vec.sort();

  // *********************************************************************************************************
  // PREPREQUISITES - GET DATA - TARGET HSM

  // Get target HSM group members
  let target_hsm_group_member_vec: Vec<String> = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      vec![target_hsm_group_name.to_string()],
    )
    .await
    .unwrap();
  /* hsm::group::utils::get_member_vec_from_hsm_group_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      target_hsm_group_name,
  )
  .await; */

  // Get HSM hw component counters for target HSM
  let mut target_hsm_node_hw_component_count_vec =
    get_hsm_hw_node_component_counter(
      backend,
      shasta_token,
      &user_defined_delta_hw_component_vec,
      &target_hsm_group_member_vec,
      mem_lcm,
    )
    .await;
  if target_hsm_node_hw_component_count_vec.is_empty() {
    log::info!(
            "The target HSM group {} is already empty, cannot remove hardware from it.",
            target_hsm_group_name
        );

    if dryrun && delete_hsm_group {
      log::info!("The option to delete empty groups has NOT been selected, or the dryrun has been enabled. We are done with this action.");
      return;
    } else {
      log::info!(
        "The option to delete empty groups has been selected, removing it."
      );
      match backend
                .delete_group(shasta_token, &target_hsm_group_name.to_string())
                .await
            {
                Ok(_) => {
                    log::info!("HSM group removed successfully, we are done with this action.");
                    return;
                }
                Err(e2) => log::debug!(
                    "Error removing the HSM group. This always fails, ignore please. Reported: {}",
                    e2
                ),
            };
    }
  }
  // sort nodes hw counters by node name
  target_hsm_node_hw_component_count_vec.sort_by_key(
    |target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone(),
  );

  // Calculate hw component counters (summary) across all node within the HSM group
  let target_hsm_hw_component_summary: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  log::info!(
    "HSM group '{}' hw component summary: {:?}",
    target_hsm_group_name,
    target_hsm_hw_component_summary
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
  /* hsm::group::utils::get_member_vec_from_hsm_group_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      parent_hsm_group_name,
  )
  .await; */

  // Get HSM hw component counters for parent HSM
  let mut parent_hsm_node_hw_component_count_vec =
    get_hsm_node_hw_component_counter(
      backend,
      shasta_token,
      &user_defined_delta_hw_component_vec,
      &parent_hsm_group_member_vec,
      mem_lcm,
    )
    .await;

  // Sort nodes hw counters by node name
  parent_hsm_node_hw_component_count_vec.sort_by_key(
    |parent_hsm_group_hw_component| parent_hsm_group_hw_component.0.clone(),
  );

  // Calculate hw component counters (summary) across all node within the HSM group
  let parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&parent_hsm_node_hw_component_count_vec);

  log::info!(
    "HSM group '{}' hw component summary: {:?}",
    parent_hsm_group_name,
    parent_hsm_hw_component_summary_hashmap
  );

  // *********************************************************************************************************
  // CALCULATE COMBINED HSM WITH TARGET HSM AND PARENT HSM DATA

  let combined_target_parent_hsm_node_hw_component_count_vec = [
    target_hsm_node_hw_component_count_vec.clone(),
    parent_hsm_node_hw_component_count_vec.clone(),
  ]
  .concat();

  // *********************************************************************************************************
  // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY

  // Get parent HSM group members
  // Calculate nomarlized score for each hw component type in as much HSM groups as possible
  // related to the stakeholders using these nodes
  let combined_target_parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap: HashMap<
        String,
        f32,
    > = calculate_hw_component_scarcity_scores(
        &combined_target_parent_hsm_node_hw_component_count_vec,
    )
    .await;

  // *********************************************************************************************************
  // CALCULATE FINAL HSM GROUP HW COMPONENT COUNTERS (SUMMARY) AND DELTAS Calculate the hw components the target HSM group should have after applying the deltas
  // (removing the hw components from the target hsm spcecified by the user)
  let mut final_target_hsm_hw_component_summary: HashMap<String, usize> =
    HashMap::new();

  for (hw_component, counter) in &user_defined_delta_hw_component_count_hashmap
  {
    let new_counter: usize =
      target_hsm_hw_component_summary.get(hw_component).unwrap()
        - *counter as usize;

    final_target_hsm_hw_component_summary
      .insert(hw_component.to_string(), new_counter);
  }

  // *********************************************************************************************************
  // FIND NODES TO MOVE FROM PARENT TO TARGET HSM GROUP

  // Downscale parent HSM group
  let hw_component_counters_to_move_out_from_target_hsm =
        crate::cli::commands::apply_hw_cluster_unpin::utils::calculate_target_hsm_unpin(
            &final_target_hsm_hw_component_summary.clone(),
            &final_target_hsm_hw_component_summary
                .into_keys()
                .collect::<Vec<String>>(),
            &mut target_hsm_node_hw_component_count_vec,
            &combined_target_parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap,
        );

  // *********************************************************************************************************
  // PREPARE INFORMATION TO SHOW

  let nodes_moved_from_target_hsm =
    hw_component_counters_to_move_out_from_target_hsm
      .iter()
      .map(|(xname, _)| xname)
      .cloned()
      .collect::<Vec<String>>();

  // Get parent HSM group members
  let mut parent_hsm_node_vec: Vec<String> = parent_hsm_group_member_vec;
  parent_hsm_node_vec.extend(nodes_moved_from_target_hsm.clone());

  parent_hsm_node_vec.sort();

  let _parent_hsm_hw_component_summary =
    calculate_hsm_hw_component_summary(&parent_hsm_node_hw_component_count_vec);

  // Calculate hw component counters (summary) across all node within the HSM group
  let target_hsm_hw_component_summary: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  // Get list of xnames in target HSM group
  let target_hsm_node_vec = target_hsm_node_hw_component_count_vec
    .iter()
    .map(|(xname, _)| xname)
    .cloned()
    .collect::<Vec<String>>();

  // *********************************************************************************************************
  // SHOW THE SOLUTION

  log::info!("----- SOLUTION -----");

  log::info!("Hw components in HSM '{}'", target_hsm_group_name);

  log::info!(
    "hsm '{}' hw component counters: {:?}",
    target_hsm_group_name,
    target_hsm_node_hw_component_count_vec
  );

  let hw_configuration_table =
    crate::cli::commands::get_hardware_cluster::get_table(
      &user_defined_delta_hw_component_vec,
      &target_hsm_node_hw_component_count_vec,
    );

  log::info!("\n{hw_configuration_table}");

  let confirm_message = format!(
    "Please check and confirm new hw summary for cluster '{}': {:?}",
    target_hsm_group_name, target_hsm_hw_component_summary
  );

  if Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(confirm_message)
    .interact()
    .unwrap()
  {
    println!("Continue.");
  } else {
    println!("Cancelled by user. Aborting.");
    std::process::exit(0);
  }

  // *********************************************************************************************************
  // UPDATE HSM GROUP MEMBERS IN CSM

  if dryrun {
    log::info!("Dry run enabled, not modifying the HSM groups on the system.")
  } else {
    let target_group_will_be_empty =
      &target_hsm_group_member_vec.len() == &nodes_moved_from_target_hsm.len();
    for xname in nodes_moved_from_target_hsm {
      // TODO: This is creating a new client per xname, look whether this can be improved reusing the client.

      let _ = backend
        .delete_member_from_group(
          shasta_token,
          target_hsm_group_name,
          xname.as_str(),
        )
        .await;
      /* let _ = hsm::group::http_client::delete_member(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          target_hsm_group_name,
          xname.as_str(),
      )
      .await; */

      let _ = backend
        .add_members_to_group(
          shasta_token,
          parent_hsm_group_name,
          vec![xname.as_str()],
        )
        .await;
    }
    if target_group_will_be_empty {
      if delete_hsm_group {
        log::info!("HSM group {} is now empty and the option to delete empty groups has been selected, removing it.",target_hsm_group_name);
        match backend.delete_group(shasta_token,
                                            &target_hsm_group_name.to_string()).await {
                /* match hsm::group::http_client::delete_hsm_group(shasta_token,
                                                                      shasta_base_url,
                                                                      shasta_root_cert,
                                                                      &target_hsm_group_name.to_string())
                    .await { */
                    Ok(_) => log::info!("HSM group removed successfully."),
                    Err(e2) => log::debug!("Error removing the HSM group. This always fails, ignore please. Reported: {}", e2)
                };
      } else {
        log::debug!("HSM group {} is now empty and the option to delete empty groups has NOT been selected, will not remove it.",target_hsm_group_name)
      }
    }
  }

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

pub async fn get_hsm_hw_node_component_counter(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  user_defined_hw_component_vec: &[String],
  hsm_group_member_vec: &[String],
  mem_lcm: u64,
) -> Vec<(String, HashMap<String, usize>)> {
  // Get HSM group members hw configurfation based on user input

  let start = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(5)); // CSM 1.3.1 higher number of concurrent tasks won't

  // Calculate HSM group hw component counters
  // List of node hw component counters belonging to target hsm group
  let mut target_hsm_node_hw_component_count_vec = Vec::new();

  // Get HW inventory details for parent HSM group
  for hsm_member in hsm_group_member_vec.to_owned() {
    let shasta_token_string = shasta_token.to_string(); // TODO: make it static
    let user_defined_hw_component_vec =
      user_defined_hw_component_vec.to_owned();
    let backend_clone = backend.clone();

    let permit = Arc::clone(&sem).acquire_owned().await;

    // println!("user_defined_hw_profile_vec_aux: {:?}", user_defined_hw_profile_vec_aux);
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
      log::error!("Failed procesing/fetching node hw information");
    }
  }

  let duration = start.elapsed();
  log::info!("Time elapsed to calculate hw components is: {:?}", duration);

  target_hsm_node_hw_component_count_vec
}

/// Returns a triple like (<xname>, <list of hw components>, <list of memory capacity>)
/// Note: list of hw components can be either the hw componentn pattern provided by user or the
/// description from the HSM API
/// NOTE: backend it not borrowed because we need to clone it in order to use it across threads
pub async fn get_node_hw_component_count(
  backend: StaticBackendDispatcher,
  shasta_token: String,
  hsm_member: &str,
  user_defined_hw_profile_vec: Vec<String>,
) -> (String, Vec<String>, Vec<u64>) {
  let node_hw_inventory_value = backend
    .get_inventory_hardware_query(
      &shasta_token,
      /* &shasta_base_url,
      &shasta_root_cert, */
      hsm_member,
      None,
      None,
      None,
      None,
      None,
    )
    .await
    .unwrap();
  /* let node_hw_inventory_value = hsm::hw_inventory::hw_component::http_client::get_hw_inventory(
      &shasta_token,
      &shasta_base_url,
      &shasta_root_cert,
      hsm_member,
  )
  .await
  .unwrap(); */

  let node_hw_profile = get_node_hw_properties_from_value(
    &node_hw_inventory_value,
    user_defined_hw_profile_vec.clone(),
  );

  (hsm_member.to_string(), node_hw_profile.0, node_hw_profile.1)
}

/// Returns the properties in hw_property_list found in the node_hw_inventory_value which is
/// HSM hardware inventory API json response
pub fn get_node_hw_properties_from_value(
  node_hw_inventory_value: &Value,
  hw_component_pattern_list: Vec<String>,
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

  /* let hsn_nic_vec =
  hsm::utils::get_list_hsn_nics_model_from_hw_inventory_value(node_hw_inventory_value)
      .unwrap_or_default(); */

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

  let memory_vec = common::hw_inventory_utils::get_list_memory_capacity_from_hw_inventory_value(
        node_hw_inventory_value,
    )
    .unwrap_or_default();

  (node_hw_component_pattern_vec, memory_vec)
}
