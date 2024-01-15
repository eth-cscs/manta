use crate::cli::commands::apply_hw_cluster::utils::{
    calculate_hsm_hw_component_summary, calculate_new_hsm_group_members_from_user_pattern,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    pattern: &str,
) {
    // *********************************************************************************************************
    // PREPREQUISITES - FORMAT USER INPUT

    let pattern = format!("{}:{}", target_hsm_group_name, pattern);

    log::info!("pattern: {}", pattern);

    let parent_hsm_group_name = "nodes_free";

    // lcm -> used to normalize and quantify memory capacity
    let mem_lcm = 16384; // 1024 * 16

    // Normalize text in lowercase and separate each HSM group hw inventory pattern
    let pattern_lowercase = pattern.to_lowercase();

    let (target_hsm_group_name, pattern_hw_component) = pattern_lowercase.split_once(":").unwrap();

    let (target_hsm_node_hw_component_count_vec, parent_hsm_node_hw_component_count_vec) =
        calculate_new_hsm_group_members_from_user_pattern(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm_group_name,
            parent_hsm_group_name,
            pattern_hw_component,
            mem_lcm,
        )
        .await;

    // Calculate hw component counters (summary) across all node within the HSM group
    let target_hsm_hw_component_summary_hashmap =
        calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    // Calculate hw component counters (summary) across all node within the HSM group
    let parent_hsm_hw_component_summary_hashmap =
        calculate_hsm_hw_component_summary(&parent_hsm_node_hw_component_count_vec);

    // *********************************************************************************************************
    // PRINT SOLUTIONS

    // Print target HSM data
    log::info!(
        "HSM '{}' hw component summary: {:?}",
        target_hsm_group_name,
        target_hsm_hw_component_summary_hashmap
    );
    println!(
        "HSM '{}' members: {:?}",
        target_hsm_group_name,
        target_hsm_node_hw_component_count_vec
            .into_iter()
            .map(|(xname, _)| xname)
            .collect::<Vec<String>>()
    );

    // Print parent HSM data
    log::info!(
        "HSM '{}' hw component summary: {:?}",
        parent_hsm_group_name,
        parent_hsm_hw_component_summary_hashmap
    );
    println!(
        "HSM '{}' members: {:?}",
        parent_hsm_group_name,
        parent_hsm_node_hw_component_count_vec
            .into_iter()
            .map(|(xname, _)| xname)
            .collect::<Vec<String>>()
    );
}

pub mod utils {
    use std::{collections::HashMap, sync::Arc, time::Instant};

    use comfy_table::Color;
    use serde_json::Value;
    use tokio::sync::Semaphore;

    use crate::cli::commands::remove_hw_component_cluster::get_best_candidate_to_downscale_migrate_f32_score;

    // Returns a tuple with 2 list of nodes and its hardware components.
    pub async fn calculate_new_hsm_group_members_from_user_pattern(
        shasta_token: &str,
        shasta_base_url: &str,
        shasta_root_cert: &[u8],
        target_hsm_group_name: &str,
        parent_hsm_group_name: &str,
        pattern_hw_components: &str,
        mem_lcm: u64,
    ) -> (
        Vec<(String, HashMap<String, usize>)>,
        Vec<(String, HashMap<String, usize>)>,
    ) {
        let pattern_element_vec: Vec<&str> = pattern_hw_components.split(':').collect();

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
        let mut target_hsm_node_hw_component_count_vec = get_hsm_node_hw_component_counter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &user_defined_target_hsm_hw_component_vec,
            &target_hsm_group_member_vec,
            mem_lcm,
        )
        .await;

        // Sort nodes hw counters by node name
        target_hsm_node_hw_component_count_vec
            .sort_by_key(|target_hsm_group_hw_component| target_hsm_group_hw_component.0.clone());

        /* log::info!(
            "HSM '{}' hw component counters: {:?}",
            target_hsm_group_name,
            target_hsm_node_hw_component_count_vec
        ); */

        // Calculate hw component counters (summary) across all node within the HSM group
        let mut target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
            calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

        log::info!(
            "HSM group '{}' hw component summary: {:?}",
            target_hsm_group_name,
            target_hsm_hw_component_summary_hashmap
        );

        // *********************************************************************************************************
        // PREPREQUISITES - GET DATA - PARENT HSM

        // Get target HSM group members
        let parent_hsm_group_member_vec: Vec<String> =
            mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_name(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &parent_hsm_group_name,
            )
            .await;

        // Get HSM hw component counters for parent HSM
        let mut parent_hsm_node_hw_component_count_vec = get_hsm_node_hw_component_counter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &user_defined_target_hsm_hw_component_vec,
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
        // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY

        // Get parent HSM group members
        // Calculate nomarlized score for each hw component type in as much HSM groups as possible
        // related to the stakeholders using these nodes
        let target_hsm_and_parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap: HashMap<
            String,
            f32,
        > = calculate_scarcity_scores_across_both_target_and_parent_hsm_groups(
            &[
                target_hsm_node_hw_component_count_vec.clone(),
                parent_hsm_node_hw_component_count_vec.clone(),
            ]
            .concat(),
        )
        .await;

        // *********************************************************************************************************
        // CALCULATE DELTAS

        let deltas = calculate_all_deltas_apply_hw(
            &user_defined_target_hsm_hw_component_count_hashmap,
            &target_hsm_hw_component_summary_hashmap,
        );

        // Format deltas from HashMap<String, size> to HashMap<String, usize>
        let mut deltas_usize: HashMap<String, usize> = HashMap::new();
        for (hw_component, qty) in deltas.1 {
            deltas_usize.insert(hw_component, qty.abs() as usize);
        }

        // *********************************************************************************************************
        // MIGRATE NODES FROM PARENT HSM TO TARGET HSM

        // Migrate nodes
        let hw_component_counters_to_move_out_from_parent_hsm = downscale_from_deltas(
            &deltas_usize,
            &deltas_usize.keys().cloned().collect::<Vec<String>>(),
            &mut parent_hsm_node_hw_component_count_vec,
            &target_hsm_and_parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap,
        );

        // Add hw components counters moved from parent HSM to target HSM
        target_hsm_node_hw_component_count_vec
            .extend(hw_component_counters_to_move_out_from_parent_hsm);

        // Calculate hw component counters (summary) across all node within the HSM group
        target_hsm_hw_component_summary_hashmap =
            calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

        // Update user request hw component count with target HSM group hw components
        for (hw_component, _) in target_hsm_hw_component_summary_hashmap.clone() {
            user_defined_target_hsm_hw_component_count_hashmap
                .entry(hw_component)
                .or_insert(0);
        }

        log::info!(
            "User defined new hw component count: {:?}",
            user_defined_target_hsm_hw_component_count_hashmap
        );

        // *********************************************************************************************************
        // MIGRATE NODES FROM TARGET HSM TO PARENT HSM

        // Migrate nodes
        let hw_component_counters_to_move_out_from_target_hsm =
            crate::cli::commands::apply_hw_cluster::utils::downscale_from_final_hsm_group(
                &user_defined_target_hsm_hw_component_count_hashmap,
                &user_defined_target_hsm_hw_component_vec,
                &mut target_hsm_node_hw_component_count_vec,
                &target_hsm_and_parent_hsm_hw_component_type_scores_based_on_scarcity_hashmap,
            );

        // *********************************************************************************************************
        // MERGE HW COMPUTE COUNTERS MIGRATED FROM TARGET HSM INTO HW COMPONENTS COUNTERS OF PARENT HSM

        parent_hsm_node_hw_component_count_vec
            .extend(hw_component_counters_to_move_out_from_target_hsm);

        (
            target_hsm_node_hw_component_count_vec,
            parent_hsm_node_hw_component_count_vec,
        )
    }

    /// Removes as much nodes as it can from the target HSM group
    /// Returns a tuple with 2 vecs, the left one is the new target HSM group while the left one is
    /// the one containing the nodes removed from the target HSM
    pub fn downscale_from_final_hsm_group(
        user_defined_hsm_hw_components_count_hashmap: &HashMap<String, usize>, // hw
        // components summary the target hsm group should have according to user requests (this is
        // equivalent to target_hsm_node_hw_component_count_vec minus
        // hw_components_deltas_from_target_hsm_to_parent_hsm). Note hw componets needs to be grouped/filtered
        // based on user input
        user_defined_hw_component_vec: &Vec<String>, // list of hw components the user is asking
        // for
        target_hsm_node_hw_component_count_vec: &mut Vec<(String, HashMap<String, usize>)>, // list
        // of hw component counters in target HSM group
        hw_component_type_normalized_scores_hashmap: &HashMap<String, f32>, // hw
                                                                            // component type score for as much hsm groups related to the stakeholders usinjkjjjjkj these
                                                                            // nodes
    ) -> Vec<(String, HashMap<String, usize>)> {
        ////////////////////////////////
        // Initialize

        // Calculate hw component counters for the whole HSM group
        let mut target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
            calculate_hsm_hw_component_summary(target_hsm_node_hw_component_count_vec);

        // Calculate initial scores
        let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
            calculate_hsm_node_scores_from_final_hsm(
                &target_hsm_node_hw_component_count_vec,
                &target_hsm_hw_component_summary_hashmap,
                &user_defined_hsm_hw_components_count_hashmap,
                &hw_component_type_normalized_scores_hashmap,
            );

        let mut nodes_migrated_from_target_hsm: Vec<(String, HashMap<String, usize>)> = Vec::new();

        // Get best candidate
        let (mut best_candidate, mut best_candidate_counters) =
            get_best_candidate_to_downscale_migrate_f32_score(
                &mut target_hsm_node_score_tuple_vec,
                target_hsm_node_hw_component_count_vec,
            );

        // Check if we need to keep iterating
        let mut work_to_do = keep_iterating_final_hsm(
            &user_defined_hsm_hw_components_count_hashmap,
            &best_candidate_counters,
            &target_hsm_hw_component_summary_hashmap,
        );

        ////////////////////////////////
        // Iterate

        let mut iter = 0;

        while work_to_do {
            log::info!("----- ITERATION {} -----", iter);

            log::info!(
                "HSM group hw component counters: {:?}",
                target_hsm_hw_component_summary_hashmap
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
                &target_hsm_node_score_tuple_vec,
            );

            ////////////////////////////////
            // Apply changes - Migrate from target to parent HSM

            // Add best candidate to list of nodes migrated
            nodes_migrated_from_target_hsm
                .push((best_candidate.0.clone(), best_candidate_counters.clone()));

            // Remove best candidate from target HSM grour
            target_hsm_node_hw_component_count_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

            if target_hsm_node_hw_component_count_vec.is_empty() {
                break;
            }

            // Calculate hw component couters for the whole HSM group
            target_hsm_hw_component_summary_hashmap =
                calculate_hsm_hw_component_summary(target_hsm_node_hw_component_count_vec);

            // Remove best candidate from scores
            target_hsm_node_score_tuple_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

            // Get best candidate
            (best_candidate, best_candidate_counters) =
                get_best_candidate_to_downscale_migrate_f32_score(
                    &mut target_hsm_node_score_tuple_vec,
                    target_hsm_node_hw_component_count_vec,
                );

            // Check if we need to keep iterating
            work_to_do = keep_iterating_final_hsm(
                &user_defined_hsm_hw_components_count_hashmap,
                &best_candidate_counters,
                // &downscale_deltas,
                &target_hsm_hw_component_summary_hashmap,
            );

            iter += 1;
        }

        log::info!("----- FINAL RESULT -----");

        log::info!("No candidates found");

        // Print target hsm group hw configuration in table
        print_table_f32_score(
            user_defined_hw_component_vec,
            target_hsm_node_hw_component_count_vec,
            &target_hsm_node_score_tuple_vec,
        );

        nodes_migrated_from_target_hsm
    }

    /// Removes as much nodes as it can from the target HSM group
    /// Returns a tuple with 2 vecs, the left one is the new target HSM group while the left one is
    /// the one containing the nodes removed from the target HSM
    pub fn downscale_from_deltas(
        deltas_hashmap: &HashMap<String, usize>, // hw
        // components summary the target hsm group should have according to user requests (this is
        // equivalent to target_hsm_node_hw_component_count_vec minus
        // hw_components_deltas_from_target_hsm_to_parent_hsm). Note hw componets needs to be grouped/filtered
        // based on user input
        deltas_hw_component_vec: &Vec<String>, // list of hw components the user is asking
        // for
        hsm_node_hw_component_count_vec: &mut Vec<(String, HashMap<String, usize>)>, // list
        // of hw component counters in target HSM group
        hw_component_type_normalized_scores_hashmap: &HashMap<String, f32>, // hw
                                                                            // component type score for as much hsm groups related to the stakeholders usinjkjjjjkj these
                                                                            // nodes
    ) -> Vec<(String, HashMap<String, usize>)> {
        ////////////////////////////////
        // Initialize

        let mut deltas_hashmap = deltas_hashmap.clone();

        // Calculate hw component counters for the whole HSM group
        let mut target_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
            calculate_hsm_hw_component_summary(hsm_node_hw_component_count_vec);

        // Calculate initial scores
        let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
            calculate_hsm_node_scores_from_deltas(
                &hsm_node_hw_component_count_vec,
                &deltas_hashmap,
                &hw_component_type_normalized_scores_hashmap,
            );

        let mut nodes_migrated_from_target_hsm: Vec<(String, HashMap<String, usize>)> = Vec::new();

        // Get best candidate
        let (mut best_candidate, mut best_candidate_counters) =
            get_best_candidate_to_downscale_migrate_f32_score(
                &mut target_hsm_node_score_tuple_vec,
                hsm_node_hw_component_count_vec,
            );

        // Check if we need to keep iterating
        let mut work_to_do = keep_iterating_deltas(
            &deltas_hashmap,
            &best_candidate_counters,
            // &downscale_deltas,
            &target_hsm_hw_component_summary_hashmap,
        );

        ////////////////////////////////
        // Iterate

        let mut iter = 0;

        while work_to_do {
            log::info!("----- ITERATION {} -----", iter);

            log::info!(
                "HSM group hw component counters: {:?}",
                target_hsm_hw_component_summary_hashmap
            );
            log::info!("Deltas: {:?}", deltas_hashmap);
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
                deltas_hw_component_vec,
                hsm_node_hw_component_count_vec,
                &target_hsm_node_score_tuple_vec,
            );

            ////////////////////////////////
            // Apply changes - Migrate from target to parent HSM

            // Add best candidate to list of nodes migrated
            nodes_migrated_from_target_hsm
                .push((best_candidate.0.clone(), best_candidate_counters.clone()));

            // Remove best candidate from target HSM grour
            hsm_node_hw_component_count_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

            // Calculate hw component couters for the whole HSM group
            target_hsm_hw_component_summary_hashmap =
                calculate_hsm_hw_component_summary(hsm_node_hw_component_count_vec);

            // Remove best candidate from scores
            target_hsm_node_score_tuple_vec.retain(|(node, _)| !node.eq(&best_candidate.0));

            // Update deltas
            for (hw_component, qty) in best_candidate_counters {
                deltas_hashmap
                    .entry(hw_component)
                    .and_modify(|delta_qty| *delta_qty -= qty);
            }

            // Get best candidate
            (best_candidate, best_candidate_counters) =
                get_best_candidate_to_downscale_migrate_f32_score(
                    &mut target_hsm_node_score_tuple_vec,
                    hsm_node_hw_component_count_vec,
                );

            // Check if we need to keep iterating
            work_to_do = keep_iterating_deltas(
                &deltas_hashmap,
                &best_candidate_counters,
                // &downscale_deltas,
                &target_hsm_hw_component_summary_hashmap,
            );

            iter += 1;
        }

        log::info!("----- FINAL RESULT -----");

        log::info!("No candidates found");

        // Print target hsm group hw configuration in table
        print_table_f32_score(
            deltas_hw_component_vec,
            hsm_node_hw_component_count_vec,
            &target_hsm_node_score_tuple_vec,
        );

        nodes_migrated_from_target_hsm
    }

    pub async fn calculate_scarcity_scores_across_both_target_and_parent_hsm_groups(
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
    }

    /// Calculates a normalized score for each hw component in HSM group based on component
    /// scarcity.
    pub fn calculate_hsm_node_scores_from_final_hsm(
        hsm_node_hw_component_count_vec: &Vec<(String, HashMap<String, usize>)>,
        final_hsm_hw_component_summary_hashmap: &HashMap<String, usize>,
        final_hsm_summary_hashmap: &HashMap<String, usize>,
        hw_component_type_scores_hashmap: &HashMap<String, f32>,
    ) -> Vec<(String, f32)> {
        let mut node_score_vec: Vec<(String, f32)> = Vec::new();

        for (xname, hw_component_count) in hsm_node_hw_component_count_vec {
            let mut node_score: f32 = 0.0;
            for (hw_component, qty) in hw_component_count {
                if final_hsm_summary_hashmap.get(hw_component).unwrap()
                    < final_hsm_hw_component_summary_hashmap
                        .get(hw_component)
                        .unwrap()
                {
                    // positive - current hw component counter in target HSM group are higher than
                    // final (user requested) hw component counter therefore we remove this node
                    node_score +=
                        hw_component_type_scores_hashmap.get(hw_component).unwrap() * *qty as f32;
                } else {
                    // negative - current hw component counter in target HSM group is lower or
                    // equal than final (user requested) hw component counter therefor we should
                    // penalize this node
                    node_score -=
                        hw_component_type_scores_hashmap.get(hw_component).unwrap() * *qty as f32;
                }
            }
            node_score_vec.push((xname.to_string(), node_score));
        }

        node_score_vec
    }

    /// Calculates a normalized score for each hw component in HSM group based on component
    /// scarcity.
    pub fn calculate_hsm_node_scores_from_deltas(
        hsm_node_hw_component_count_vec: &Vec<(String, HashMap<String, usize>)>,
        deltas: &HashMap<String, usize>,
        target_hsm_hw_component_normalized_scores: &HashMap<String, f32>,
    ) -> Vec<(String, f32)> {
        let mut node_score_vec: Vec<(String, f32)> = Vec::new();

        for (xname, hw_component_count) in hsm_node_hw_component_count_vec {
            let mut node_score: f32 = 0.0;
            for (hw_component, qty) in hw_component_count {
                if deltas
                    .get(hw_component)
                    .is_some_and(|hw_component_qty| hw_component_qty > &0)
                {
                    // positive - we remove this node
                    node_score += target_hsm_hw_component_normalized_scores
                        .get(hw_component)
                        .unwrap()
                        * *qty as f32;
                } else {
                    // negative - we should penalize this node
                    node_score -= target_hsm_hw_component_normalized_scores
                        .get(hw_component)
                        .unwrap()
                        * *qty as f32;
                }
            }
            node_score_vec.push((xname.to_string(), node_score));
        }

        node_score_vec
    }

    pub fn keep_iterating_final_hsm(
        hsm_final_hw_component_summary_hashmap: &HashMap<String, usize>, // hw components in
        // the target hsm group asked by the user (this is the minimum boundary, we can't provide
        // less than this)
        best_candidate_counters: &HashMap<String, usize>,
        // hw_components_deltas_from_target_hsm_to_parent_hsm: &HashMap<String, isize>, // minimum boundaries (we
        // can't provide less that this)
        hsm_current_hw_component_summary_hashmap: &HashMap<String, usize>, // list of nodes
                                                                           // and its scores
    ) -> bool {
        for (hw_component, qty) in best_candidate_counters {
            let updated_hw_component_qty = hsm_current_hw_component_summary_hashmap
                .get(hw_component)
                .unwrap()
                - qty;
            if updated_hw_component_qty
                < *hsm_final_hw_component_summary_hashmap
                    .get(hw_component)
                    .unwrap()
            {
                log::info!("Best candidate '{:?}' can't be removed from target HSM group '{:?}', othwerwise we will violate user request '{:?}'", best_candidate_counters, hsm_current_hw_component_summary_hashmap, hsm_final_hw_component_summary_hashmap);
                return false;
            }
        }

        true
    }

    pub fn keep_iterating_deltas(
        deltas_hashmap: &HashMap<String, usize>,
        best_candidate_counters: &HashMap<String, usize>,
        hsm_current_hw_component_summary_hashmap: &HashMap<String, usize>,
    ) -> bool {
        for (hw_component, qty) in best_candidate_counters {
            if let Some(delta_qty) = deltas_hashmap.get(hw_component) {
                if delta_qty < qty {
                    log::info!("Best candidate '{:?}' can't be removed from target HSM group '{:?}', othwerwise we will surprass the deltas '{:?}'", best_candidate_counters, hsm_current_hw_component_summary_hashmap, deltas_hashmap);
                    return false;
                }
            }
        }

        true
    }

    pub fn calculate_all_deltas_apply_hw(
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

    // Calculate/groups hw component counters
    pub fn calculate_hsm_hw_component_summary(
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

    pub fn print_table_f32_score(
        user_defined_hw_componet_vec: &[String],
        hsm_hw_pattern_vec: &[(String, HashMap<String, usize>)],
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

    pub async fn get_hsm_node_hw_component_counter(
        shasta_token: &str,
        shasta_base_url: &str,
        shasta_root_cert: &[u8],
        user_defined_hw_component_vec: &Vec<String>,
        hsm_group_member_vec: &Vec<String>,
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
        for hsm_member in hsm_group_member_vec.clone() {
            let shasta_token_string = shasta_token.to_string(); // TODO: make it static
            let shasta_base_url_string = shasta_base_url.to_string(); // TODO: make it static
            let shasta_root_cert_vec = shasta_root_cert.to_vec();
            let user_defined_hw_component_vec = user_defined_hw_component_vec.clone();

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
}

/* #[tokio::test]
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
                "Processing node {} in hsm group {}",
                member, hsm_group_name
            );
            if node_hsm_groups_hw_inventory_map.contains_key(member) {
                println!(
                    "Node {} already processed for hsm groups {:?}",
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
                    "Fetching hw components for node {} in hsm group {}",
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
            "Node {} HSM groups {:?} hw inventory {:?} memory dimms capacity {:?} lcm {}",
            node,
            hsm_groups_hw_inventory.0,
            hsm_groups_hw_inventory.1,
            hsm_groups_hw_inventory.2,
            node_memory_capacity_lcm
        );
    }

    println!("Query LCM: {}", hsm_memory_capacity_lcm);
} */

/* pub fn test_hsm_hw_management() {
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
} */
