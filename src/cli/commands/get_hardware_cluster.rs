use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
  time::Instant,
};

use comfy_table::{Color, Table};
use manta_backend_dispatcher::{
  interfaces::hsm::{group::GroupTrait, hardware_inventory::HardwareInventory},
  types::NodeSummary,
};
use tokio::sync::Semaphore;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: StaticBackendDispatcher,
  shasta_token: &str,
  hsm_group_name: &str,
  output_opt: Option<&String>,
) {
  let pipe_size = 15;

  // Target HSM group
  let hsm_group = backend
    .get_group(shasta_token, hsm_group_name)
    .await
    .unwrap();

  // Get target HSM group members
  let hsm_group_target_members =
    hsm_group.members.unwrap().ids.unwrap_or_default();

  log::debug!(
    "Get HW artifacts for nodes in HSM group '{}' and members {:?}",
    hsm_group.label,
    hsm_group_target_members
  );

  let mut hsm_summary: Vec<NodeSummary> = Vec::new();

  let start_total = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(pipe_size));
  // make it faster

  let num_hsm_group_members = hsm_group_target_members.len();

  let mut i = 1;

  // Calculate number of digits of a number
  let width = num_hsm_group_members.checked_ilog10().unwrap_or(0) as usize + 1;

  // Get HW inventory details for target HSM group
  for hsm_member in hsm_group_target_members.iter() {
    log::info!(
            "\rGetting hw components for node '{hsm_member}' [{:>width$}/{num_hsm_group_members}]",
            i + 1
        );

    let backend_cp = backend.clone();
    let shasta_token_string = shasta_token.to_string(); // TODO: make it static
    let hsm_member_string = hsm_member.to_string(); // TODO: make it static
                                                    //
    let permit = Arc::clone(&sem).acquire_owned().await;

    // log::debug!("Getting HW inventory details for node '{}'", hsm_member);

    tasks.spawn(async move {
      let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885
      let hw_inventory_value = backend_cp
        .get_inventory_hardware_query(
          &shasta_token_string,
          &hsm_member_string,
          None,
          None,
          None,
          None,
          None,
        )
        .await;

      let node_hw_inventory_value_opt =
        hw_inventory_value.unwrap().pointer("/Nodes/0").cloned();

      let node_hw_inventory = match node_hw_inventory_value_opt {
        Some(node_hw_inventory) => {
          NodeSummary::from_csm_value(node_hw_inventory.clone())
        }
        None => {
          let mut node_summary_default = NodeSummary::default();
          node_summary_default.xname = hsm_member_string;
          node_summary_default
        }
      };

      node_hw_inventory
    });

    i += 1;
  }

  while let Some(message) = tasks.join_next().await {
    match message {
      Ok(node_hw_inventory) => {
        hsm_summary.push(node_hw_inventory);
      }
      Err(e) => log::error!("Failed fetching node hardware information: {}", e),
    }
    /* if let Ok(node_hw_inventory) = message {
        hsm_summary.push(node_hw_inventory);
    } else {
        log::error!("Failed fetching node hw information");
    } */
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
    let hsm_node_hw_component_count_hashmap =
      get_cluster_hw_pattern(hsm_summary);
    print_to_terminal_cluster_hw_pattern(
      hsm_group_name,
      hsm_node_hw_component_count_hashmap,
    )
  } else if output_opt.is_some_and(|output| output.eq("details")) {
    print_table_details(&hsm_summary);
  } else if output_opt.is_some_and(|output| output.eq("summary")) {
    let hsm_node_hw_component_summary =
      calculate_hsm_hw_component_summary(&hsm_summary);

    print_table_summary(&hsm_node_hw_component_summary);
  } else {
    eprintln!("'output' value not valid. Exit");
  }
}

pub fn calculate_hsm_hw_component_summary(
  node_summary_vec: &Vec<NodeSummary>,
) -> HashMap<String, usize> {
  let mut node_hw_component_summary: HashMap<String, usize> = HashMap::new();

  for node_summary in node_summary_vec {
    for artifact_summary in &node_summary.processors {
      node_hw_component_summary
        .entry(artifact_summary.info.as_ref().unwrap().to_string())
        .and_modify(|summary_quantity| *summary_quantity += 1)
        .or_insert(1);
    }
    for artifact_summary in &node_summary.node_accels {
      node_hw_component_summary
        .entry(artifact_summary.info.as_ref().unwrap().to_string())
        .and_modify(|summary_quantity| *summary_quantity += 1)
        .or_insert(1);
    }
    for artifact_summary in &node_summary.memory {
      let memory_capacity = artifact_summary
        .info
        .as_ref()
        .unwrap_or(&"ERROR NA".to_string())
        .split(' ')
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .parse::<usize>()
        .unwrap_or(0);
      node_hw_component_summary
        .entry(artifact_summary.r#type.to_string() + " (GiB)")
        .and_modify(|summary_quantity| {
          *summary_quantity += memory_capacity / 1024;
        })
        .or_insert(memory_capacity / 1024);
    }
    for artifact_summary in &node_summary.node_hsn_nics {
      node_hw_component_summary
        .entry(artifact_summary.info.as_ref().unwrap().to_string())
        .and_modify(|summary_quantity| *summary_quantity += 1)
        .or_insert(1);
    }
  }

  node_hw_component_summary
}

pub fn get_cluster_hw_pattern(
  hsm_summary: Vec<NodeSummary>,
) -> HashMap<String, usize> {
  let mut hsm_node_hw_component_count_hashmap: HashMap<String, usize> =
    HashMap::new();

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
        .and_modify(|memory_capacity_aux| {
          *memory_capacity_aux += memory_capacity
        })
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

pub fn print_table_summary(
  hsm_hw_component_summary_vec: &HashMap<String, usize>,
) {
  /* let headers = hsm_hw_component_summary_vec.keys();

  let mut table = comfy_table::Table::new();

  table.set_header(headers);
  table.add_row(comfy_table::Row::from(
      hsm_hw_component_summary_vec.values(),
  ));

  println!("{table}"); */

  let headers = ["HW Component", "Quantity"];

  let mut table = comfy_table::Table::new();

  table.set_header(headers);

  for (hw_component, qty) in hsm_hw_component_summary_vec {
    table.add_row(vec![hw_component, &qty.to_string()]);
  }

  println!("{table}");
}

pub fn print_table_details(node_summary_vec: &Vec<NodeSummary>) {
  let mut hsm_node_hw_component_count_hashmap_vec: Vec<(
    String,
    HashMap<String, usize>,
  )> = vec![];

  let mut processor_set: HashSet<String> = HashSet::new();
  let mut accelerator_set: HashSet<String> = HashSet::new();
  let mut memory_set: HashSet<String> = HashSet::new();
  let mut hsn_set: HashSet<String> = HashSet::new();

  for node_summary in node_summary_vec {
    let mut node_hw_component_count_hashmap: HashMap<String, usize> =
      HashMap::new();

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

    let hw_component_set: HashSet<String> =
      processor_count_hashmap.keys().cloned().collect();
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

    let hw_component_set: HashSet<String> =
      accelerator_count_hashmap.keys().cloned().collect();
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

    let hw_component_set: HashSet<String> =
      memory_count_hashmap.keys().cloned().collect();
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

    let hw_component_set: HashSet<String> =
      hsn_nic_count_hashmap.keys().cloned().collect();
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

  let hw_configuration_table =
    get_table(&headers, &hsm_node_hw_component_count_hashmap_vec);

  println!("{hw_configuration_table}");
}

pub fn get_table(
  user_defined_hw_componet_vec: &[String],
  hsm_node_hw_pattern_vec: &[(String, HashMap<String, usize>)],
) -> Table {
  let hsm_hw_component_vec: Vec<String> = hsm_node_hw_pattern_vec
    .iter()
    .flat_map(|(_xname, node_pattern_hashmap)| {
      node_pattern_hashmap.keys().cloned()
    })
    .collect();

  let mut all_hw_component_vec =
    [hsm_hw_component_vec, user_defined_hw_componet_vec.to_vec()].concat();

  all_hw_component_vec.sort();
  all_hw_component_vec.dedup();

  let mut table = comfy_table::Table::new();

  table.set_header(
    [vec!["Node".to_string()], all_hw_component_vec.clone()].concat(),
  );

  for (xname, node_pattern_hashmap) in hsm_node_hw_pattern_vec {
    // println!("node_pattern_hashmap: {:?}", node_pattern_hashmap);

    let mut row: Vec<comfy_table::Cell> = Vec::new();
    // Node xname table cell
    row.push(
      comfy_table::Cell::new(xname.clone())
        .set_alignment(comfy_table::CellAlignment::Center),
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
