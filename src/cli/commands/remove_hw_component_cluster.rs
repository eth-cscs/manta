use std::{collections::HashMap, sync::Arc, time::Instant};

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::hsm;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::cli::commands::apply_hw_cluster::utils::{
    calculate_hsm_hw_component_summary, calculate_hw_component_scarcity_scores,
    get_hsm_node_hw_component_counter,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    pattern: &str,
) {
    let pattern = format!("{}:{}", target_hsm_group_name, pattern);

    log::info!("pattern: {}", pattern);

    // lcm -> used to normalize and quantify memory capacity
    let mem_lcm = 16384; // 1024 * 16

    // Normalize text in lowercase and separate each HSM group hw inventory pattern
    let pattern_lowercase = pattern.to_lowercase();

    let mut pattern_element_vec: Vec<&str> = pattern_lowercase.split(':').collect();

    let target_hsm_group_name = pattern_element_vec.remove(0);

    let mut user_defined_delta_hw_component_count_hashmap: HashMap<String, isize> = HashMap::new();

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
    let target_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm_group_name,
        )
        .await;

    // Get HSM hw component counters for target HSM
    let mut target_hsm_node_hw_component_count_vec = get_hsm_hw_node_component_counter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &user_defined_delta_hw_component_vec,
        &target_hsm_group_member_vec,
        mem_lcm,
    )
    .await;

    // sort nodes hw counters by node name
    target_hsm_node_hw_component_count_vec
        .sort_by_key(|target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone());

    /* log::info!(
        "HSM '{}' hw component counters: {:?}",
        target_hsm_group_name,
        target_hsm_node_hw_component_count_vec
    ); */

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
    let parent_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            parent_hsm_group_name,
        )
        .await;

    // Get HSM hw component counters for parent HSM
    let mut parent_hsm_node_hw_component_count_vec = get_hsm_node_hw_component_counter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &user_defined_delta_hw_component_vec,
        &parent_hsm_group_member_vec,
        mem_lcm,
    )
    .await;

    // Sort nodes hw counters by node name
    parent_hsm_node_hw_component_count_vec
        .sort_by_key(|parent_hsm_group_hw_component| parent_hsm_group_hw_component.0.clone());

    /* log::info!(
        "HSM '{}' hw component counters: {:?}",
        parent_hsm_group_name,
        parent_hsm_node_hw_component_count_vec
    ); */

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
    let mut final_target_hsm_hw_component_summary: HashMap<String, usize> = HashMap::new();

    for (hw_component, counter) in &user_defined_delta_hw_component_count_hashmap {
        let new_counter: usize =
            target_hsm_hw_component_summary.get(hw_component).unwrap() - *counter as usize;

        final_target_hsm_hw_component_summary.insert(hw_component.to_string(), new_counter);
    }

    // *********************************************************************************************************
    // FIND NODES TO MOVE FROM PARENT TO TARGET HSM GROUP

    // Downscale parent HSM group
    let hw_component_counters_to_move_out_from_target_hsm =
        crate::cli::commands::apply_hw_cluster::utils::downscale_from_final_hsm_group(
            &final_target_hsm_hw_component_summary.clone(),
            &final_target_hsm_hw_component_summary
                .into_keys()
                .collect::<Vec<String>>(),
            &mut target_hsm_node_hw_component_count_vec,
            &combined_target_parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap,
        );

    // *********************************************************************************************************
    // PREPARE INFORMATION TO SHOW

    let nodes_moved_from_target_hsm = hw_component_counters_to_move_out_from_target_hsm
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

    let hw_configuration_table = crate::cli::commands::get_hw_configuration_cluster::get_table(
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
    for xname in nodes_moved_from_target_hsm {
        let _ = hsm::group::shasta::http_client::delete_member(parent_hsm_group_name, &xname).await;

        let _ = hsm::group::shasta::http_client::post_member(target_hsm_group_name, &xname).await;
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
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
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
        let shasta_base_url_string = shasta_base_url.to_string(); // TODO: make it static
        let shasta_root_cert_vec = shasta_root_cert.to_vec();
        let user_defined_hw_component_vec = user_defined_hw_component_vec.to_owned();

        let permit = Arc::clone(&sem).acquire_owned().await;

        // println!("user_defined_hw_profile_vec_aux: {:?}", user_defined_hw_profile_vec_aux);
        tasks.spawn(async move {
            let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885
            get_node_hw_component_count(
                shasta_token_string,
                shasta_base_url_string,
                shasta_root_cert_vec,
                &hsm_member,
                user_defined_hw_component_vec,
            )
            .await
        });
    }

    while let Some(message) = tasks.join_next().await {
        if let Ok(mut node_hw_component_vec_tuple) = message {
            node_hw_component_vec_tuple.1.sort();

            let mut node_hw_component_count_hashmap: HashMap<String, usize> = HashMap::new();

            for node_hw_property_vec in node_hw_component_vec_tuple.1 {
                let count = node_hw_component_count_hashmap
                    .entry(node_hw_property_vec)
                    .or_insert(0);
                *count += 1;
            }

            let node_memory_total_capacity: u64 = node_hw_component_vec_tuple.2.iter().sum();

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

/* // Calculate/groups hw component counters
pub fn calculate_hsm_hw_component_count(
    target_hsm_node_hw_component_count_vec: &Vec<(String, HashMap<String, usize>)>,
) -> HashMap<String, usize> {
    let mut hsm_hw_component_count_hashmap = HashMap::new();

    for (_xname, node_hw_component_count_hashmap) in target_hsm_node_hw_component_count_vec {
        for (hw_component, &qty) in node_hw_component_count_hashmap {
            hsm_hw_component_count_hashmap
                .entry(hw_component.to_string())
                .and_modify(|qty_aux| *qty_aux += qty)
                .or_insert(qty);
        }
    }

    hsm_hw_component_count_hashmap
} */

/* // Calculate/groups hw component counters filtered by user request
pub fn filter_hsm_hw_component_count_based_on_user_input_pattern(
    user_defined_hw_component_vec: &Vec<String>,
    target_hsm_node_hw_component_counter_vec: &Vec<(String, HashMap<String, usize>)>,
) -> HashMap<String, usize> {
    let mut hsm_hw_component_count_hashmap = HashMap::new();

    for x in user_defined_hw_component_vec {
        let mut hsm_hw_component_count = 0;
        for y in target_hsm_node_hw_component_counter_vec {
            hsm_hw_component_count += y.1.get(x).unwrap_or(&0);
        }
        hsm_hw_component_count_hashmap.insert(x.to_string(), hsm_hw_component_count);
    }

    hsm_hw_component_count_hashmap
} */

/* pub fn calculate_all_deltas(
    user_defined_hw_component_counter_hashmap: &HashMap<String, usize>,
    hsm_hw_component_summary_hashmap: &HashMap<String, usize>,
) -> (HashMap<String, isize>, HashMap<String, isize>) {
    let mut hw_components_to_migrate_from_target_hsm_to_parent_hsm: HashMap<String, isize> =
        HashMap::new();

    let mut hw_components_to_migrate_from_parent_hsm_to_target_hsm: HashMap<String, isize> =
        HashMap::new();

    for (user_defined_hw_component, desired_quantity) in user_defined_hw_component_counter_hashmap {
        let current_quantity = hsm_hw_component_summary_hashmap
            .get(user_defined_hw_component)
            .unwrap();

        let delta = (*desired_quantity as isize) - (*current_quantity as isize);

        /* if delta > 0 {
            // Migrate nodes from parent to target HSM group
            hw_components_to_migrate_from_parent_hsm_to_target_hsm
                .insert(user_defined_hw_component.to_string(), -delta);
        } else if delta < 0 {
            // Migrate nodes from target to parent HSM group
            hw_components_to_migrate_from_target_hsm_to_parent_hsm
                .insert(user_defined_hw_component.to_string(), delta);
        } else {
            // No change
        } */

        match delta as i32 {
            1.. =>
            // delta > 0 -> Migrate nodes from parent to target HSM group
            {
                hw_components_to_migrate_from_parent_hsm_to_target_hsm
                    .insert(user_defined_hw_component.to_string(), -delta);
            }
            ..=-1 =>
            // delta < 0 -> Migrate nodes from target to parent HSM group
            {
                hw_components_to_migrate_from_target_hsm_to_parent_hsm
                    .insert(user_defined_hw_component.to_string(), delta);
            }
            0 =>
                // delta == 0 -> Do nothing
                {}
        }
    }

    (
        hw_components_to_migrate_from_target_hsm_to_parent_hsm,
        hw_components_to_migrate_from_parent_hsm_to_target_hsm,
    )
} */

/* pub async fn calculate_scarcity_scores_across_both_target_and_parent_hsm_groups(
    hsm_node_hw_component_count_filtered_by_user_request_vec: &Vec<(
        String,
        HashMap<String, usize>,
    )>,
) -> HashMap<String, f32> {
    let total_num_nodes = hsm_node_hw_component_count_filtered_by_user_request_vec.len();

    let mut hw_component_vec: Vec<&String> =
        hsm_node_hw_component_count_filtered_by_user_request_vec
            .iter()
            .flat_map(|(_, hw_component_counter_hashmap)| hw_component_counter_hashmap.keys())
            .collect();

    hw_component_vec.sort();
    hw_component_vec.dedup();

    let mut hw_component_scarcity_score_hashmap: HashMap<String, f32> = HashMap::new();
    for hw_component in hw_component_vec {
        let mut node_count = 0;

        for (_, hw_component_counter_hashmap) in
            hsm_node_hw_component_count_filtered_by_user_request_vec
        {
            if hw_component_counter_hashmap.contains_key(hw_component) {
                node_count += 1;
            }
        }

        hw_component_scarcity_score_hashmap.insert(
            hw_component.to_string(),
            (total_num_nodes as f32) / (node_count as f32),
        );
    }

    log::info!(
        "Hw component scarcity scores: {:?}",
        hw_component_scarcity_score_hashmap
    );

    hw_component_scarcity_score_hashmap
} */

/* /// Removes as much nodes as it can from the target HSM group Returns a tuple with 2 vecs, the left one is the new target HSM group while the left one is
/// the one containing the nodes removed from the target HSM
pub fn downscale_node_migration(
    user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>, // hw
    // components summary the target hsm group should have according to user requests (this is
    // equivalent to target_hsm_node_hw_component_count_vec minus
    // hw_components_deltas_from_target_hsm_to_parent_hsm). Note hw componets needs to be grouped/filtered
    // based on user input
    user_defined_hw_component_vec: &Vec<String>, // list of hw components the user is asking
    // for
    target_hsm_node_hw_component_count_vec: &mut Vec<(String, HashMap<String, usize>)>, // list
    // of hw component counters in target HSM group
    multiple_hw_component_type_normalized_scores_hashmap: &HashMap<String, f32>, // hw
                                                                                 // component type score for as much hsm groups related to the stakeholders usinjkjjjjkj these
                                                                                 // nodes
) -> Vec<(String, HashMap<String, usize>)> {
    ////////////////////////////////
    // Initialize

    // Calculate hw component counters for the whole HSM group
    let mut target_hsm_hw_component_count_hashmap =
        calculate_hsm_hw_component_count(target_hsm_node_hw_component_count_vec);

    /* let mut deltas: HashMap<String, isize> =
    hw_components_deltas_from_target_hsm_to_parent_hsm.clone(); */
    let mut deltas: HashMap<String, isize> = calculate_all_deltas(
        user_defined_hsm_hw_components_count_hashmap,
        &target_hsm_hw_component_count_hashmap,
    )
    .0;

    // Calculate density scores for each node in HSM
    let target_hsm_node_density_score_hashmap: HashMap<String, usize> =
        calculate_node_density_score(target_hsm_node_hw_component_count_vec);

    // Calculate initial scores
    let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
        calculate_hsm_hw_component_normalized_node_density_score_downscale(
            target_hsm_node_hw_component_count_vec,
            &deltas,
            multiple_hw_component_type_normalized_scores_hashmap,
            &target_hsm_hw_component_count_hashmap,
        );

    if target_hsm_node_score_tuple_vec.is_empty() {
        log::info!("No candidates to choose from");
        return Vec::new();
    }

    let mut nodes_migrated_from_target_hsm: Vec<(String, HashMap<String, usize>)> = Vec::new();

    // Get best candidate
    let (mut best_candidate, mut best_candidate_counters) =
        get_best_candidate_to_downscale_migrate_f32_score(
            &mut target_hsm_node_score_tuple_vec,
            target_hsm_node_hw_component_count_vec,
        );

    // Check if we need to keep iterating
    let mut work_to_do = keep_iterating_downscale(
        user_defined_hsm_hw_components_count_hashmap,
        &best_candidate_counters,
        &deltas,
        &target_hsm_hw_component_count_hashmap,
    );

    ////////////////////////////////
    // Iterate

    let mut iter = 0;

    while work_to_do {
        log::info!("----- ITERATION {} -----", iter);

        log::info!("Deltas: {:?}", deltas);
        // Calculate HSM group hw component counters
        let target_hsm_hw_component_filtered_by_user_request_count_hashmap =
            filter_hsm_hw_component_count_based_on_user_input_pattern(
                user_defined_hw_component_vec,
                target_hsm_node_hw_component_count_vec,
            );
        log::info!(
            "HSM group hw component counters: {:?}",
            target_hsm_hw_component_filtered_by_user_request_count_hashmap
        );
        log::info!(
            "Final hw component counters the user wants: {:?}",
            user_defined_hsm_hw_components_count_hashmap
        );
        log::info!(
            "Best candidate is '{}' with score {} and hw component counters {:?}",
            best_candidate.0,
            target_hsm_node_score_tuple_vec
                .iter()
                .find(|(node, _score)| node.eq(&best_candidate.0))
                .unwrap()
                .1,
            best_candidate_counters
        );

        // Print target hsm group hw configuration in table
        print_table_f32_score(
            user_defined_hw_component_vec,
            target_hsm_node_hw_component_count_vec,
            &target_hsm_node_density_score_hashmap,
            &target_hsm_node_score_tuple_vec,
        );

        ////////////////////////////////
        // Apply changes - Migrate from target to parent HSM

        // Add best candidate to parent HSM group
        nodes_migrated_from_target_hsm
            .push((best_candidate.0.clone(), best_candidate_counters.clone()));

        // Remove best candidate from target HSM grour
        target_hsm_node_hw_component_count_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

        if target_hsm_node_hw_component_count_vec.is_empty() {
            break;
        }

        // Calculate hw component couters for the whole HSM group
        target_hsm_hw_component_count_hashmap =
            calculate_hsm_hw_component_count(target_hsm_node_hw_component_count_vec);

        // Update deltas
        deltas = calculate_all_deltas(
            user_defined_hsm_hw_components_count_hashmap,
            &target_hsm_hw_component_count_hashmap,
        )
        .0;

        // Update scores
        target_hsm_node_score_tuple_vec =
            calculate_hsm_hw_component_normalized_node_density_score_downscale(
                target_hsm_node_hw_component_count_vec,
                // &hw_components_to_migrate_from_target_hsm_to_parent_hsm,
                &deltas,
                multiple_hw_component_type_normalized_scores_hashmap,
                &target_hsm_hw_component_count_hashmap,
            );

        // Remove best candidate from scores
        target_hsm_node_score_tuple_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

        // Get best candidate
        (best_candidate, best_candidate_counters) =
            get_best_candidate_to_downscale_migrate_f32_score(
                &mut target_hsm_node_score_tuple_vec,
                target_hsm_node_hw_component_count_vec,
            );

        // Check if we need to keep iterating
        work_to_do = keep_iterating_downscale(
            user_defined_hsm_hw_components_count_hashmap,
            &best_candidate_counters,
            &deltas,
            &target_hsm_hw_component_count_hashmap,
        );

        iter += 1;
    }

    log::info!("----- FINAL RESULT -----");

    log::info!("No candidates found");

    // Print target hsm group hw configuration in table
    print_table_f32_score(
        user_defined_hw_component_vec,
        target_hsm_node_hw_component_count_vec,
        &target_hsm_node_density_score_hashmap,
        &target_hsm_node_score_tuple_vec,
    );

    nodes_migrated_from_target_hsm
} */

/// Returns a triple like (<xname>, <list of hw components>, <list of memory capacity>)
/// Note: list of hw components can be either the hw componentn pattern provided by user or the
/// description from the HSM API
pub async fn get_node_hw_component_count(
    shasta_token: String,
    shasta_base_url: String,
    shasta_root_cert: Vec<u8>,
    hsm_member: &str,
    user_defined_hw_profile_vec: Vec<String>,
) -> (String, Vec<String>, Vec<u64>) {
    let node_hw_inventory_value = mesa::hsm::hw_inventory::shasta::http_client::get_hw_inventory(
        &shasta_token,
        &shasta_base_url,
        &shasta_root_cert,
        hsm_member,
    )
    .await
    .unwrap();

    let node_hw_profile = get_node_hw_properties_from_value(
        &node_hw_inventory_value,
        user_defined_hw_profile_vec.clone(),
    );

    (hsm_member.to_string(), node_hw_profile.0, node_hw_profile.1)
}

/* // Calculates node score based on hw component density
pub fn calculate_node_density_score(
    hsm_node_hw_component_count_hashmap_target_hsm_vec: &Vec<(String, HashMap<String, usize>)>,
) -> HashMap<String, usize> {
    let mut target_hsm_density_score_hashmap: HashMap<String, usize> = HashMap::new();
    for target_hsm_group_hw_component in hsm_node_hw_component_count_hashmap_target_hsm_vec {
        let node_density_score = target_hsm_group_hw_component.1.values().sum();
        target_hsm_density_score_hashmap
            .insert(target_hsm_group_hw_component.clone().0, node_density_score);
    }

    target_hsm_density_score_hashmap
} */

/* /// Calculates a normalized score for each hw component in HSM group based on component
/// scarcity.
pub fn calculate_hsm_hw_component_normalized_node_density_score_downscale(
    target_hsm_node_hw_component_count_hashmap_vec: &Vec<(String, HashMap<String, usize>)>,
    // hw_component_count_to_migrate_from_one_hsm_to_another_hsm: &HashMap<String, isize>, // deltas
    hw_component_count_requested_by_user: &HashMap<String, isize>, // If this is emtpy, then,
    // we just remove the deltas, otherwise, we need to check new target HSM hw component
    // counters after applying the deltas don't vilate user's request by reducing any hw
    // component counter below to what the user is asking
    target_hsm_hw_component_normalized_scores: &HashMap<String, f32>,
    target_hsm_hw_component_count_hashmap: &HashMap<String, usize>,
) -> Vec<(String, f32)> {
    let mut target_hsm_density_score_hashmap: HashMap<String, f32> = HashMap::new();

    for (xname, node_hw_component_count) in target_hsm_node_hw_component_count_hashmap_vec {
        let mut node_hw_normalize_score = 0f32;

        if !can_node_be_removed_without_violating_user_request(
            node_hw_component_count,
            hw_component_count_requested_by_user,
            target_hsm_hw_component_count_hashmap,
        ) {
            // cannot remove this hw component, otherwise we won't have enough
            // resources to fullfil user request
            node_hw_normalize_score = f32::MIN;
        } else {
            for hw_component in node_hw_component_count.keys() {
                let hw_component_normalize_score: f32 = if hw_component_count_requested_by_user
                    .get(hw_component)
                    .is_some_and(|qty| qty.abs() > 0)
                {
                    /* let hsm_hw_component_count_hashmap: HashMap<String, usize> =
                    calculate_hsm_hw_component_count(
                        target_hsm_node_hw_component_count_hashmap_vec,
                    ); */
                    // found hw component that needs to move out
                    /* let quantity_yet_to_go =
                    hw_components_to_migrate_from_one_hsm_to_another_hsm
                        .get(hw_component)
                        .unwrap()
                        .abs() as usize; */
                    if can_node_be_removed_without_violating_user_request(
                        node_hw_component_count,
                        hw_component_count_requested_by_user,
                        target_hsm_hw_component_count_hashmap,
                    ) {
                        // found hw component in hsm goup with enough quantity to remove
                        100_f32
                            - (*target_hsm_hw_component_normalized_scores
                                .get(hw_component)
                                .unwrap())
                    } else {
                        // cannot remove this hw component, otherwise we won't have enough
                        // resources to fullfil user request
                        -(100_f32
                            - (*target_hsm_hw_component_normalized_scores
                                .get(hw_component)
                                .unwrap()))
                    }
                } else {
                    // found a hw component the user is not asking for
                    -(100_f32
                        - (*target_hsm_hw_component_normalized_scores
                            .get(hw_component)
                            .unwrap()))
                };

                node_hw_normalize_score += hw_component_normalize_score;
            }
        }

        target_hsm_density_score_hashmap.insert(xname.to_string(), node_hw_normalize_score);
    }

    let target_hsm_normalized_density_score_tuple_vec: Vec<(String, f32)> =
        target_hsm_density_score_hashmap.into_iter().collect();

    target_hsm_normalized_density_score_tuple_vec
} */

pub fn get_best_candidate_to_downscale_migrate_f32_score(
    target_hsm_score_vec: &mut [(String, f32)],
    target_hsm_hw_component_vec: &[(String, HashMap<String, usize>)],
) -> ((String, f32), HashMap<String, usize>) {
    target_hsm_score_vec.sort_by_key(|elem| elem.0.clone());
    target_hsm_score_vec.sort_by(|b, a| a.1.partial_cmp(&b.1).unwrap());

    // Get node with highest normalized score (best candidate)
    let best_candidate: (String, f32) = target_hsm_score_vec.first().unwrap().clone();
    /* let best_candidate: (String, f32) = target_hsm_score_vec
    .iter()
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    .unwrap()
    .clone(); */

    let best_candidate_counters = &target_hsm_hw_component_vec
        .iter()
        .find(|(node, _)| node.eq(&best_candidate.0))
        .unwrap()
        .1;

    /* println!(
        "Best candidate is '{}' with a normalized score of {} with alphas {:?}",
        best_candidate.0, highest_normalized_score, best_candidate_counters
    ); */

    (best_candidate, best_candidate_counters.clone())
}

/* pub fn keep_iterating_downscale(
    user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>, // hw components in
    // the target hsm group asked by the user (this is the minimum boundary, we can't provide
    // less than this)
    best_candidate_counters: &HashMap<String, usize>,
    hw_components_deltas_from_target_hsm_to_parent_hsm: &HashMap<String, isize>, // minimum boundaries (we
    // can't provide less that this)
    target_hsm_hw_component_count_hashmap: &HashMap<String, usize>, // list of nodes
                                                                    // and its scores
) -> bool {
    // lower boundaries (hw components counters requested by user) won't get violated. We
    // proceed in checking if it is worthy keep iterating

    // Check if anything to process according to deltas
    if hw_components_deltas_from_target_hsm_to_parent_hsm.is_empty() {
        println!("Deltas list is empty (nothing else to process)");
        return false;
    }

    // Check if ANY hw component in best candidate can be removed (included in deltas). Otherwise, this node should not be
    // considered a candidate because none of its hw component should be removed
    if best_candidate_counters
        .keys()
        .all(|best_candidate_hw_component| {
            !user_defined_hsm_hw_components_count_hashmap.contains_key(best_candidate_hw_component)
        })
    {
        println!("Stop processing because none of the hw components in best candidate should be removed. Best candidate {:?}, hw components to remove {:?}", best_candidate_counters, user_defined_hsm_hw_components_count_hashmap);
        return false;
    }

    // Check if removing best candidate from HSM group would violate user's request by removing
    // more hw components in target hsm group than the user has requested
    for (hw_component, counter) in best_candidate_counters {
        if let Some(user_defined_hw_component_counter) =
            hw_components_deltas_from_target_hsm_to_parent_hsm.get(hw_component)
        {
            let target_hsm_new_hw_component_counter = *target_hsm_hw_component_count_hashmap
                .get(hw_component)
                .unwrap()
                - *best_candidate_counters.get(hw_component).unwrap();

            if (target_hsm_new_hw_component_counter as isize) < *user_defined_hw_component_counter {
                println!("Stop processing because otherwise user will get less hw components ({}) than requested because best candidate has {} and we have {} left", hw_component, best_candidate_counters.get(hw_component).unwrap(), counter);
                return false;
            }
        }
    }

    true
} */

/// Returns the properties in hw_property_list found in the node_hw_inventory_value which is
/// HSM hardware inventory API json response
pub fn get_node_hw_properties_from_value(
    node_hw_inventory_value: &Value,
    hw_component_pattern_list: Vec<String>,
) -> (Vec<String>, Vec<u64>) {
    let processor_vec =
        mesa::hsm::hw_inventory::shasta::utils::get_list_processor_model_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

    let accelerator_vec =
        mesa::hsm::hw_inventory::shasta::utils::get_list_accelerator_model_from_hw_inventory_value(
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

    let memory_vec =
        mesa::hsm::hw_inventory::shasta::utils::get_list_memory_capacity_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

    (node_hw_component_pattern_vec, memory_vec)
}

/* // Given a node hw conter, user request hw conter and HSM hw counters, calculate if it is save
// to remove this node form the HSM and still be able to fullful the user request using the
// rest of nodes in the HSM group. This method is used to validate if HSM group still can
// fulfill user request after removing a node.
pub fn can_node_be_removed_without_violating_user_request(
    node_hw_component_count_hashmap: &HashMap<String, usize>,
    user_request_hw_components_count_hashmap: &HashMap<String, isize>,
    target_hsm_node_hw_components_count_hashmap: &HashMap<String, usize>,
) -> bool {
    for (hw_component, requested_qty) in user_request_hw_components_count_hashmap {
        if let Some(node_hw_component_count) = node_hw_component_count_hashmap.get(hw_component) {
            let target_hsm_hw_component_count: usize = *target_hsm_node_hw_components_count_hashmap
                .get(hw_component)
                .unwrap();

            return target_hsm_hw_component_count as isize - *node_hw_component_count as isize
                >= *requested_qty;
        }
    }

    true
} */

/* pub fn print_table_f32_score(
    user_defined_hw_componet_vec: &[String],
    hsm_hw_pattern_vec: &[(String, HashMap<String, usize>)],
    hsm_density_score_hashmap: &HashMap<String, usize>,
    hsm_score_vec: &[(String, f32)],
) {
    let table = get_table_f32_score(
        user_defined_hw_componet_vec,
        hsm_hw_pattern_vec,
        hsm_density_score_hashmap,
        hsm_score_vec,
    );

    log::info!("\n{table}");
} */

/* pub fn get_table_f32_score(
    user_defined_hw_componet_vec: &[String],
    hsm_hw_pattern_vec: &[(String, HashMap<String, usize>)],
    hsm_density_score_hashmap: &HashMap<String, usize>,
    hsm_score_vec: &[(String, f32)],
) -> comfy_table::Table {
    let hsm_hw_component_vec: Vec<String> = hsm_hw_pattern_vec
        .iter()
        .flat_map(|(_xname, node_pattern_hashmap)| node_pattern_hashmap.keys().cloned())
        .collect();

    let mut all_hw_component_vec =
        [hsm_hw_component_vec, user_defined_hw_componet_vec.to_vec()].concat();

    all_hw_component_vec.sort();
    all_hw_component_vec.dedup();

    let mut table = comfy_table::Table::new();

    table.set_header(
        [
            vec!["Node".to_string()],
            all_hw_component_vec.clone(),
            vec!["Density Score".to_string()],
            vec!["Score".to_string()],
        ]
        .concat(),
    );

    for (xname, node_pattern_hashmap) in hsm_hw_pattern_vec {
        // println!("node_pattern_hashmap: {:?}", node_pattern_hashmap);

        let mut row: Vec<comfy_table::Cell> = Vec::new();
        // Node xname table cell
        row.push(
            comfy_table::Cell::new(xname.clone()).set_alignment(comfy_table::CellAlignment::Center),
        );
        // User hw components table cell
        for hw_component in &all_hw_component_vec {
            if user_defined_hw_componet_vec.contains(hw_component)
                && node_pattern_hashmap.contains_key(hw_component)
            {
                let counter = node_pattern_hashmap.get(hw_component).unwrap();
                row.push(
                    comfy_table::Cell::new(format!("✅ ({})", counter,))
                        .fg(Color::Green)
                        .set_alignment(comfy_table::CellAlignment::Center),
                );
            } else if node_pattern_hashmap.contains_key(hw_component) {
                let counter = node_pattern_hashmap.get(hw_component).unwrap();
                row.push(
                    comfy_table::Cell::new(format!("⚠️ ({})", counter)) // NOTE: emojis
                        // can also be printed using unicode like \u{26A0}
                        .fg(Color::Yellow)
                        .set_alignment(comfy_table::CellAlignment::Center),
                );
            } else {
                // node does not contain hardware but it was requested by the user
                row.push(
                    comfy_table::Cell::new("❌".to_string())
                        .set_alignment(comfy_table::CellAlignment::Center),
                );
            }
        }
        /* for user_defined_hw_component in user_defined_hw_componet_vec {
            if node_pattern_hashmap.contains_key(user_defined_hw_component) {
                let counter = node_pattern_hashmap.get(user_defined_hw_component).unwrap();
                row.push(
                    comfy_table::Cell::new(format!("✅ ({})", counter,))
                        .fg(Color::Green)
                        .set_alignment(comfy_table::CellAlignment::Center),
                );
            } else {
                row.push(
                    comfy_table::Cell::new("❌".to_string())
                        .set_alignment(comfy_table::CellAlignment::Center),
                );
            }
        } */
        // Node density score table cell
        row.push(
            comfy_table::Cell::new(hsm_density_score_hashmap.get(xname).unwrap_or(&0))
                .set_alignment(comfy_table::CellAlignment::Center),
        );
        // Node score table cell
        let node_score = hsm_score_vec
            .iter()
            .find(|(node_name, _)| node_name.eq(xname))
            .unwrap_or(&(xname.to_string(), 0f32))
            .1;
        let node_score_table_cell = if node_score <= 0f32 {
            comfy_table::Cell::new(node_score)
                .set_alignment(comfy_table::CellAlignment::Center)
                .fg(Color::Red)
        } else {
            comfy_table::Cell::new(node_score)
                .set_alignment(comfy_table::CellAlignment::Center)
                .fg(Color::Green)
        };
        row.push(node_score_table_cell);
        table.add_row(row);
    }

    table
} */
