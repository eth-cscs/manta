use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use comfy_table::{Color, Table};
use mesa::hsm::hw_components::NodeSummary;
use tokio::sync::Semaphore;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: &str,
    output_opt: Option<&String>,
) {
    // Target HSM group
    let hsm_group_value = mesa::hsm::group::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&hsm_group_name.to_string()),
    )
    .await
    .unwrap()
    .first()
    .unwrap()
    .clone();

    log::info!(
        "Get HW artifacts for nodes in HSM group '{:?}' and members {:?}",
        hsm_group_value["label"],
        hsm_group_value["members"]
    );

    // Get target HSM group members
    let hsm_group_target_members =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_value(&hsm_group_value);

    let mut hsm_summary: Vec<NodeSummary> = Vec::new();

    let start_total = Instant::now();

    let mut tasks = tokio::task::JoinSet::new();

    let sem = Arc::new(Semaphore::new(5)); // CSM 1.3.1 higher number of concurrent tasks won't
                                           // make it faster

    // Get HW inventory details for target HSM group
    for hsm_member in hsm_group_target_members.clone() {
        let shasta_token_string = shasta_token.to_string(); // TODO: make it static
        let shasta_base_url_string = shasta_base_url.to_string(); // TODO: make it static
        let shasta_root_cert_vec = shasta_root_cert.to_vec();
        let hsm_member_string = hsm_member.to_string(); // TODO: make it static
                                                        //
        let permit = Arc::clone(&sem).acquire_owned().await;

        log::info!("Getting HW inventory details for node '{}'", hsm_member);
        tasks.spawn(async move {
            let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885
            mesa::hsm::hw_inventory::shasta::http_client::get_hw_inventory(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                &hsm_member_string,
            )
            .await
            .unwrap()
        });
    }

    while let Some(message) = tasks.join_next().await {
        if let Ok(mut node_hw_inventory) = message {
            node_hw_inventory = node_hw_inventory.pointer("/Nodes/0").unwrap().clone();
            let node_summary = NodeSummary::from_csm_value(node_hw_inventory.clone());
            hsm_summary.push(node_summary);
        } else {
            log::error!("Failed procesing/fetching node hw information");
        }
    }

    let duration = start_total.elapsed();

    log::info!(
        "Time elapsed in http calls to get hw inventory for HSM '{}' is: {:?}",
        hsm_group_name,
        duration
    );

    if output_opt.is_some_and(|output| output.eq("json")) {
        for node_summary in &hsm_summary {
            println!("{}", serde_json::to_string_pretty(&node_summary).unwrap());
        }
    } else if output_opt.is_some_and(|output| output.eq("pattern")) {
        let hsm_node_hw_component_count_hashmap = get_cluster_hw_pattern(hsm_summary);
        print_to_terminal_cluster_hw_pattern(hsm_group_name, hsm_node_hw_component_count_hashmap)
    } else if output_opt.is_some_and(|output| output.eq("details")) {
        print_table_details(&hsm_summary);
    } else if output_opt.is_some_and(|output| output.eq("summary")) {
        let hsm_node_hw_component_summary =
            mesa::hsm::hw_inventory::mesa::utils::calculate_hsm_hw_component_summary(&hsm_summary);

        print_table_summary(&hsm_node_hw_component_summary);
    } else {
        eprintln!("'output' value not valid. Exit");
    }
}

pub fn get_cluster_hw_pattern(hsm_summary: Vec<NodeSummary>) -> HashMap<String, usize> {
    let mut hsm_node_hw_component_count_hashmap: HashMap<String, usize> = HashMap::new();

    for node_summary in hsm_summary {
        for processor in node_summary.processors {
            hsm_node_hw_component_count_hashmap
                .entry(
                    processor
                        .info
                        .unwrap()
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect(),
                )
                .and_modify(|qty| *qty += 1)
                .or_insert(1);
        }

        for node_accel in node_summary.node_accels {
            hsm_node_hw_component_count_hashmap
                .entry(
                    node_accel
                        .info
                        .unwrap()
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect(),
                )
                .and_modify(|qty| *qty += 1)
                .or_insert(1);
        }

        for memory_dimm in node_summary.memory {
            let memory_capacity = memory_dimm
                .clone()
                .info
                .unwrap_or("0".to_string())
                .split(' ')
                .collect::<Vec<_>>()
                .first()
                .unwrap()
                .to_string()
                .parse::<usize>()
                .unwrap();

            hsm_node_hw_component_count_hashmap
                .entry("memory".to_string())
                .and_modify(|memory_capacity_aux| *memory_capacity_aux += memory_capacity)
                .or_insert(memory_capacity);
        }
    }

    hsm_node_hw_component_count_hashmap
}

pub fn print_to_terminal_cluster_hw_pattern(
    hsm_group_name: &str,
    hsm_node_hw_component_count_hashmap: HashMap<String, usize>,
) {
    println!(
        "{}:{}",
        hsm_group_name,
        hsm_node_hw_component_count_hashmap
            .iter()
            .map(|(hw_component, qty)| format!("{}:{}", hw_component, qty))
            .collect::<Vec<String>>()
            .join(":")
    );
}

pub fn print_table_summary(hsm_hw_component_summary_vec: &HashMap<String, usize>) {
    let headers = hsm_hw_component_summary_vec.keys();

    let mut table = comfy_table::Table::new();

    table.set_header(headers);
    table.add_row(comfy_table::Row::from(
        hsm_hw_component_summary_vec.values(),
    ));

    println!("{table}");
}

pub fn print_table_details(node_summary_vec: &Vec<NodeSummary>) {
    let mut hsm_node_hw_component_count_hashmap_vec: Vec<(String, HashMap<String, usize>)> = vec![];

    let mut processor_set: HashSet<String> = HashSet::new();
    let mut accelerator_set: HashSet<String> = HashSet::new();
    let mut memory_set: HashSet<String> = HashSet::new();
    let mut hsn_set: HashSet<String> = HashSet::new();

    for node_summary in node_summary_vec {
        let mut node_hw_component_count_hashmap: HashMap<String, usize> = HashMap::new();

        let processor_info_vec: Vec<String> = node_summary
            .processors
            .iter()
            .map(|processor| processor.info.as_ref().unwrap().clone())
            .collect();

        let mut processor_count_hashmap: HashMap<String, usize> = HashMap::new();
        for processor_info in &processor_info_vec {
            processor_count_hashmap
                .entry(processor_info.to_string())
                .and_modify(|qty| *qty += 1)
                .or_insert(1);
        }

        let hw_component_set: HashSet<String> = processor_count_hashmap.keys().cloned().collect();
        processor_set.extend(hw_component_set);
        node_hw_component_count_hashmap.extend(processor_count_hashmap.clone());

        let accelerator_info_vec: Vec<String> = node_summary
            .node_accels
            .iter()
            .map(|node_accel| node_accel.info.as_ref().unwrap().clone())
            .collect();

        let mut accelerator_count_hashmap: HashMap<String, usize> = HashMap::new();
        for accelerator_info in &accelerator_info_vec {
            accelerator_count_hashmap
                .entry(accelerator_info.to_string())
                .and_modify(|qty| *qty += 1)
                .or_insert(1);
        }

        let hw_component_set: HashSet<String> = accelerator_count_hashmap.keys().cloned().collect();
        accelerator_set.extend(hw_component_set);
        node_hw_component_count_hashmap.extend(accelerator_count_hashmap);

        let memory_info_vec: Vec<String> = node_summary
            .memory
            .iter()
            .map(|mem| mem.info.as_ref().unwrap_or(&"ERROR".to_string()).clone())
            .collect();

        let mut memory_count_hashmap: HashMap<String, usize> = HashMap::new();
        for memory_info in &memory_info_vec {
            memory_count_hashmap
                .entry(memory_info.to_string())
                .and_modify(|qty| *qty += 1)
                .or_insert(1);
        }

        let hw_component_set: HashSet<String> = memory_count_hashmap.keys().cloned().collect();
        memory_set.extend(hw_component_set);
        node_hw_component_count_hashmap.extend(memory_count_hashmap);

        let hsn_nic_info_vec: Vec<String> = node_summary
            .node_hsn_nics
            .iter()
            .map(|hsn_nic| hsn_nic.info.as_ref().unwrap().clone())
            .collect();

        let mut hsn_nic_count_hashmap: HashMap<String, usize> = HashMap::new();
        for hsn_nic_info in &hsn_nic_info_vec {
            hsn_nic_count_hashmap
                .entry(hsn_nic_info.to_string())
                .and_modify(|qty| *qty += 1)
                .or_insert(1);
        }

        let hw_component_set: HashSet<String> = hsn_nic_count_hashmap.keys().cloned().collect();
        hsn_set.extend(hw_component_set);
        node_hw_component_count_hashmap.extend(hsn_nic_count_hashmap);

        hsm_node_hw_component_count_hashmap_vec
            .push((node_summary.xname.clone(), node_hw_component_count_hashmap))
    }

    let headers = Vec::from_iter(
        [
            Vec::from_iter(processor_set),
            Vec::from_iter(accelerator_set),
            Vec::from_iter(memory_set),
            Vec::from_iter(hsn_set),
        ]
        .concat(),
    );

    hsm_node_hw_component_count_hashmap_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let hw_configuration_table = get_table(&headers, &hsm_node_hw_component_count_hashmap_vec);

    println!("{hw_configuration_table}");
}

pub fn get_table(
    user_defined_hw_componet_vec: &[String],
    hsm_node_hw_pattern_vec: &[(String, HashMap<String, usize>)],
) -> Table {
    let hsm_hw_component_vec: Vec<String> = hsm_node_hw_pattern_vec
        .iter()
        .flat_map(|(_xname, node_pattern_hashmap)| node_pattern_hashmap.keys().cloned())
        .collect();

    let mut all_hw_component_vec =
        [hsm_hw_component_vec, user_defined_hw_componet_vec.to_vec()].concat();

    all_hw_component_vec.sort();
    all_hw_component_vec.dedup();

    let mut table = comfy_table::Table::new();

    table.set_header([vec!["Node".to_string()], all_hw_component_vec.clone()].concat());

    for (xname, node_pattern_hashmap) in hsm_node_hw_pattern_vec {
        // println!("node_pattern_hashmap: {:?}", node_pattern_hashmap);

        let mut row: Vec<comfy_table::Cell> = Vec::new();
        // Node xname table cell
        row.push(
            comfy_table::Cell::new(xname.clone()).set_alignment(comfy_table::CellAlignment::Center),
        );
        // User hw components table cell
        for hw_component in &all_hw_component_vec {
            if hw_component.to_uppercase().contains("ERROR")
                && node_pattern_hashmap
                    .get(hw_component)
                    .is_some_and(|counter| *counter > 0)
            {
                let counter = node_pattern_hashmap.get(hw_component).unwrap();
                row.push(
                    comfy_table::Cell::new(format!("⚠️  ({})", counter))
                        .fg(Color::Yellow)
                        .set_alignment(comfy_table::CellAlignment::Center),
                );
            } else if user_defined_hw_componet_vec.contains(hw_component)
                && node_pattern_hashmap.contains_key(hw_component)
            {
                let counter = node_pattern_hashmap.get(hw_component).unwrap();
                row.push(
                    comfy_table::Cell::new(format!("✅ ({})", counter,))
                        .fg(Color::Green)
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
        table.add_row(row);
    }

    table
}
