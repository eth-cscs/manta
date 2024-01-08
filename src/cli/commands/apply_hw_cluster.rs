use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::Semaphore;

use crate::cli::commands::apply_hw_cluster::utils::{
    calculate_all_deltas, calculate_hsm_hw_component_count,
    calculate_hsm_hw_component_normalized_density_score_from_hsm_node_hw_component_count_vec,
    calculate_hsm_total_number_hw_components, get_hsm_hw_component_count_filtered_by_user_request,
    get_node_hw_component_count, upscale_node_migration,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    pattern: &str,
) {
    let pattern = format!("{}:{}", target_hsm_group_name, pattern);

    log::info!("pattern: {}", pattern);

    let parent_hsm_group_name = "nodes_free";

    // lcm -> used to normalize and quantify memory capacity
    let mem_lcm = 16384; // 1024 * 16

    // Normalize text in lowercase and separate each HSM group hw inventory pattern
    let pattern_lowercase = pattern.to_lowercase();

    let mut pattern_element_vec: Vec<&str> = pattern_lowercase.split(':').collect();

    let target_hsm_group_name = pattern_element_vec.remove(0);

    let mut user_defined_target_hsm_hw_component_count_hashmap: HashMap<String, usize> =
        HashMap::new();

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
    // PREPREQUISITES TARGET HSM GROUP

    // Get target HSM group details
    let hsm_group_target_value: Value = mesa::hsm::group::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_group_name.to_string()),
    )
    .await
    .unwrap()
    .first()
    .unwrap_or(&json!({
        "label": target_hsm_group_name,
        "description": "",
        "members": {
            "ids": []
        }
    }))
    .clone();

    // Get target HSM group members
    let hsm_group_target_members =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_value(
            &hsm_group_target_value,
        );

    // Get HSM group members hw configurfation based on user input
    let start = Instant::now();

    let mut tasks = tokio::task::JoinSet::new();

    let sem = Arc::new(Semaphore::new(5)); // CSM 1.3.1 higher number of concurrent tasks won't

    // List of node hw component counters belonging to target hsm group
    let mut target_hsm_node_hw_component_count_vec = Vec::new();

    // Get HW inventory details for target HSM group
    for hsm_member in hsm_group_target_members.clone() {
        let shasta_token_string = shasta_token.to_string(); // TODO: make it static
        let shasta_base_url_string = shasta_base_url.to_string(); // TODO: make it static
        let shasta_root_cert_vec = shasta_root_cert.to_vec();
        let user_defined_hw_component_vec = user_defined_target_hsm_hw_component_count_hashmap
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .clone();

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
    log::info!(
        "Time elapsed to calculate actual_hsm_node_hw_profile_vec in '{}' is: {:?}",
        target_hsm_group_name,
        duration
    );

    // Sort nodes hw counters by node name
    target_hsm_node_hw_component_count_vec
        .sort_by_key(|target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone());

    // Calculate hw component counters in HSM filtered by user request
    let target_hsm_hw_component_count_filtered_by_user_request_hashmap: HashMap<String, usize> =
        get_hsm_hw_component_count_filtered_by_user_request(
            &user_defined_target_hsm_hw_component_vec,
            &target_hsm_node_hw_component_count_vec,
        );

    println!(
        "HSM '{}' hw component counters filtered by user request: {:?}",
        target_hsm_group_name, target_hsm_hw_component_count_filtered_by_user_request_hashmap
    );

    // *********************************************************************************************************
    // PREREQUISITES PARENT HSM GROUP

    // Get parent HSM group details
    let hsm_group_parent_value = mesa::hsm::group::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&parent_hsm_group_name.to_string()),
    )
    .await
    .unwrap()
    .first()
    .unwrap()
    .clone();

    // Get target HSM group members
    let hsm_group_parent_members =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_value(
            &hsm_group_parent_value,
        );

    // Get HSM group members hw configurfation based on user input
    let start = Instant::now();

    let mut tasks = tokio::task::JoinSet::new();

    let sem = Arc::new(Semaphore::new(5)); // CSM 1.3.1 higher number of concurrent tasks won't
                                           // make it faster

    // List of node hw component counters belonging to parent hsm group
    let mut parent_hsm_node_hw_component_count_vec = Vec::new();

    // Get HW inventory details for parent HSM group
    for hsm_member in hsm_group_parent_members.clone() {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();
        let user_defined_hw_component_vec = user_defined_target_hsm_hw_component_count_hashmap
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .clone();

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

            parent_hsm_node_hw_component_count_vec.push((
                node_hw_component_vec_tuple.0,
                node_hw_component_count_hashmap,
            ));
        } else {
            log::error!("Failed procesing/fetching node hw information");
        }
    }

    let duration = start.elapsed();
    log::info!(
        "Time elapsed to calculate actual_hsm_node_hw_profile_vec in '{}' is: {:?}",
        parent_hsm_group_name,
        duration
    );

    // Sort nodes hw counters by node name
    parent_hsm_node_hw_component_count_vec
        .sort_by_key(|parent_hsm_group_hw_component| parent_hsm_group_hw_component.0.clone());

    // *********************************************************************************************************
    // COLLECTIVE DATA TO HELP CALCULATING SCORES

    // Calculate hs component counters for target and parent hsm groups
    let mut target_parent_hsm_node_hw_component_count_vec = [
        target_hsm_node_hw_component_count_vec.clone(),
        parent_hsm_node_hw_component_count_vec.clone(),
    ]
    .concat();

    // Calculate hw component counters in HSM
    let target_parent_hsm_hw_component_count_hashmap: HashMap<String, usize> =
        calculate_hsm_hw_component_count(&target_parent_hsm_node_hw_component_count_vec);

    // Calculate hw component counters in HSM filtered by user request
    let target_parent_hsm_hw_component_count_filtered_by_user_request_hashmap: HashMap<
        String,
        usize,
    > = get_hsm_hw_component_count_filtered_by_user_request(
        &user_defined_target_hsm_hw_component_vec,
        &target_parent_hsm_node_hw_component_count_vec,
    );

    println!(
        "HSM 'collective' hw component counters filtered by user request: {:?}",
        target_parent_hsm_hw_component_count_filtered_by_user_request_hashmap
    );

    // Calculate total number of hw components in HSM
    let target_parent_hsm_total_number_hw_components: usize =
        calculate_hsm_total_number_hw_components(&target_parent_hsm_node_hw_component_count_vec);

    // Calculate nomarlized score for each hw component in HSM group
    let target_parent_hsm_hw_component_normalized_scores_hashmap =
        calculate_hsm_hw_component_normalized_density_score_from_hsm_node_hw_component_count_vec(
            &target_parent_hsm_node_hw_component_count_vec,
            target_parent_hsm_total_number_hw_components,
        );

    user_defined_target_hsm_hw_component_count_hashmap.retain(|hw_component, _qty| {
        target_parent_hsm_hw_component_count_hashmap.contains_key(hw_component)
    });

    // *********************************************************************************************************
    // VALIDATION
    // Check collective HSM has enough capacity to process user request
    for (hw_component, qty_requested) in &user_defined_target_hsm_hw_component_count_hashmap {
        let qty_available = target_parent_hsm_hw_component_count_hashmap
            .get(hw_component)
            .unwrap();
        if qty_available < qty_requested {
            eprintln!("HSM 'collective' does not have enough resources to fulfill user request. User is requesting {} ({}) but only avaiable {}. Exit", hw_component, qty_requested, qty_available);
            std::process::exit(1);
        }
    }

    // *********************************************************************************************************
    // FIND NODES TO MOVE FROM PARENT TO TARGET HSM GROUP

    println!(
        "\n----- PARENT HSM GROUP '{}' -----\n",
        parent_hsm_group_name
    );

    let hw_components_to_migrate_from_parent_hsm_to_target_hsm: HashMap<String, isize> =
        user_defined_target_hsm_hw_component_count_hashmap
            .iter()
            .map(|(hw_inventory, count)| (hw_inventory.to_string(), -(*count as isize)))
            .collect();

    println!(
        "Components to move from '{}' to '{}' --> {:?}",
        parent_hsm_group_name,
        target_hsm_group_name,
        hw_components_to_migrate_from_parent_hsm_to_target_hsm
    );

    println!(
        "Components to move from '{}' to '{}' --> {:?}",
        parent_hsm_group_name,
        target_hsm_group_name,
        user_defined_target_hsm_hw_component_count_hashmap
    );

    // *********************************************************************************************************
    // FIND NODES TO MOVE FROM PARENT TO TARGET HSM GROUP

    // Migrate nodes
    let hw_component_counters_to_move_out_from_target_parent_hsm = upscale_node_migration(
        &user_defined_target_hsm_hw_component_count_hashmap,
        &user_defined_target_hsm_hw_component_vec,
        &mut target_parent_hsm_node_hw_component_count_vec,
        &target_parent_hsm_hw_component_normalized_scores_hashmap,
    );

    // *********************************************************************************************************
    // PREPARE INFORMATION TO SHOW

    // Get hw component counters for target HSM group
    let mut target_hsm_node_hw_component_count_vec =
        target_parent_hsm_node_hw_component_count_vec.clone();

    // Sort target HSM group details
    target_hsm_node_hw_component_count_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Get hw component summary for target HSM group
    let target_hsm_hw_component_count_hashmap =
        calculate_hsm_hw_component_count(&hw_component_counters_to_move_out_from_target_parent_hsm);

    // Get hw component counters for parent HSM group
    let mut parent_hsm_node_hw_component_count_vec =
        hw_component_counters_to_move_out_from_target_parent_hsm.clone();

    // Sort parent HSM group details
    parent_hsm_node_hw_component_count_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Get hw component summary for parent HSM group
    let parent_hsm_hw_component_count_hashmap =
        calculate_hsm_hw_component_count(&target_parent_hsm_node_hw_component_count_vec);

    // *********************************************************************************************************
    // SHOW THE SOLUTION

    log::info!("----- SOLUTION -----");

    log::info!("Hw components in HSM '{}'", parent_hsm_group_name);

    let hw_configuration_table = crate::cli::commands::get_hw_configuration_cluster::get_table(
        &user_defined_target_hsm_hw_component_vec,
        &parent_hsm_node_hw_component_count_vec,
    );

    log::info!("\n{hw_configuration_table}");

    log::info!(
        "Hw components in HSM '{}': {:?}",
        parent_hsm_group_name,
        parent_hsm_hw_component_count_hashmap
    );

    log::info!(
        "Target HSM '{}' members: {}",
        parent_hsm_group_name,
        parent_hsm_node_hw_component_count_vec
            .into_iter()
            .map(|node_hw_component| node_hw_component.0)
            .collect::<Vec<String>>()
            .join(", ")
    );

    log::info!("Hw components in HSM '{}'", target_hsm_group_name);

    let hw_configuration_table = crate::cli::commands::get_hw_configuration_cluster::get_table(
        &user_defined_target_hsm_hw_component_vec,
        &target_hsm_node_hw_component_count_vec,
    );

    log::info!("\n{hw_configuration_table}");

    log::info!(
        "Hw components in HSM '{}': {:?}",
        target_hsm_group_name,
        target_hsm_hw_component_count_hashmap
    );

    log::info!(
        "Target HSM '{}' members: {}",
        target_hsm_group_name,
        target_hsm_node_hw_component_count_vec
            .into_iter()
            .map(|node_hw_component| node_hw_component.0)
            .collect::<Vec<String>>()
            .join(", ")
    );
}

pub mod utils {
    use std::collections::HashMap;

    use comfy_table::Color;
    use serde_json::Value;

    /// Removes as much nodes as it can from the parent HSM group
    /// Returns a tuple with 2 vecs, the left one is the new parent HSM group while the left one is
    /// the one containing the nodes removed from the parent HSM
    pub fn upscale_node_migration(
        user_defined_hsm_hw_component_count_hashmap: &HashMap<String, usize>,
        user_defined_hw_component_vec: &Vec<String>,
        parent_hsm_node_hw_component_count_vec: &mut Vec<(String, HashMap<String, usize>)>,
        parent_hsm_hw_component_type_normalized_scores_hashmap: &HashMap<String, f32>,
    ) -> Vec<(String, HashMap<String, usize>)> {
        ////////////////////////////////
        // Initialize

        // Calculate hw component counters for the whole HSM group
        let mut target_hsm_hw_component_count_hashmap =
            calculate_hsm_hw_component_count(parent_hsm_node_hw_component_count_vec);

        let mut hw_components_to_migrate_from_parent_hsm_to_target_hsm: HashMap<String, isize> =
            calculate_all_deltas(
                user_defined_hsm_hw_component_count_hashmap,
                &target_hsm_hw_component_count_hashmap,
            )
            .0;

        // Calculate density scores for each node in HSM
        let parent_hsm_density_score_hashmap: HashMap<String, usize> =
            calculate_node_density_score(&parent_hsm_node_hw_component_count_vec);

        let mut parent_hsm_score_tuple_vec =
            calculate_hsm_hw_component_normalized_node_density_score_upscale(
                parent_hsm_node_hw_component_count_vec,
                &hw_components_to_migrate_from_parent_hsm_to_target_hsm,
                // user_defined_hw_component_count_hashmap,
                parent_hsm_hw_component_type_normalized_scores_hashmap,
                // &parent_hsm_hw_component_count_hashmap,
                // parent_hsm_total_number_hw_components,
            );

        if parent_hsm_score_tuple_vec.is_empty() {
            log::info!("No candidates to choose from");
            return Vec::new();
        }

        let mut nodes_migrated_from_parent_hsm: Vec<(String, HashMap<String, usize>)> = Vec::new();

        // Get best candidate
        let (mut best_candidate, mut best_candidate_counters) =
            get_best_candidate_to_upscale_migrate_f32_score(
                &mut parent_hsm_score_tuple_vec,
                parent_hsm_node_hw_component_count_vec,
            );

        // Check if we need to keep iterating
        let mut work_to_do =
            keep_iterating_upscale(&hw_components_to_migrate_from_parent_hsm_to_target_hsm);

        ////////////////////////////////
        // Itarate

        let mut iter = 0;

        while work_to_do {
            log::info!("----- ITERATION {} -----", iter);

            log::info!(
                "HW component counters requested by user: {:?}",
                user_defined_hsm_hw_component_count_hashmap
            );
            // Calculate HSM group hw component counters
            let parent_hsm_hw_component_count_hashmap =
                get_hsm_hw_component_count_filtered_by_user_request(
                    user_defined_hw_component_vec,
                    parent_hsm_node_hw_component_count_vec,
                );
            log::info!(
                "HSM group hw component counters: {:?}",
                parent_hsm_hw_component_count_hashmap
            );
            log::info!(
                "HW component counters yet to remove: {:?}",
                hw_components_to_migrate_from_parent_hsm_to_target_hsm
            );
            log::info!(
                "Best candidate is '{}' with score {} and hw component counters {:?}\n",
                best_candidate.0,
                parent_hsm_score_tuple_vec
                    .iter()
                    .find(|(node, _score)| node.eq(&best_candidate.0))
                    .unwrap()
                    .1,
                best_candidate_counters
            );

            // Print target hsm group hw configuration in table
            print_table_f32_score(
                user_defined_hw_component_vec,
                parent_hsm_node_hw_component_count_vec,
                &parent_hsm_density_score_hashmap,
                &parent_hsm_score_tuple_vec,
            );

            ////////////////////////////////
            // Apply changes - Migrate from target to parent HSM

            // Add best candidate to parent HSM group
            nodes_migrated_from_parent_hsm
                .push((best_candidate.0.clone(), best_candidate_counters.clone()));

            // Remove best candidate from target HSM group
            parent_hsm_node_hw_component_count_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

            if parent_hsm_node_hw_component_count_vec.is_empty() {
                break;
            }

            // Calculate hw component couters for the whole HSM group
            target_hsm_hw_component_count_hashmap =
                calculate_hsm_hw_component_count(parent_hsm_node_hw_component_count_vec);

            hw_components_to_migrate_from_parent_hsm_to_target_hsm = calculate_all_deltas(
                user_defined_hsm_hw_component_count_hashmap,
                &target_hsm_hw_component_count_hashmap,
            )
            .0;

            // Update scores
            parent_hsm_score_tuple_vec =
                calculate_hsm_hw_component_normalized_node_density_score_upscale(
                    parent_hsm_node_hw_component_count_vec,
                    &hw_components_to_migrate_from_parent_hsm_to_target_hsm,
                    // user_defined_hw_component_count_hashmap,
                    parent_hsm_hw_component_type_normalized_scores_hashmap,
                    // &parent_hsm_hw_component_count_hashmap,
                    // parent_hsm_total_number_hw_components,
                );

            // Remove best candidate from scores
            parent_hsm_score_tuple_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

            // Get best candidate
            (best_candidate, best_candidate_counters) =
                get_best_candidate_to_upscale_migrate_f32_score(
                    &mut parent_hsm_score_tuple_vec,
                    parent_hsm_node_hw_component_count_vec,
                );

            // Check if we need to keep iterating
            work_to_do =
                keep_iterating_upscale(&hw_components_to_migrate_from_parent_hsm_to_target_hsm);

            iter += 1;
        }

        log::info!("----- FINAL RESULT -----");

        log::info!("No candidates found");

        // Print target hsm group hw configuration in table
        print_table_f32_score(
            user_defined_hw_component_vec,
            parent_hsm_node_hw_component_count_vec,
            &parent_hsm_density_score_hashmap,
            &parent_hsm_score_tuple_vec,
        );

        nodes_migrated_from_parent_hsm
    }

    pub fn keep_iterating_downscale(
        hw_components_to_migrate_from_target_hsm_to_parent_hsm: &HashMap<String, isize>,
        best_candidate_counters: &HashMap<String, usize>,
    ) -> bool {
        let mut work_to_do = true;

        // Check best candidate has hw components to remove. Otherwise, this node should not be
        // considered a candidate
        // TODO: move this to the function that selects best candidate?
        if best_candidate_counters
            .keys()
            .all(|best_candidate_hw_component| {
                !hw_components_to_migrate_from_target_hsm_to_parent_hsm
                    .contains_key(best_candidate_hw_component)
            })
        {
            log::info!("Stop processing because none of the hw components in best candidate should be removed. Best candidate {:?}, hw components to remove {:?}", best_candidate_counters, hw_components_to_migrate_from_target_hsm_to_parent_hsm);
            work_to_do = false;
        }

        for (hw_component, quantity) in hw_components_to_migrate_from_target_hsm_to_parent_hsm {
            /* println!("\nDEBUG - hw_component: {}", hw_component);
            println!("DEBUG - quantity: {}", quantity);
            println!(
                "DEBUG - best_candidate_counters: {:?}\n",
                best_candidate_counters
            ); */
            if best_candidate_counters.get(hw_component).is_some()
                && quantity.unsigned_abs() < *best_candidate_counters.get(hw_component).unwrap()
            {
                println!("Stop processing because otherwise user will get less hw components ({}) than requested because best candidate has {} and we have {} left", hw_component, best_candidate_counters.get(hw_component).unwrap(), quantity.abs());
                work_to_do = false;
                break;
            }

            /* if quantity.abs() > 0
                && best_candidate_counters.get(hw_component).is_some()
                && best_candidate_counters.get(hw_component).unwrap() <= &(quantity.unsigned_abs())
            {
                work_to_do = true;
                break;
            } */
        }

        work_to_do
    }

    pub fn keep_iterating_upscale(
        hw_components_to_migrate_from_target_hsm_to_parent_hsm: &HashMap<String, isize>,
    ) -> bool {
        /* println!("DEBUG - inspecting if need to continue iterating ...");
        println!(
            "DEBUG - hw_components_to_migrate_from_target_hsm_to_parent_hsm: {:?}",
            hw_components_to_migrate_from_target_hsm_to_parent_hsm
        );
        println!("DEBUG - best_candidate_counters: {:?}", best_candidate_counters); */

        let mut work_to_do = false;

        for quantity in hw_components_to_migrate_from_target_hsm_to_parent_hsm.values() {
            if quantity.abs() > 0
            /* && best_candidate_counters.get(hw_component).is_some()
            && best_candidate_counters.get(hw_component).unwrap() >= &(quantity.unsigned_abs()) */
            {
                work_to_do = true;
                break;
            }
        }

        work_to_do
    }

    pub fn get_best_candidate_to_upscale_migrate_f32_score(
        parent_hsm_score_vec: &mut [(String, f32)],
        parent_hsm_hw_component_vec: &[(String, HashMap<String, usize>)],
    ) -> ((String, f32), HashMap<String, usize>) {
        parent_hsm_score_vec.sort_by(|b, a| a.1.partial_cmp(&b.1).unwrap());

        // Get node with highest normalized score (best candidate)
        let best_candidate: (String, f32) = parent_hsm_score_vec
            .iter_mut()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .clone();

        let best_candidate_counters = &parent_hsm_hw_component_vec
            .iter()
            .find(|(node, _)| node.eq(&best_candidate.0))
            .unwrap()
            .1;

        (best_candidate, best_candidate_counters.clone())
    }

    // Calculates node score based on hw component density
    pub fn calculate_node_density_score(
        node_hw_component_count_hashmap_target_hsm_vec: &Vec<(String, HashMap<String, usize>)>,
    ) -> HashMap<String, usize> {
        let mut target_hsm_density_score_hashmap: HashMap<String, usize> = HashMap::new();
        for target_hsm_group_hw_component in node_hw_component_count_hashmap_target_hsm_vec {
            let node_density_score = target_hsm_group_hw_component.1.values().sum();
            target_hsm_density_score_hashmap
                .insert(target_hsm_group_hw_component.clone().0, node_density_score);
        }

        target_hsm_density_score_hashmap
    }

    pub fn calculate_hsm_hw_component_normalized_node_density_score_upscale(
        hsm_node_hw_component_count_hashmap_vec: &Vec<(String, HashMap<String, usize>)>,
        hw_components_to_migrate_from_one_hsm_to_another_hsm: &HashMap<String, isize>,
        // hw_component_count_requested_by_user: &HashMap<String, usize>,
        hsm_hw_component_normalized_scores: &HashMap<String, f32>,
        // hsm_hw_component_count_hashmap: &HashMap<String, usize>,
        // hsm_hw_component_count: usize,
    ) -> Vec<(String, f32)> {
        let mut hsm_density_score_hashmap: HashMap<String, f32> = HashMap::new();

        for (xname, node_hw_component_count) in hsm_node_hw_component_count_hashmap_vec {
            let mut node_hw_normalize_score = 0f32;

            for hw_component in node_hw_component_count.keys() {
                let hw_component_normalize_score: f32 =
                    if hw_components_to_migrate_from_one_hsm_to_another_hsm
                        .get(hw_component)
                        .is_some_and(|qty| qty.abs() > 0)
                    {
                        // found hw component that needs to move out
                        100_f32
                            - (*hsm_hw_component_normalized_scores
                                .get(hw_component)
                                .unwrap())
                    } else {
                        // found a hw component the user is not asking for
                        /* println!(
                            "DEBUG - hw component: {} to look for in {:?}",
                            hw_component, hsm_hw_component_normalized_scores
                        ); */
                        -(100_f32
                            - (*hsm_hw_component_normalized_scores
                                .get(hw_component)
                                .unwrap()))
                    };

                /* println!(
                    "DEBUG - node: {}, hw component: {}, normalized score: {}",
                    xname, hw_component, hw_component_normalize_score
                ); */
                node_hw_normalize_score += hw_component_normalize_score;
            }
            // }

            hsm_density_score_hashmap.insert(xname.to_string(), node_hw_normalize_score);
        }

        let target_hsm_normalized_density_score_tuple_vec: Vec<(String, f32)> =
            hsm_density_score_hashmap
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect();

        target_hsm_normalized_density_score_tuple_vec
    }

    pub fn calculate_all_deltas(
        user_defined_hw_component_counter_hashmap: &HashMap<String, usize>,
        hsm_hw_component_summary_hashmap: &HashMap<String, usize>,
    ) -> (HashMap<String, isize>, HashMap<String, isize>) {
        let mut hw_components_to_migrate_from_target_hsm_to_parent_hsm: HashMap<String, isize> =
            HashMap::new();

        let mut hw_components_to_migrate_from_parent_hsm_to_target_hsm: HashMap<String, isize> =
            HashMap::new();

        for (user_defined_hw_component, desired_quantity) in
            user_defined_hw_component_counter_hashmap
        {
            let current_quantity = hsm_hw_component_summary_hashmap
                .get(user_defined_hw_component)
                .unwrap();

            let delta = (*desired_quantity as isize) - (*current_quantity as isize);

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
    }

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
        let node_hw_inventory_value =
            mesa::hsm::hw_inventory::shasta::http_client::get_hw_inventory(
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

    // Calculate/groups hw component counters filtered by user request
    pub fn get_hsm_hw_component_count_filtered_by_user_request(
        user_defined_hw_component_vec: &Vec<String>,
        target_hsm_group_hw_component_vec: &Vec<(String, HashMap<String, usize>)>,
    ) -> HashMap<String, usize> {
        let mut hsm_hw_component_count_hashmap = HashMap::new();

        for x in user_defined_hw_component_vec {
            let mut hsm_hw_component_count = 0;
            for y in target_hsm_group_hw_component_vec {
                hsm_hw_component_count += y.1.get(x).unwrap_or(&0);
            }
            hsm_hw_component_count_hashmap.insert(x.to_string(), hsm_hw_component_count);
        }

        hsm_hw_component_count_hashmap
    }

    // Calculate/groups hw component counters
    pub fn calculate_hsm_hw_component_count(
        target_hsm_group_node_hw_component_vec: &Vec<(String, HashMap<String, usize>)>,
    ) -> HashMap<String, usize> {
        let mut hsm_hw_component_count_hashmap = HashMap::new();

        for (_xname, node_hw_component_count_hashmap) in target_hsm_group_node_hw_component_vec {
            for (hw_component, &qty) in node_hw_component_count_hashmap {
                hsm_hw_component_count_hashmap
                    .entry(hw_component.to_string())
                    .and_modify(|qty_aux| *qty_aux += qty)
                    .or_insert(qty);
            }
        }

        hsm_hw_component_count_hashmap
    }

    // Given a list of tuples (xname, list of hw components qty hasmap), this function will return
    // the list of hw components wih their quantity normalized in within the hsm group
    pub fn calculate_hsm_hw_component_normalized_density_score_from_hsm_node_hw_component_count_vec(
        hsm_node_hw_component_count_vec: &Vec<(String, HashMap<String, usize>)>,
        total_number_hw_components: usize,
    ) -> HashMap<String, f32> {
        let mut hsm_hw_component_count_hashmap: HashMap<String, usize> = HashMap::new();

        for (_xname, node_hw_component_count_hashmap) in hsm_node_hw_component_count_vec {
            for (hw_component, qty) in node_hw_component_count_hashmap {
                hsm_hw_component_count_hashmap
                    .entry(hw_component.to_string())
                    .and_modify(|qty_aux| *qty_aux += qty)
                    .or_insert(*qty);
            }
        }

        calculate_hsm_hw_component_normalized_density_score_from_hsm_hw_component_count_hashmap(
            hsm_hw_component_count_hashmap,
            total_number_hw_components,
        )
    }

    // Given the list of hw components qty in a node, this function will return the list of hw
    // components with their quantity normalized within the node
    pub fn calculate_hsm_hw_component_normalized_density_score_from_hsm_hw_component_count_hashmap(
        hsm_hw_component_count_hashmap: HashMap<String, usize>,
        total_number_hw_components: usize,
    ) -> HashMap<String, f32> {
        hsm_hw_component_count_hashmap
            .iter()
            .map(|(hw_component, qty)| {
                (
                    hw_component.to_string(),
                    (*qty * 100) as f32 / total_number_hw_components as f32,
                )
            })
            .collect()
    }

    /// Returns the properties in hw_property_list found in the node_hw_inventory_value which is
    /// HSM hardware inventory API json response
    pub fn get_node_hw_properties_from_value(
        node_hw_inventory_value: &Value,
        hw_component_pattern_list: Vec<String>,
    ) -> (Vec<String>, Vec<u64>) {
        let processor_vec = mesa::hsm::hw_inventory::shasta::utils::get_list_processor_model_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

        let accelerator_vec = mesa::hsm::hw_inventory::shasta::utils::get_list_accelerator_model_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

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

        let memory_vec = mesa::hsm::hw_inventory::shasta::utils::get_list_memory_capacity_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

        (node_hw_component_pattern_vec, memory_vec)
    }

    /// Calculates greatest common factor or lowest common multiple
    pub fn calculate_lcm(numbers: &Vec<u64>) -> u64 {
        let mut lcm = u64::MAX;
        for number in numbers {
            let factors = (1..number + 1)
                .filter(|&x| number % x == 0)
                .collect::<Vec<u64>>();

            println!("Prime factors for {} --> {:?}", number, factors);

            if factors.last().is_some() && factors.last().unwrap() < &lcm {
                lcm = *factors.last().unwrap();
            }
        }

        lcm
    }

    pub fn print_table_f32_score(
        user_defined_hw_componet_vec: &[String],
        hsm_hw_pattern_vec: &[(String, HashMap<String, usize>)],
        hsm_density_score_hashmap: &HashMap<String, usize>,
        hsm_score_vec: &[(String, f32)],
    ) {
        /* println!("DEBUG - hsm_hw_pattern_vec:\n{:?}", hsm_hw_pattern_vec);
        println!(
            "DEBUG - hsm_density_score_hashmap:\n{:?}",
            hsm_density_score_hashmap
        ); */

        let hsm_hw_component_vec: Vec<String> = hsm_hw_pattern_vec
            .iter()
            .flat_map(|(_xname, node_pattern_hashmap)| node_pattern_hashmap.keys().cloned())
            .collect();

        let mut all_hw_component_vec =
            [hsm_hw_component_vec, user_defined_hw_componet_vec.to_vec()].concat();

        all_hw_component_vec.sort();
        all_hw_component_vec.dedup();

        // println!("DEBUG - all_hw_component_vec : {:?}", all_hw_component_vec);

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
                comfy_table::Cell::new(xname.clone())
                    .set_alignment(comfy_table::CellAlignment::Center),
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

        log::info!("\n{table}\n");
    }

    pub fn calculate_hsm_total_number_hw_components(
        target_hsm_hw_component_count_vec: &[(String, HashMap<String, usize>)],
    ) -> usize {
        target_hsm_hw_component_count_vec
            .iter()
            .flat_map(|(_node, hw_component_hashmap)| hw_component_hashmap.values())
            .sum()
    }
}

#[tokio::test]
pub async fn test_memory_capacity() {
    // XDG Base Directory Specification
    let project_dirs = directories::ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    // DEBUG
    let hsm_name = "zinal";

    let mut path_to_manta_configuration_file =
        std::path::PathBuf::from(project_dirs.unwrap().config_dir());

    path_to_manta_configuration_file.push("config.toml"); // ~/.config/manta/config is the file

    log::info!(
        "Reading manta configuration from {}",
        &path_to_manta_configuration_file.to_string_lossy()
    );

    let settings = crate::common::config_ops::get_configuration();

    let site_name = settings.get_string("site").unwrap();
    let site_detail_hashmap = settings.get_table("sites").unwrap();
    let site_detail_value = site_detail_hashmap
        .get(&site_name)
        .unwrap()
        .clone()
        .into_table()
        .unwrap();

    let shasta_base_url = site_detail_value
        .get("shasta_base_url")
        .unwrap()
        .to_string();

    let keycloak_base_url = site_detail_value
        .get("keycloak_base_url")
        .unwrap()
        .to_string();

    if let Some(socks_proxy) = site_detail_value.get("socks5_proxy") {
        std::env::set_var("SOCKS5", socks_proxy.to_string());
    }

    let shasta_root_cert = crate::common::config_ops::get_csm_root_cert_content(&site_name);

    let shasta_token = mesa::common::authentication::get_api_token(
        &shasta_base_url,
        &shasta_root_cert,
        &keycloak_base_url,
    )
    .await
    .unwrap();

    let hsm_group_vec = mesa::hsm::group::shasta::http_client::get_all(
        &shasta_token,
        &shasta_base_url,
        &shasta_root_cert,
    )
    .await
    .unwrap();
    /* let hsm_group_vec = hsm::http_client::get_hsm_group_vec(
        &shasta_token,
        &shasta_base_url,
        &shasta_root_cert,
        Some(&hsm_name.to_string()),
    )
    .await
    .unwrap(); */

    let mut node_hsm_groups_hw_inventory_map: HashMap<&str, (Vec<&str>, Vec<String>, Vec<u64>)> =
        HashMap::new();

    let new_vec = Vec::new();

    for hsm_group in &hsm_group_vec {
        let hsm_group_name = hsm_group["label"].as_str().unwrap();
        let hsm_member_vec: Vec<&str> = hsm_group["members"]["ids"]
            .as_array()
            .unwrap_or(&new_vec)
            .iter()
            .map(|member| member.as_str().unwrap())
            .collect();

        for member in hsm_member_vec {
            println!(
                "DEBUG - processing node {} in hsm group {}",
                member, hsm_group_name
            );
            if node_hsm_groups_hw_inventory_map.contains_key(member) {
                println!(
                    "DEBUG - node {} already processed for hsm groups {:?}",
                    member,
                    node_hsm_groups_hw_inventory_map.get(member).unwrap().0
                );

                node_hsm_groups_hw_inventory_map
                    .get_mut(member)
                    .unwrap()
                    .0
                    .push(&hsm_group_name);
            } else {
                println!(
                    "DEBUG - fetching hw components for node {} in hsm group {}",
                    member, hsm_group_name
                );
                let hw_inventory = get_node_hw_component_count(
                    shasta_token.to_string(),
                    shasta_base_url.to_string(),
                    shasta_root_cert.clone(),
                    member,
                    Vec::new(),
                )
                .await;

                node_hsm_groups_hw_inventory_map.insert(
                    member,
                    (vec![hsm_group_name], hw_inventory.1, hw_inventory.2),
                );
            }
        }
    }

    println!("\n************************************\nDEBUG - HW COMPONENT SUMMARY:\n",);

    let mut hsm_memory_capacity_lcm = u64::MAX;

    for (node, hsm_groups_hw_inventory) in node_hsm_groups_hw_inventory_map {
        let node_memory_capacity_lcm = utils::calculate_lcm(&hsm_groups_hw_inventory.2);
        if node_memory_capacity_lcm < hsm_memory_capacity_lcm {
            hsm_memory_capacity_lcm = node_memory_capacity_lcm;
        }
        println!(
            "DEBUG - node {} hwm groups {:?} hw inventory {:?} memory dimms capacity {:?} lcm {}",
            node,
            hsm_groups_hw_inventory.0,
            hsm_groups_hw_inventory.1,
            hsm_groups_hw_inventory.2,
            node_memory_capacity_lcm
        );
    }

    println!("Query LCM: {}", hsm_memory_capacity_lcm);
}

pub fn test_hsm_hw_management() {
    let hsm_zinal_hw_counters = vec![
        (
            "x1001c1s5b0n0",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s5b0n1",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s5b1n0",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s5b1n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s6b0n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 15)]),
        ),
        (
            "x1001c1s6b0n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s6b1n0",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s6b1n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s7b0n0",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s7b0n1",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s7b1n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s7b1n1",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1005c0s4b0n0",
            HashMap::from([("a100", 4), ("epyc", 1), ("Memory 16384", 32)]),
        ),
        (
            "x1005c0s4b0n1",
            HashMap::from([("epyc", 1), ("Memory 16384", 32), ("a100", 4)]),
        ),
        (
            "x1006c1s4b0n0",
            HashMap::from([("instinct", 8), ("Memory 16384", 32), ("epyc", 1)]),
        ),
        (
            "x1006c1s4b1n0",
            HashMap::from([("instinct", 8), ("epyc", 1), ("Memory 16384", 32)]),
        ),
    ];

    let hsm_nodes_free_hw_conters = vec![
        (
            "x1000c1s7b0n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1000c1s7b0n1",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1000c1s7b1n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1000c1s7b1n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s1b0n0",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s1b0n1",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s1b1n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s1b1n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s2b0n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s2b0n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s4b0n0",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
        (
            "x1001c1s4b0n1",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s4b1n0",
            HashMap::from([("epyc", 2), ("Memory 16384", 16)]),
        ),
        (
            "x1001c1s4b1n1",
            HashMap::from([("Memory 16384", 16), ("epyc", 2)]),
        ),
    ];
}
