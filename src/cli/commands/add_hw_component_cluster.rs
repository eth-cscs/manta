use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::{
  interfaces::hsm::group::GroupTrait, types::Group,
};

use crate::{
  cli::commands::apply_hw_cluster_pin::utils::{
    calculate_hsm_hw_component_summary, calculate_hw_component_scarcity_scores,
    get_hsm_node_hw_component_counter,
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
  create_hsm_group: bool,
) {
  let pattern = format!("{}:{}", target_hsm_group_name, pattern);

  match backend.get_group(shasta_token, target_hsm_group_name).await {
    Ok(_) => {
      log::debug!("The HSM group {} exists, good.", target_hsm_group_name)
    }
    Err(_) => {
      if create_hsm_group {
        log::info!("HSM group {} does not exist, but the option to create the group has been selected, creating it now.", target_hsm_group_name.to_string());
        if dryrun {
          log::error!(
            "Dryrun selected, cannot create the new group and continue."
          );
          std::process::exit(1);
        } else {
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
            .expect("Unable to create new HSM group");
        }
      } else {
        log::error!("HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_group_name.to_string());
        std::process::exit(1);
      }
    }
  };

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
  // PREPREQUISITES - GET DATA - PARENT HSM

  // Get parent HSM group members
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

  // Get HSM hw component counters for target HSM
  let mut parent_hsm_node_hw_component_count_vec =
    get_hsm_node_hw_component_counter(
      backend,
      shasta_token,
      &user_defined_delta_hw_component_vec,
      &parent_hsm_group_member_vec,
      mem_lcm,
    )
    .await;

  // sort nodes hw counters by node name
  parent_hsm_node_hw_component_count_vec.sort_by_key(
    |target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone(),
  );

  /* log::info!(
      "HSM '{}' hw component counters: {:?}",
      parent_hsm_group_name,
      parent_hsm_node_hw_component_count_vec
  ); */

  // Calculate hw component counters (summary) across all node within the HSM group
  let parent_hsm_hw_component_summary: HashMap<String, usize> =
    calculate_hsm_hw_component_summary(&parent_hsm_node_hw_component_count_vec);

  log::info!(
    "HSM group '{}' hw component summary: {:?}",
    parent_hsm_group_name,
    parent_hsm_hw_component_summary
  );

  // *********************************************************************************************************
  // CALCULATE FINAL HSM GROUP HW COMPONENT COUNTERS (SUMMARY) AND DELTAS Calculate the hw components the target HSM group should have after applying the deltas
  // (removing the hw components from the target hsm spcecified by the user)
  let mut final_parent_hsm_hw_component_summary: HashMap<String, usize> =
    HashMap::new();

  for (hw_component, counter) in &user_defined_delta_hw_component_count_hashmap
  {
    let new_counter: usize =
      parent_hsm_hw_component_summary.get(hw_component).unwrap()
        - *counter as usize;

    final_parent_hsm_hw_component_summary
      .insert(hw_component.to_string(), new_counter);
  }

  //*************************************************************************************
  // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY
  // Get parent HSM group members
  // Calculate nomarlized score for each hw component type in as much HSM groups as possible
  // related to the stakeholders using these nodes
  let parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap: HashMap<
    String,
    f32,
  > = calculate_hw_component_scarcity_scores(
    &parent_hsm_node_hw_component_count_vec,
  )
  .await;

  // *********************************************************************************************************
  // FIND NODES TO MOVE FROM PARENT TO TARGET HSM GROUP

  // Downscale parent HSM group
  let hw_component_counters_to_move_out_from_parent_hsm =
        crate::cli::commands::apply_hw_cluster_unpin::utils::calculate_target_hsm_unpin(
            &final_parent_hsm_hw_component_summary.clone(),
            &final_parent_hsm_hw_component_summary
                .into_iter()
                .map(|(hw_component, _)| hw_component)
                .collect::<Vec<String>>(),
            &mut parent_hsm_node_hw_component_count_vec,
            &parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap,
        );

  // *********************************************************************************************************
  // PREPARE INFORMATION TO SHOW

  let nodes_moved_from_parent_hsm =
    hw_component_counters_to_move_out_from_parent_hsm
      .iter()
      .map(|(xname, _)| xname)
      .cloned()
      .collect::<Vec<String>>();

  // Get target HSM group members
  let mut target_hsm_node_vec: Vec<String> = backend
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

  target_hsm_node_vec.extend(nodes_moved_from_parent_hsm.clone());

  target_hsm_node_vec.sort();

  // Get HSM hw component counters for target HSM
  let target_hsm_node_hw_component_count_vec =
    get_hsm_node_hw_component_counter(
      backend,
      shasta_token,
      &user_defined_delta_hw_component_vec,
      &target_hsm_node_vec,
      mem_lcm,
    )
    .await;

  let target_hsm_hw_component_summary =
    calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

  // Get list of xnames in target HSM group
  let parent_hsm_node_vec = parent_hsm_node_hw_component_count_vec
    .iter()
    .map(|(xname, _)| xname)
    .cloned()
    .collect::<Vec<String>>();

  // *********************************************************************************************************
  // SHOW THE SOLUTION

  log::info!("----- SOLUTION -----");

  log::info!("Hw components in HSM '{}'", target_hsm_group_name);

  // get hsm hw component counters for target hsm
  let mut target_hsm_node_hw_component_count_vec =
    get_hsm_node_hw_component_counter(
      backend,
      shasta_token,
      &user_defined_delta_hw_component_vec,
      &target_hsm_node_vec,
      mem_lcm,
    )
    .await;

  // sort nodes hw counters by node name
  target_hsm_node_hw_component_count_vec.sort_by_key(
    |target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone(),
  );

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
    log::info!("Dryrun enabled, not modifying the HSM groups on the system.")
  } else {
    for xname in nodes_moved_from_parent_hsm {
      let _ = backend
        .delete_member_from_group(shasta_token, parent_hsm_group_name, &xname)
        .await
        .unwrap();
      /* let _ = hsm::group::http_client::delete_member(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          parent_hsm_group_name,
          &xname,
      )
      .await; */

      let _ = backend
        .add_members_to_group(shasta_token, target_hsm_group_name, vec![&xname])
        .await
        .unwrap();
      /* let _ = hsm::group::http_client::post_member(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          target_hsm_group_name,
          &xname,
      )
      .await; */
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
