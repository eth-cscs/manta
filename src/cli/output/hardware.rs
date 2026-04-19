use std::collections::{HashMap, HashSet};

use comfy_table::{Cell, Color, Table};
use manta_backend_dispatcher::types::NodeSummary;

/// Print HSM group name + HW component counts as a single line.
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

/// Print a summary table of HW component counts.
pub fn print_table_summary(
  hsm_hw_component_summary_vec: &HashMap<String, usize>,
) {
  let headers = ["HW Component", "Quantity"];

  let mut table = comfy_table::Table::new();

  table.set_header(headers);

  for (hw_component, qty) in hsm_hw_component_summary_vec {
    table.add_row(vec![hw_component, &qty.to_string()]);
  }

  println!("{table}");
}

/// Print a detailed per-node HW component table.
pub fn print_table_details(node_summary_vec: &[NodeSummary]) {
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

    let (proc_counts, proc_keys) = count_hw_components(
      node_summary.processors.iter().map(|p| p.info.clone()),
    );
    processor_set.extend(proc_keys);
    node_hw_component_count_hashmap.extend(proc_counts);

    let (accel_counts, accel_keys) = count_hw_components(
      node_summary.node_accels.iter().map(|a| a.info.clone()),
    );
    accelerator_set.extend(accel_keys);
    node_hw_component_count_hashmap.extend(accel_counts);

    let (mem_counts, mem_keys) = count_hw_components(
      node_summary
        .memory
        .iter()
        .map(|m| Some(m.info.clone().unwrap_or_else(|| "ERROR".to_string()))),
    );
    memory_set.extend(mem_keys);
    node_hw_component_count_hashmap.extend(mem_counts);

    let (hsn_counts, hsn_keys) = count_hw_components(
      node_summary.node_hsn_nics.iter().map(|h| h.info.clone()),
    );
    hsn_set.extend(hsn_keys);
    node_hw_component_count_hashmap.extend(hsn_counts);

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

/// Count occurrences of hardware component info strings.
fn count_hw_components(
  info_iter: impl Iterator<Item = Option<String>>,
) -> (HashMap<String, usize>, HashSet<String>) {
  let mut counts: HashMap<String, usize> = HashMap::new();
  for info in info_iter.flatten() {
    counts.entry(info).and_modify(|q| *q += 1).or_insert(1);
  }
  let keys: HashSet<String> = counts.keys().cloned().collect();
  (counts, keys)
}

/// Build a table of hardware component counts per node.
pub fn get_table(
  user_defined_hw_component_vec: &[String],
  hsm_node_hw_pattern_vec: &[(String, HashMap<String, usize>)],
) -> Table {
  let hsm_hw_component_vec: Vec<String> = hsm_node_hw_pattern_vec
    .iter()
    .flat_map(|(_xname, node_pattern_hashmap)| {
      node_pattern_hashmap.keys().cloned()
    })
    .collect();

  let mut all_hw_component_vec =
    [hsm_hw_component_vec, user_defined_hw_component_vec.to_vec()].concat();

  all_hw_component_vec.sort();
  all_hw_component_vec.dedup();

  let mut table = comfy_table::Table::new();

  table.set_header(
    [vec!["Node".to_string()], all_hw_component_vec.clone()].concat(),
  );

  for (xname, node_pattern_hashmap) in hsm_node_hw_pattern_vec {
    let mut row: Vec<comfy_table::Cell> = Vec::new();
    row.push(
      comfy_table::Cell::new(xname.clone())
        .set_alignment(comfy_table::CellAlignment::Center),
    );
    for hw_component in &all_hw_component_vec {
      if hw_component.to_uppercase().contains("ERROR")
        && node_pattern_hashmap
          .get(hw_component)
          .is_some_and(|counter| *counter > 0)
      {
        let counter =
          node_pattern_hashmap.get(hw_component).copied().unwrap_or(0);
        row.push(
          comfy_table::Cell::new(format!("⚠️  ({})", counter))
            .fg(Color::Yellow)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else if user_defined_hw_component_vec.contains(hw_component)
        && node_pattern_hashmap.contains_key(hw_component)
      {
        let counter =
          node_pattern_hashmap.get(hw_component).copied().unwrap_or(0);
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
    }
    table.add_row(row);
  }

  table
}

/// Print a per-node hardware inventory table (for single-node view).
pub fn print_node_table(node_summary_vec: &[NodeSummary]) {
  let mut table = Table::new();

  table.set_header(vec![
    "Node XName",
    "Component XName",
    "Component Type",
    "Component Info",
  ]);

  for node_summary in node_summary_vec {
    for processor in &node_summary.processors {
      table.add_row(vec![
        Cell::new(node_summary.xname.clone()),
        Cell::new(processor.xname.clone()),
        Cell::new(processor.r#type.clone()),
        Cell::new(
          processor
            .info
            .clone()
            .unwrap_or_else(|| "*** Missing info".to_string()),
        ),
      ]);
    }

    for memory in &node_summary.memory {
      table.add_row(vec![
        Cell::new(node_summary.xname.clone()),
        Cell::new(memory.xname.clone()),
        Cell::new(memory.r#type.clone()),
        Cell::new(
          memory
            .info
            .clone()
            .unwrap_or_else(|| "*** Missing info".to_string()),
        ),
      ]);
    }

    for node_accel in &node_summary.node_accels {
      table.add_row(vec![
        Cell::new(node_summary.xname.clone()),
        Cell::new(node_accel.xname.clone()),
        Cell::new(node_accel.r#type.clone()),
        Cell::new(
          node_accel
            .info
            .clone()
            .unwrap_or_else(|| "*** Missing info".to_string()),
        ),
      ]);
    }

    for node_hsn_nic in &node_summary.node_hsn_nics {
      table.add_row(vec![
        Cell::new(node_summary.xname.clone()),
        Cell::new(node_hsn_nic.xname.clone()),
        Cell::new(node_hsn_nic.r#type.clone()),
        Cell::new(
          node_hsn_nic
            .info
            .clone()
            .unwrap_or_else(|| "*** Missing info".to_string()),
        ),
      ]);
    }
  }

  println!("{table}");
}
