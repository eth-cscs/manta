use mesa::hsm;
use std::collections::HashMap;

use mesa::hsm::group::utils::update_hsm_group_members;

use crate::cli::commands::apply_hw_cluster::utils::{
    calculate_hsm_hw_component_summary, get_hsm_group_members_from_user_pattern,
    get_hsm_node_hw_component_counter,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    pattern: &str,
    nodryrun: bool,
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

    let (target_hsm_group_name, pattern_hw_component) = pattern_lowercase.split_once(':').unwrap();

    let pattern_element_vec: Vec<&str> = pattern_hw_component.split(':').collect();

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

    match mesa::hsm::group::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&target_hsm_group_name.to_string()),
    )
    .await
    {
        Ok(_) => log::debug!("Target HSM group {} exists, good.", target_hsm_group_name),
        Err(_) => {
            if create_target_hsm_group {
                log::info!("Target HSM group {} does not exist, but the option to create the group has been selected, creating it now.", target_hsm_group_name.to_string());
                if nodryrun {
                    mesa::hsm::group::http_client::create_new_hsm_group(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_name,
                        &[],
                        "false",
                        "",
                        &[],
                    )
                    .await
                    .expect("Unable to create new target HSM group");
                } else {
                    log::error!("Dryrun selected, cannot create the new group and continue.");
                    std::process::exit(1);
                }
            } else {
                log::error!("Target HSM group {} does not exist, but the option to create the group was NOT specificied, cannot continue.", target_hsm_group_name.to_string());
                std::process::exit(1);
            }
        }
    };

    // Get target HSM group members
    let target_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            target_hsm_group_name,
        )
        .await;

    // Get HSM hw component counters for target HSM
    let mut target_hsm_node_hw_component_count_vec: Vec<(String, HashMap<String, usize>)> =
        get_hsm_node_hw_component_counter(
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
    let parent_hsm_group_member_vec: Vec<String> =
        mesa::hsm::group::utils::get_member_vec_from_hsm_group_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            parent_hsm_group_name,
        )
        .await;

    // Get HSM hw component counters for parent HSM
    let mut parent_hsm_node_hw_component_count_vec: Vec<(String, HashMap<String, usize>)> =
        get_hsm_node_hw_component_counter(
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

    // *********************************************************************************************************
    // CALCULATE 'COMBINED HSM'
    // COMBINED HSM is the result of combining all nodes from TARGET HSM and PARENT HSM into a
    // single pool

    let parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
        calculate_hsm_hw_component_summary(&parent_hsm_node_hw_component_count_vec);

    log::info!(
        "HSM group '{}' hw component summary: {:?}",
        parent_hsm_group_name,
        parent_hsm_hw_component_summary_hashmap
    );

    // *********************************************************************************************************
    // CALCULATE NODES TO MOVE FROM COMBINED HSM TO TARGET HSM

    let (target_hsm_node_hw_component_count_vec, parent_hsm_node_hw_component_count_vec) =
        get_hsm_group_members_from_user_pattern(
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
    if !nodryrun {
        log::info!("Dry run enabled, not modifying the HSM groups on the system.");
    } else {
        // The target HSM group will never be empty, the way the pattern works it'll always
        // contain at least one node, so there is no need to add code to delete it if it's empty.
        let _ = update_hsm_group_members(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
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
    if !nodryrun {
        log::info!("Dry run enabled, not modifying the HSM groups on the system.");
    } else {
        // The parent group might be out of resources after applying this, so it's safe to check
        // if there are still nodes there and, delete it after moving out the resources.
        let parent_group_will_be_empty =
            &target_hsm_group_member_vec.len() == &parent_hsm_group_member_vec.len();
        let _ = update_hsm_group_members(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            parent_hsm_group_name,
            &parent_hsm_group_member_vec,
            &parent_hsm_node_vec,
        )
        .await;
        if parent_group_will_be_empty {
            if delete_empty_parent_hsm_group {
                log::info!("Parent HSM group {} is now empty and the option to delete empty groups has been selected, removing it.",parent_hsm_group_name);
                match hsm::group::http_client::delete_hsm_group(shasta_token,
                                                                      shasta_base_url,
                                                                      shasta_root_cert,
                                                                      &parent_hsm_group_name.to_string())
                    .await {
                    Ok(_) => log::info!("HSM group removed successfully."),
                    Err(e2) => log::debug!("Error removing the HSM group. This always fails, ignore please. Reported: {}", e2)
                };
            } else {
                log::debug!("Parent HSM group {} is now empty and the option to delete empty groups has NOT been selected, will not remove it.",parent_hsm_group_name)
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

pub mod utils {
    use std::{collections::HashMap, sync::Arc, time::Instant};

    use comfy_table::Color;
    use serde_json::Value;
    use tokio::sync::Semaphore;

    use crate::cli::commands::remove_hw_component_cluster::get_best_candidate_to_downscale_migrate_f32_score;

    // Returns a tuple (target_hsm, parent_hsm) with 2 list of nodes and its hardware components.
    // The left tuple element are the nodes moved from the
    pub async fn get_hsm_group_members_from_user_pattern(
        target_hsm_node_hw_component_count_vec: Vec<(String, HashMap<String, usize>)>,
        parent_hsm_node_hw_component_count_vec: Vec<(String, HashMap<String, usize>)>,
        user_defined_target_hsm_hw_component_count_hashmap: HashMap<String, usize>,
    ) -> (
        Vec<(String, HashMap<String, usize>)>,
        Vec<(String, HashMap<String, usize>)>,
    ) {
        // *********************************************************************************************************
        // CALCULATE 'COMBINED HSM' WITH TARGET HSM AND PARENT HSM ELEMENTS COMBINED
        // NOTE: PARENT HSM many contain elements in TARGET HSM, we need to only add those xnames
        // which are not part of PARENT HSM already

        let mut combined_target_parent_hsm_node_hw_component_count_vec =
            parent_hsm_node_hw_component_count_vec.clone();

        for elem in target_hsm_node_hw_component_count_vec {
            if !parent_hsm_node_hw_component_count_vec
                .iter()
                .any(|(xname, _)| xname.eq(&elem.0))
            {
                combined_target_parent_hsm_node_hw_component_count_vec.push(elem);
            }
        }

        let combined_target_parent_hsm_hw_component_summary_hashmap =
            calculate_hsm_hw_component_summary(
                &combined_target_parent_hsm_node_hw_component_count_vec,
            );

        // *********************************************************************************************************
        // CALCULATE HW COMPONENT TYPE SCORE BASED ON SCARCITY

        // Get parent HSM group members
        // Calculate nomarlized score for each hw component type in as much HSM groups as possible
        // related to the stakeholders using these nodes
        let hw_component_scarcity_scores_hashmap: HashMap<String, f32> =
            calculate_hw_component_scarcity_scores(
                &combined_target_parent_hsm_node_hw_component_count_vec,
            )
            .await;

        // *********************************************************************************************************
        // CALCULATE FINAL HSM SUMMARY COUNTERS AFTER REMOVING THE NODES THAT NEED TO GO TO TARGET
        // HSM (SUBSTRACT USER INPUT SUMMARY FROM INITIAL COMBINED HSM SUMMARY)
        let mut final_combined_target_parent_hsm_hw_component_summary =
            user_defined_target_hsm_hw_component_count_hashmap.clone();

        for (hw_component, qty) in combined_target_parent_hsm_hw_component_summary_hashmap {
            final_combined_target_parent_hsm_hw_component_summary
                .entry(hw_component)
                .and_modify(|current_qty| *current_qty = qty - *current_qty);
        }

        // Downscale combined HSM group
        let hw_component_counters_to_move_out_from_combined_hsm =
            crate::cli::commands::apply_hw_cluster::utils::downscale_from_final_hsm_group(
                &final_combined_target_parent_hsm_hw_component_summary.clone(),
                &final_combined_target_parent_hsm_hw_component_summary
                    .into_keys()
                    .collect::<Vec<String>>(),
                &mut combined_target_parent_hsm_node_hw_component_count_vec,
                &hw_component_scarcity_scores_hashmap,
            );

        let new_target_hsm_node_hw_component_count_vec =
            hw_component_counters_to_move_out_from_combined_hsm;
        (
            new_target_hsm_node_hw_component_count_vec,
            combined_target_parent_hsm_node_hw_component_count_vec,
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
        user_defined_hw_component_vec: &[String], // list of hw components the user is asking
        // for
        combination_target_parent_hsm_node_hw_component_count_vec: &mut Vec<(
            String,
            HashMap<String, usize>,
        )>, // list
        // of hw component counters in target HSM group
        hw_component_scarcity_scores_hashmap: &HashMap<String, f32>, // hw
                                                                     // component type score for as much hsm groups related to the stakeholders using these
                                                                     // nodes
    ) -> Vec<(String, HashMap<String, usize>)> {
        ////////////////////////////////
        // Initialize

        // Calculate hw component counters for the whole HSM group
        let mut combination_target_parent_hsm_hw_component_summary_hashmap: HashMap<String, usize> =
            calculate_hsm_hw_component_summary(
                combination_target_parent_hsm_node_hw_component_count_vec,
            );

        // Calculate initial scores
        let mut combination_target_parent_hsm_node_score_tuple_vec: Vec<(String, f32)> =
            calculate_hsm_node_scores_from_final_hsm(
                combination_target_parent_hsm_node_hw_component_count_vec,
                &combination_target_parent_hsm_hw_component_summary_hashmap,
                user_defined_hsm_hw_components_count_hashmap,
                hw_component_scarcity_scores_hashmap,
            );

        let mut nodes_migrated_from_combination_target_parent_hsm: Vec<(
            String,
            HashMap<String, usize>,
        )> = Vec::new();

        // Get best candidate
        let (mut best_candidate, mut best_candidate_counters) =
            get_best_candidate_to_downscale_migrate_f32_score(
                &mut combination_target_parent_hsm_node_score_tuple_vec,
                combination_target_parent_hsm_node_hw_component_count_vec,
            );

        // Check if we need to keep iterating
        let mut work_to_do = keep_iterating_final_hsm(
            user_defined_hsm_hw_components_count_hashmap,
            // &best_candidate_counters,
            &combination_target_parent_hsm_hw_component_summary_hashmap,
        );

        ////////////////////////////////
        // Iterate

        let mut iter = 0;

        while work_to_do {
            log::info!("----- ITERATION {} -----", iter);

            log::info!(
                "HSM group hw component counters: {:?}",
                combination_target_parent_hsm_hw_component_summary_hashmap
            );
            log::info!(
                "Final hw component counters the user wants: {:?}",
                user_defined_hsm_hw_components_count_hashmap
            );
            log::info!(
                "Best candidate is '{}' with score {} and hw component counters {:?}",
                best_candidate.0,
                combination_target_parent_hsm_node_score_tuple_vec
                    .iter()
                    .find(|(node, _score)| node.eq(&best_candidate.0))
                    .unwrap()
                    .1,
                best_candidate_counters
            );

            // Print target hsm group hw configuration in table
            print_table_f32_score(
                user_defined_hw_component_vec,
                combination_target_parent_hsm_node_hw_component_count_vec,
                &combination_target_parent_hsm_node_score_tuple_vec,
            );

            ////////////////////////////////
            // Apply changes - Migrate from target to parent HSM

            // Add best candidate to list of nodes migrated
            nodes_migrated_from_combination_target_parent_hsm
                .push((best_candidate.0.clone(), best_candidate_counters.clone()));

            // Remove best candidate from target HSM grour
            combination_target_parent_hsm_node_hw_component_count_vec
                .retain(|(node, _)| !node.eq(&best_candidate.0));

            if combination_target_parent_hsm_node_hw_component_count_vec.is_empty() {
                break;
            }

            // Calculate hw component couters for the whole HSM group
            combination_target_parent_hsm_hw_component_summary_hashmap =
                calculate_hsm_hw_component_summary(
                    combination_target_parent_hsm_node_hw_component_count_vec,
                );

            // Remove best candidate from scores
            combination_target_parent_hsm_node_score_tuple_vec
                .retain(|(node, _)| !node.eq(&best_candidate.0));

            // Recalculate scores
            let mut target_hsm_node_score_tuple_vec: Vec<(String, f32)> =
                calculate_hsm_node_scores_from_final_hsm(
                    combination_target_parent_hsm_node_hw_component_count_vec,
                    &combination_target_parent_hsm_hw_component_summary_hashmap,
                    user_defined_hsm_hw_components_count_hashmap,
                    hw_component_scarcity_scores_hashmap,
                );

            // Get best candidate
            (best_candidate, best_candidate_counters) =
                get_best_candidate_to_downscale_migrate_f32_score(
                    &mut target_hsm_node_score_tuple_vec,
                    combination_target_parent_hsm_node_hw_component_count_vec,
                );

            // Check if we need to keep iterating
            work_to_do = keep_iterating_final_hsm(
                user_defined_hsm_hw_components_count_hashmap,
                // &best_candidate_counters,
                // &downscale_deltas,
                &combination_target_parent_hsm_hw_component_summary_hashmap,
            );

            iter += 1;
        }

        log::info!("----- FINAL RESULT -----");

        log::info!("No candidates found");

        // Print target hsm group hw configuration in table
        print_table_f32_score(
            user_defined_hw_component_vec,
            combination_target_parent_hsm_node_hw_component_count_vec,
            &combination_target_parent_hsm_node_score_tuple_vec,
        );

        nodes_migrated_from_combination_target_parent_hsm
    }

    /* /// Removes as much nodes as it can from the target HSM group
    /// Returns a tuple with 2 vecs, the left one is the new target HSM group while the left one is
    /// the one containing the nodes removed from the target HSM
    pub fn downscale_from_deltas(
        deltas_hashmap: &HashMap<String, usize>, // hw
        // components summary the target hsm group should have according to user requests (this is
        // equivalent to target_hsm_node_hw_component_count_vec minus
        // hw_components_deltas_from_target_hsm_to_parent_hsm). Note hw componets needs to be grouped/filtered
        // based on user input
        deltas_hw_component_vec: &[String], // list of hw components the user is asking
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
                hsm_node_hw_component_count_vec,
                &deltas_hashmap,
                hw_component_type_normalized_scores_hashmap,
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

            // TODO: Recalculate scores ????

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
    } */

    /// Calculate a score for each hw component the user has access to. Scaricity scales with value hence
    /// high value means high scarcity
    /* pub async fn calculate_hw_component_scarcity_scores(
        hsm_node_hw_component_count: &Vec<(String, HashMap<String, usize>)>,
    ) -> HashMap<String, f32> {
        let total_num_nodes = hsm_node_hw_component_count.len();

        let mut hw_component_vec: Vec<&String> = hsm_node_hw_component_count
            .iter()
            .flat_map(|(_, hw_component_counter_hashmap)| hw_component_counter_hashmap.keys())
            .collect();

        hw_component_vec.sort();
        hw_component_vec.dedup();

        let mut hw_component_scarcity_score_hashmap: HashMap<String, f32> = HashMap::new();
        for hw_component in hw_component_vec {
            let mut node_count = 0;

            for (_, hw_component_counter_hashmap) in hsm_node_hw_component_count {
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

    pub async fn calculate_hw_component_scarcity_scores(
        hsm_node_hw_component_count: &Vec<(String, HashMap<String, usize>)>,
    ) -> HashMap<String, f32> {
        let total_num_hw_components: usize = hsm_node_hw_component_count
            .iter()
            .flat_map(|(_, hw_component_qty_hashmap)| {
                hw_component_qty_hashmap
                    .iter()
                    .map(|(_, hw_component_qty)| hw_component_qty)
            })
            .sum();

        let mut hw_component_vec: Vec<&String> = hsm_node_hw_component_count
            .iter()
            .flat_map(|(_, hw_component_counter_hashmap)| hw_component_counter_hashmap.keys())
            .collect();

        hw_component_vec.sort();
        hw_component_vec.dedup();

        let mut hw_component_scarcity_score_hashmap: HashMap<String, f32> = HashMap::new();
        for hw_component in hw_component_vec {
            let mut hsm_hw_component_count = 0;

            for (_, hw_component_counter_hashmap) in hsm_node_hw_component_count {
                if let Some(hw_component_qty) = hw_component_counter_hashmap.get(hw_component) {
                    hsm_hw_component_count += hw_component_qty;
                }
            }

            hw_component_scarcity_score_hashmap.insert(
                hw_component.to_string(),
                (total_num_hw_components as f32) / (hsm_hw_component_count as f32),
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
        parent_hsm_node_hw_component_count_vec: &Vec<(String, HashMap<String, usize>)>,
        parent_hsm_hw_component_summary_hashmap: &HashMap<String, usize>,
        final_hsm_summary_hashmap: &HashMap<String, usize>,
        hw_component_scarcity_scores_hashmap: &HashMap<String, f32>,
    ) -> Vec<(String, f32)> {
        let mut node_score_vec: Vec<(String, f32)> = Vec::new();

        for (xname, hw_component_count) in parent_hsm_node_hw_component_count_vec {
            let mut node_score: f32 = 0.0;
            for (hw_component, qty) in hw_component_count {
                if final_hsm_summary_hashmap.get(hw_component).is_none() {
                    // final/user request does NOT contain hw component
                    // negative - current hw component counter in HSM group is not requested by the user therefor we should
                    // penalize this node
                    node_score -= hw_component_scarcity_scores_hashmap
                        .get(hw_component)
                        .unwrap()
                        * *qty as f32;
                } else {
                    // final/user request does contain hw component
                    if final_hsm_summary_hashmap.get(hw_component).unwrap()
                        < parent_hsm_hw_component_summary_hashmap
                            .get(hw_component)
                            .unwrap()
                    {
                        // positive - current hw component counter in parent/combined HSM group are higher than
                        // final (user requested) hw component counter therefore we remove this node
                        node_score += hw_component_scarcity_scores_hashmap
                            .get(hw_component)
                            .unwrap()
                            * *qty as f32;
                    } else {
                        // negative - current hw component counter in parent/combined HSM group is lower or
                        // equal than final (user requested) hw component counter therefor we should
                        // penalize this node
                        node_score -= hw_component_scarcity_scores_hashmap
                            .get(hw_component)
                            .unwrap()
                            * *qty as f32;
                    }
                }
            }
            node_score_vec.push((xname.to_string(), node_score));
        }

        node_score_vec
    }

    /* /// Calculates a normalized score for each hw component in HSM group based on component
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
    } */

    pub fn keep_iterating_final_hsm(
        hsm_final_hw_component_summary_hashmap: &HashMap<String, usize>, // hw components in
        // the target hsm group asked by the user (this is the minimum boundary, we can't provide
        // less than this)
        // best_candidate_counters: &HashMap<String, usize>,
        // hw_components_deltas_from_target_hsm_to_parent_hsm: &HashMap<String, isize>, // minimum boundaries (we
        // can't provide less that this)
        hsm_current_hw_component_summary_hashmap: &HashMap<String, usize>, // list of nodes
                                                                           // and its scores
    ) -> bool {
        /* for (hw_component, _) in best_candidate_counters {
            let updated_hw_component_qty = hsm_current_hw_component_summary_hashmap
                .get(hw_component)
                .unwrap();
            if hsm_final_hw_component_summary_hashmap
                .get(hw_component)
                .is_some()
                && updated_hw_component_qty
                    < hsm_final_hw_component_summary_hashmap
                        .get(hw_component)
                        .unwrap()
            {
                log::info!("Best candidate '{:?}' can't be removed from HSM group '{:?}', othwerwise current HSM group will have less resources than requested '{:?}'", best_candidate_counters, hsm_current_hw_component_summary_hashmap, hsm_final_hw_component_summary_hashmap);
                return false;
            }
        }

        true */

        for (hw_component, final_qty) in hsm_final_hw_component_summary_hashmap {
            if hsm_current_hw_component_summary_hashmap
                .get(hw_component)
                .is_some_and(|current_qty| current_qty > final_qty)
            {
                return true;
            }
        }

        false
    }

    /* pub fn keep_iterating_deltas(
        deltas_hashmap: &HashMap<String, usize>,
        best_candidate_counters: &HashMap<String, usize>,
        hsm_current_hw_component_summary_hashmap: &HashMap<String, usize>,
    ) -> bool {
        if deltas_hashmap.is_empty() {
            log::info!("Deltas is empty. Stop iterating");
            return false;
        }

        for (hw_component, qty) in best_candidate_counters {
            if let Some(delta_qty) = deltas_hashmap.get(hw_component) {
                if delta_qty < qty {
                    log::info!("Best candidate '{:?}' can't be removed from target HSM group '{:?}', othwerwise we will surprass the deltas '{:?}'", best_candidate_counters, hsm_current_hw_component_summary_hashmap, deltas_hashmap);
                    return false;
                }
            }
        }

        true
    } */

    /* pub fn calculate_all_deltas_apply_hw(
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
        let node_hw_inventory_value =
            mesa::hsm::hw_inventory::hw_component::http_client::get_hw_inventory(
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
        let processor_vec = mesa::hsm::hw_inventory::hw_component::utils::get_list_processor_model_from_hw_inventory_value(
            node_hw_inventory_value,
        )
        .unwrap_or_default();

        let accelerator_vec = mesa::hsm::hw_inventory::hw_component::utils::get_list_accelerator_model_from_hw_inventory_value(
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

        let memory_vec = mesa::hsm::hw_inventory::hw_component::utils::get_list_memory_capacity_from_hw_inventory_value(
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
                        comfy_table::Cell::new(format!("⚠️  ({})", counter)) // NOTE: emojis
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
}

#[tokio::test]
pub async fn test_hsm_hw_management_1() {
    let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

    let hsm_zinal_hw_counters = vec![
        (
            "x1001c1s5b0n0".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s5b0n1".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s5b1n0".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s5b1n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s6b0n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 15)]),
        ),
        (
            "x1001c1s6b0n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s6b1n0".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s6b1n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s7b0n0".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s7b0n1".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s7b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s7b1n1".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1005c0s4b0n0".to_string(),
            HashMap::from([
                ("a100".to_string(), 4),
                ("epyc".to_string(), 1),
                ("Memory 16384".to_string(), 32),
            ]),
        ),
        (
            "x1005c0s4b0n1".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("Memory 16384".to_string(), 32),
                ("a100".to_string(), 4),
            ]),
        ),
        (
            "x1006c1s4b0n0".to_string(),
            HashMap::from([
                ("instinct".to_string(), 8),
                ("Memory 16384".to_string(), 32),
                ("epyc".to_string(), 1),
            ]),
        ),
        (
            "x1006c1s4b1n0".to_string(),
            HashMap::from([
                ("instinct".to_string(), 8),
                ("epyc".to_string(), 1),
                ("Memory 16384".to_string(), 32),
            ]),
        ),
    ];

    let hsm_nodes_free_hw_conters = vec![
        (
            "x1000c1s7b0n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1000c1s7b0n1".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1000c1s7b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1000c1s7b1n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s1b0n0".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s1b0n1".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s1b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s1b1n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s2b0n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s2b0n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s4b0n0".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s4b0n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s4b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("Memory 16384".to_string(), 16)]),
        ),
        (
            "x1001c1s4b1n1".to_string(),
            HashMap::from([("Memory 16384".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
    ];

    let (target_hsm_node_hw_component_count_vec, parent_hsm_node_hw_component_count_vec) =
        get_hsm_group_members_from_user_pattern(
            hsm_zinal_hw_counters,
            hsm_nodes_free_hw_conters,
            user_request_hw_summary.clone(),
        )
        .await;

    println!(
        "DEBUG - target HSM group:\n{:#?}",
        target_hsm_node_hw_component_count_vec
    );

    let target_hsm_hw_summary: HashMap<String, usize> =
        calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    println!(
        "DEBUG - target HSM group hw summary:\n{:#?}",
        target_hsm_hw_summary
    );

    // Check if user request is fulfilled
    let mut success = true;
    for (hw_component, qty) in user_request_hw_summary {
        if target_hsm_hw_summary.get(&hw_component).is_none()
            || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
        {
            println!("DEBUG - hw component '{}' with quantity '{}' in user request does not comply with solution {:#?}", hw_component, qty, target_hsm_hw_summary);
            success = false;
        }
    }

    println!("DEBUG - success? {}", success);

    assert!(success)
}

#[tokio::test]
pub async fn test_hsm_hw_management_2() {
    let user_request_hw_summary = HashMap::from([("epyc".to_string(), 8)]);

    let hsm_zinal_hw_counters = vec![
        (
            "x1001c1s5b0n1".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s5b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s5b1n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b0n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b0n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b1n1".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1005c0s4b0n0".to_string(),
            HashMap::from([
                ("nvidia_a100-sxm4-80gb".to_string(), 4),
                ("epyc".to_string(), 1),
                ("memory".to_string(), 32),
            ]),
        ),
        (
            "x1005c0s4b0n1".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("nvidia_a100-sxm4-80gb".to_string(), 4),
                ("memory".to_string(), 32),
            ]),
        ),
        (
            "x1006c1s4b0n0".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
                ("memory".to_string(), 32),
            ]),
        ),
        (
            "x1006c1s4b1n0".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("memory".to_string(), 32),
                ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
            ]),
        ),
    ];

    let hsm_nodes_free_hw_conters = vec![
        (
            "x1001c1s5b0n0".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s5b0n1".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s5b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s5b1n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b0n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b0n1".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b1n0".to_string(),
            HashMap::from([("epyc".to_string(), 2), ("memory".to_string(), 16)]),
        ),
        (
            "x1001c1s6b1n1".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s7b0n0".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s7b0n1".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s7b1n0".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1001c1s7b1n1".to_string(),
            HashMap::from([("memory".to_string(), 16), ("epyc".to_string(), 2)]),
        ),
        (
            "x1005c0s4b0n0".to_string(),
            HashMap::from([
                ("nvidia_a100-sxm4-80gb".to_string(), 4),
                ("epyc".to_string(), 1),
                ("memory".to_string(), 32),
            ]),
        ),
        (
            "x1005c0s4b0n1".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("nvidia_a100-sxm4-80gb".to_string(), 4),
                ("memory".to_string(), 32),
            ]),
        ),
        (
            "x1006c1s4b0n0".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
                ("memory".to_string(), 32),
            ]),
        ),
        (
            "x1006c1s4b1n0".to_string(),
            HashMap::from([
                ("epyc".to_string(), 1),
                ("memory".to_string(), 32),
                ("amd instinct mi200 (mcm) oam lc".to_string(), 8),
            ]),
        ),
    ];

    let (target_hsm_node_hw_component_count_vec, parent_hsm_node_hw_component_count_vec) =
        get_hsm_group_members_from_user_pattern(
            hsm_zinal_hw_counters,
            hsm_nodes_free_hw_conters,
            user_request_hw_summary.clone(),
        )
        .await;

    println!(
        "DEBUG - target HSM group:\n{:#?}",
        target_hsm_node_hw_component_count_vec
    );

    let target_hsm_hw_summary: HashMap<String, usize> =
        calculate_hsm_hw_component_summary(&target_hsm_node_hw_component_count_vec);

    println!(
        "DEBUG - target HSM group hw summary:\n{:#?}",
        target_hsm_hw_summary
    );

    // Check if user request is fulfilled
    let mut success = true;
    for (hw_component, qty) in user_request_hw_summary {
        if target_hsm_hw_summary.get(&hw_component).is_none()
            || qty > *target_hsm_hw_summary.get(&hw_component).unwrap()
        {
            println!("DEBUG - hw component '{}' with quantity '{}' in user request does not comply with solution {:#?}", hw_component, qty, target_hsm_hw_summary);
            success = false;
        }
    }

    println!("DEBUG - success? {}", success);

    assert!(success)
}
