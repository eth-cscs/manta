use crate::shasta::hsm;

/// Manipulates HSM groups `apply hsm-group (<hsm group name>(:<prop>)*:<num nodes>(:<prop>*:<num nodes>))+ or <hsm_group1>:<prop111>:<prop112>:...:<num_nodes>:<prop121>:<prop122>:...:<num_nodes>:... <hsm_group2>:<prop21>:<prop22>:...:<num_nodes>: ...`
/// each propX is a propery from https://api.cmn.alps.cscs.ch/apis/smd/hsm/v2/Inventory/Hardware/Query/{xname}
/// for each
/// <hsm_group_name>:<prop11>:<prop12>:...:<num_nodes>:<prop21>:<prop22>:...:<num_nodes>:...
///   - Calculate `delta number`: number of nodes affected. Eg zinal hsm group has 4 nodes and
///   apply hsm-group has zinal:6, this means delta number would be +2 meaning 2 nodes needs to be
///   removed from the free pool and added to the zinal hsm group. Eg 2 zinal hsm group has 4 nodes and apply
///   hsm-group has zinal:2, this means delta number is -2 meaning 2 nodes from zinal hsm group
///   needs to be moved to the free pool hsm group
///   - Calculate `delta operation`:
///      - `remove`: if delta number is negative, then nodes are movec form <hsm> group to `free pool` hsm group
///      - `add`: if delta number if positive, then nodes are moved from the `free pool` hsm group to the <hsm> group
///   - Fetch nodes props from https://api.cmn.alps.cscs.ch/apis/smd/hsm/v2/Inventory/Hardware/Query/{xname} from all nodes in HSM groups affected
///   - Create a plan:
///      1) adds all nodes with negative delta numbers to the free pool
///      2) calculates there are enough resources to add all nodes to all hsm groups
///   - Checks operations for all hsm groups can be fullfilled
///      NOTE: apply hsm-group is a unique transaction, if any hsm group update can't be done, then
///      the command will fail.
///   - Operates the deltas for each group

// Get a list of members in current pattern/HSM group with specified props
// Calculate the deltas (is this hsm group short of members with specified props or do we
// need to add more nodes from the free pool?)
// NOTE: if delta is integer then -2 means moving 2 nodes from current HSM group to free pool and +2 means
// adding 2 nodes from the free pool to the current HSM group. This means we already
// integrated the delta operation into delta number
// Calculate the new free pool
// Evaluate whether we have enough resources in the free pool to satify all positive deltas
// (eg +2)
// NOTE: if delta number is 0, then through a WARNING

// Also, think we may have nodes which fulfills most more than 1 pattern eg x1007c1s2b0n0 has x1 `AMD EPYC` and x4 `NVIDIA_A100` whcih means this node alone would fulfill these 2 patterns <hsm>:a100:2 and <hsm>:epyc:1 . The point I am trying to make is that we should match each node with as much unfulfilled patterns we can (by testing each pattern left with each node) in order to use nodes more efficiently, otherwise we may be undersubscribing??? nodes because we may not select the nodes that matches the maximun amount of patterns, therefore the number of nodes allocated may be higher than what is really needed.
//
// NOTE 2: Hypothesis about the number after the hardware profile refers to number of components of
// that type and not number of nodes... how should manta deal with the situation where we have x4 nodes (x4 A100, x1 AMD EPYC), (x4 A100,
// x1 AMD EPYC), (x2 AMD EPYC), (x8 A100) . And the user wants <HSM>:a100:8:epyc:2 ? should user
// get (x8 a100) and (x2 EPYC) or (x4 A100, x1 EPYC) and (x4 A100, x1 EPYC) ?????? component
// afinity so we try to put similar components toghether in the same node??????

pub async fn exec(
    _vault_base_url: &str,
    _vault_token: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    patterns: &str,
) {
    let hsm_patterns_list: Vec<&str> = patterns.split(' ').collect();

    let hsm_groups_details = hsm::http_client::get_all_hsm_groups(shasta_token, shasta_base_url)
        .await
        .unwrap_or(Vec::new());

    for hsm_patterns in hsm_patterns_list {
        let hsm_group_name = hsm_patterns
            .chars()
            .take_while(|&char| char != ':')
            .collect::<String>();

        let hsm_group_pattern_list_vec: Vec<Vec<String>> = utils::get_pattern_vec(hsm_patterns);

        println!("hsm_group_name: {}", hsm_group_name);
        println!(
            "hsm_group_pattern_list_vec: {:?}",
            hsm_group_pattern_list_vec
        );

        let hsm_group_option = hsm_groups_details
            .iter()
            .find(|hsm_group| hsm_group["label"].eq(&hsm_group_name));

        let hsm_group_members =
            hsm::utils::get_members_from_hsm_group_serde_value(hsm_group_option.unwrap());

        println!("hsm_group_members: {:?}", hsm_group_members);

        for xname in hsm_group_members {
            let node_hw_inventory_value =
                hsm::http_client::get_hw_inventory(shasta_token, shasta_base_url, &xname)
                    .await
                    .unwrap();

            let processor_model = hsm::utils::get_list_processor_model_from_hw_inventory_value(
                &node_hw_inventory_value,
            )
            .unwrap_or_default();

            let accelerator = hsm::utils::get_list_accelerator_model_from_hw_inventory_value(
                &node_hw_inventory_value,
            )
            .unwrap_or_default();

            let memory_capacity = hsm::utils::get_list_memory_capacity_from_hw_inventory_value(
                &node_hw_inventory_value,
            )
            .unwrap_or_default();

            let mem_total_capacity = memory_capacity
                .iter()
                .map(|mem_capacity| mem_capacity.parse::<u32>().unwrap_or(0))
                .sum::<u32>();

            println!("{} - Processor:\n{:#?}", xname, processor_model);
            println!("{} - Accelerators:\n{:#?}", xname, accelerator);
            println!("{} - Memory capacity:\n{:?}", xname, mem_total_capacity);

            // 1) remove white spaces in pattern
            // 2) put all low caps
            // 3) sort by hw inventroy type text alphabetically
            // 4) aggregate all numbers of same hw inventory type text from the text pattern provided
            // by the user
        }
    }
}

pub mod utils {

    pub fn get_pattern_vec(hsm_group_patterns: &str) -> Vec<Vec<String>> {
        let mut hsm_group_pattern_list_vec: Vec<Vec<String>> = Vec::new();

        let mut hsm_group_pattern: Vec<String> = Vec::new();

        for pattern in hsm_group_patterns.split(':') {
            hsm_group_pattern.push(pattern.to_string());

            if pattern.parse::<u8>().is_ok() {
                // We found num_nodes
                hsm_group_pattern_list_vec.push(hsm_group_pattern.clone());
                hsm_group_pattern.clear();
            }
        }

        hsm_group_pattern_list_vec
    }
}
