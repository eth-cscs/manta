use std::collections::HashMap;

use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::cli::commands::{
    apply_hw_cluster::{
        scores::calculate_scarcity_scores,
        utils::{
            calculate_all_deltas, calculate_hsm_hw_component_count, downscale_node_migration,
            get_hsm_hw_component_count_filtered_by_user_request, get_hsm_hw_component_counter,
        },
    },
    get_hw_configuration_cluster::get_table_f32_score,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: &str,
    pattern: &str,
) {
    let pattern = format!("{}:{}", hsm_group_name, pattern);

    log::info!("pattern: {}", pattern);

    let parent_hsm_group_name = "nodes_free";

    // lcm -> used to normalize and quantify memory capacity
    let mem_lcm = 16384; // 1024 * 16

    // Normalize text in lowercase and separate each HSM group hw inventory pattern
    let pattern_lowercase = pattern.to_lowercase();

    let mut pattern_element_vec: Vec<&str> = pattern_lowercase.split(':').collect();

    let target_hsm_group_name = pattern_element_vec.remove(0);

    let mut delta_hashmap: HashMap<String, isize> = HashMap::new();

    // Check user input is correct
    for hw_component_counter in pattern_element_vec.chunks(2) {
        if hw_component_counter[0].parse::<String>().is_ok()
            && hw_component_counter[1].parse::<isize>().is_ok()
        {
            delta_hashmap.insert(
                hw_component_counter[0].parse::<String>().unwrap(),
                hw_component_counter[1].parse::<isize>().unwrap(),
            );
        } else {
            log::error!("Error in pattern. Please make sure to follow <hsm name>:<hw component>:<counter>:... eg <tasna>:a100:4:epyc:10:instinct:8");
        }
    }

    log::info!(
        "User defined hw components with counters: {:?}",
        delta_hashmap
    );

    let mut delta_hw_component_vec: Vec<String> = delta_hashmap.keys().cloned().collect();

    delta_hw_component_vec.sort();

    // *********************************************************************************************************
    // PREPREQUISITES - GET DATA

    // Get target HSM group members
    let target_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &target_hsm_group_name,
        )
        .await;

    // Get HSM hw component counters for target HSM
    let mut target_hsm_node_hw_component_count_vec = get_hsm_hw_component_counter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &delta_hw_component_vec,
        &target_hsm_group_member_vec,
        mem_lcm,
    )
    .await;

    // sort nodes hw counters by node name
    target_hsm_node_hw_component_count_vec
        .sort_by_key(|target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone());

    // *********************************************************************************************************
    // TRANSFORM DATA

    // Calculate hw component counters (summary) in HSM
    let target_hsm_hw_component_count_hashmap: HashMap<String, usize> =
        calculate_hsm_hw_component_count(&target_hsm_node_hw_component_count_vec);

    // Calculate hw component counters (summary) in HSM filtered by user request
    let target_hsm_node_hw_component_count_filtered_by_user_request_hashmap: HashMap<
        String,
        usize,
    > = get_hsm_hw_component_count_filtered_by_user_request(
        &delta_hw_component_vec,
        &target_hsm_node_hw_component_count_vec,
    );

    log::info!(
        "HSM '{}' hw component counters filtered by user request: {:?}",
        target_hsm_group_name,
        target_hsm_node_hw_component_count_filtered_by_user_request_hashmap
    );

    // Calculate the hw components the target HSM group should have after applying the deltas
    // (removing the hw components from the target hsm spcecified by the user)
    let mut final_target_hsm_hw_component_count_filtered_by_user_request_hashmap: HashMap<
        String,
        isize,
    > = HashMap::new();
    for (hw_component, counter) in &delta_hashmap {
        let new_counter = *target_hsm_node_hw_component_count_filtered_by_user_request_hashmap
            .get(hw_component)
            .unwrap() as isize
            - counter;
        final_target_hsm_hw_component_count_filtered_by_user_request_hashmap
            .insert(hw_component.to_string(), new_counter);
    }

    // User may ask for resources that does not exists in either target or parent HSM groups they
    // manage, for this reason we are going to clean the hw components in the user request which
    // does not exists in either target or parent HSM group
    delta_hashmap.retain(|hw_component, _qty| {
        target_hsm_hw_component_count_hashmap.contains_key(hw_component)
    });

    let (
        hw_components_deltas_from_target_hsm_to_parent_hsm,
        _hw_components_deltas_from_parent_hsm_to_target_hsm,
    ) = calculate_all_deltas(
        &final_target_hsm_hw_component_count_filtered_by_user_request_hashmap,
        &target_hsm_node_hw_component_count_filtered_by_user_request_hashmap,
    );

    // *********************************************************************************************************
    // VALIDATION
    // Check collective HSM has enough capacity to process user request
    for (hw_component, qty_requested) in &hw_components_deltas_from_target_hsm_to_parent_hsm {
        let qty_available = *target_hsm_hw_component_count_hashmap
            .get(hw_component)
            .unwrap() as isize;
        if qty_available < *qty_requested {
            eprintln!("HSM '{}' does not have enough resources to fulfill user request. User is requesting {} ({}) but only avaiable {}. Exit",parent_hsm_group_name,  hw_component, qty_requested, qty_available);
            std::process::exit(1);
        }
    }

    log::info!("----- TARGET HSM GROUP '{}' -----", target_hsm_group_name);

    let hw_components_to_migrate_from_target_hsm_to_parent_hsm: HashMap<String, isize> =
        delta_hashmap.clone();

    log::info!(
        "Components to move from '{}' to '{}' --> {:?}",
        target_hsm_group_name,
        parent_hsm_group_name,
        hw_components_to_migrate_from_target_hsm_to_parent_hsm
    );

    log::info!(
        "Deltas to move from '{}' to '{}' --> {:?}",
        target_hsm_group_name,
        parent_hsm_group_name,
        hw_components_deltas_from_target_hsm_to_parent_hsm
    );

    //*************************************************************************************
    // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY
    // Get parent HSM group members
    // Calculate nomarlized score for each hw component type in as much HSM groups as possible
    // related to the stakeholders using these nodes
    let multiple_hw_component_type_scores_based_on_scracity_hashmap: HashMap<String, f32> =
        calculate_scarcity_scores(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &delta_hw_component_vec,
            mem_lcm,
            &target_hsm_group_member_vec,
            parent_hsm_group_name,
        )
        .await;

    // *********************************************************************************************************
    // FIND NODES TO MOVE FROM PARENT TO TARGET HSM GROUP

    // Migrate nodes
    let mut hw_component_counters_moved_from_target_hsm = downscale_node_migration(
        &hw_components_deltas_from_target_hsm_to_parent_hsm,
        &final_target_hsm_hw_component_count_filtered_by_user_request_hashmap,
        &delta_hw_component_vec,
        &mut target_hsm_node_hw_component_count_vec,
        &multiple_hw_component_type_scores_based_on_scracity_hashmap,
    );

    // Sort hw components moved form target HSM group
    hw_component_counters_moved_from_target_hsm.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Get xnames form list of hw components moved from target HSM group
    let target_hsm_node_list = hw_component_counters_moved_from_target_hsm
        .iter()
        .map(|(xname, _)| xname.clone())
        .collect::<Vec<String>>()
        .join(", ");

    // *********************************************************************************************************
    // SHOW THE SOLUTION
    log::info!("----- SOLUTION -----");

    let target_hsm_node_vec = target_hsm_node_hw_component_count_vec
        .iter()
        .map(|(xname, _)| xname)
        .cloned()
        .collect::<Vec<String>>();

    let target_hsm_hw_component_count_hashmap =
        calculate_hsm_hw_component_count(&target_hsm_node_hw_component_count_vec);

    let hw_component_count_leaving_hsm_target_hashmap =
        calculate_hsm_hw_component_count(&hw_component_counters_moved_from_target_hsm);

    let hw_configuration_table = get_table_f32_score(
        &delta_hw_component_vec,
        &target_hsm_node_hw_component_count_vec,
    );

    log::info!("\n{hw_configuration_table}");

    let hw_configuration_table = get_table_f32_score(
        &delta_hw_component_vec,
        &hw_component_counters_moved_from_target_hsm,
    );

    log::info!("\n{hw_configuration_table}");

    log::info!(
        "Target HSM '{}' deltas: {:?}",
        target_hsm_group_name,
        hw_components_deltas_from_target_hsm_to_parent_hsm
    );

    let confirm_message = format!(
        "Please check and confirm new hw summary for cluster '{}': {:?}",
        target_hsm_group_name, target_hsm_hw_component_count_hashmap
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

    println!(
        "Target HSM '{}' members left: {}",
        target_hsm_group_name, target_hsm_node_list
    );

    log::info!(
        "Target HSM '{}' members left hw components summary: {:?}",
        target_hsm_group_name,
        hw_component_count_leaving_hsm_target_hashmap
    );

    println!(
        "Target HSM '{}' final members: {}",
        target_hsm_group_name,
        target_hsm_node_vec.join(",")
    );

    println!("Hsm group '{}' un trouched", target_hsm_group_name);
}
