use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
  time::Instant,
};

use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
};
use anyhow::{Context, Error, bail};
use comfy_table::{Color, Table};
use manta_backend_dispatcher::{
  interfaces::hsm::{group::GroupTrait, hardware_inventory::HardwareInventory},
  types::NodeSummary,
};
use tokio::sync::Semaphore;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Maximum number of concurrent hardware inventory requests.
const HW_INVENTORY_CONCURRENCY_LIMIT: usize = 15;

/// Divisor to convert MiB to GiB.
const MIB_PER_GIB: usize = 1024;

/// Display hardware inventory for a cluster.
pub async fn exec(
  backend: StaticBackendDispatcher,
  site_name: &str,
  hsm_group_name_arg_opt: Option<&str>,
  settings_hsm_group_name_opt: Option<&str>,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, site_name).await?;

  let target_hsm_group_vec = get_groups_names_available(
    &backend,
    &shasta_token,
    hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  let hsm_group_name = target_hsm_group_vec
    .first()
    .context("No HSM groups available for this user")?;

  // Target HSM group
  let hsm_group = backend
    .get_group(&shasta_token, hsm_group_name)
    .await
    .context("Failed to get HSM group")?;

  // Get target HSM group members
  let hsm_group_target_members = hsm_group
    .members
    .unwrap_or_default()
    .ids
    .unwrap_or_default();

  log::debug!(
    "Get HW artifacts for nodes in HSM group '{}' and members {:?}",
    hsm_group.label,
    hsm_group_target_members
  );

  let mut hsm_summary: Vec<NodeSummary> = Vec::new();

  let start_total = Instant::now();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(HW_INVENTORY_CONCURRENCY_LIMIT));
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

      let node_hw_inventory_value_opt = match hw_inventory_value {
        Ok(value) => value.pointer("/Nodes/0").cloned(),
        Err(e) => {
          log::error!(
            "Failed to get HW inventory for '{}': {}",
            hsm_member_string,
            e
          );
          None
        }
      };

      match node_hw_inventory_value_opt {
        Some(node_hw_inventory) => {
          NodeSummary::from_csm_value(node_hw_inventory.clone())
        }
        None => NodeSummary {
          xname: hsm_member_string,
          ..Default::default()
        },
      }
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
  }

  let duration = start_total.elapsed();

  log::info!(
    "Time elapsed in http calls to get hw inventory for HSM '{}' is: {:?}",
    hsm_group_name,
    duration
  );

  if output_opt.is_some_and(|output| output.eq("json")) {
    for node_summary in &hsm_summary {
      println!(
        "{}",
        serde_json::to_string_pretty(&node_summary)
          .context("Failed to serialize node summary",)?
      );
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
    bail!("'output' value not valid");
  }

  Ok(())
}

fn calculate_hsm_hw_component_summary(
  node_summary_vec: &[NodeSummary],
) -> HashMap<String, usize> {
  let mut node_hw_component_summary: HashMap<String, usize> = HashMap::new();

  for node_summary in node_summary_vec {
    for artifact_summary in &node_summary.processors {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.to_string())
          .and_modify(|summary_quantity| *summary_quantity += 1)
          .or_insert(1);
      }
    }
    for artifact_summary in &node_summary.node_accels {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.to_string())
          .and_modify(|summary_quantity| *summary_quantity += 1)
          .or_insert(1);
      }
    }
    for artifact_summary in &node_summary.memory {
      let memory_capacity = artifact_summary
        .info
        .as_deref()
        .unwrap_or("ERROR NA")
        .split(' ')
        .collect::<Vec<_>>()
        .first()
        .copied()
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap_or(0);
      node_hw_component_summary
        .entry(artifact_summary.r#type.to_string() + " (GiB)")
        .and_modify(|summary_quantity| {
          *summary_quantity += memory_capacity / MIB_PER_GIB;
        })
        .or_insert(memory_capacity / MIB_PER_GIB);
    }
    for artifact_summary in &node_summary.node_hsn_nics {
      if let Some(info) = artifact_summary.info.as_ref() {
        node_hw_component_summary
          .entry(info.to_string())
          .and_modify(|summary_quantity| *summary_quantity += 1)
          .or_insert(1);
      }
    }
  }

  node_hw_component_summary
}

fn get_cluster_hw_pattern(
  hsm_summary: Vec<NodeSummary>,
) -> HashMap<String, usize> {
  let mut hsm_node_hw_component_count_hashmap: HashMap<String, usize> =
    HashMap::new();

  for node_summary in hsm_summary {
    for processor in node_summary.processors {
      if let Some(info) = processor.info {
        hsm_node_hw_component_count_hashmap
          .entry(info.chars().filter(|c| !c.is_whitespace()).collect())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }

    for node_accel in node_summary.node_accels {
      if let Some(info) = node_accel.info {
        hsm_node_hw_component_count_hashmap
          .entry(info.chars().filter(|c| !c.is_whitespace()).collect())
          .and_modify(|qty| *qty += 1)
          .or_insert(1);
      }
    }

    for memory_dimm in node_summary.memory {
      let memory_capacity = memory_dimm
        .info
        .unwrap_or_else(|| "0".to_string())
        .split(' ')
        .next()
        .unwrap_or("0")
        .to_string()
        .parse::<usize>()
        .unwrap_or(0);

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

fn print_to_terminal_cluster_hw_pattern(
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

fn print_table_summary(hsm_hw_component_summary_vec: &HashMap<String, usize>) {
  let headers = ["HW Component", "Quantity"];

  let mut table = comfy_table::Table::new();

  table.set_header(headers);

  for (hw_component, qty) in hsm_hw_component_summary_vec {
    table.add_row(vec![hw_component, &qty.to_string()]);
  }

  println!("{table}");
}

/// Count occurrences of hardware component info strings.
///
/// Takes an iterator of `Option<String>` info values (one per
/// hardware component instance), filters out `None`s, and returns
/// both the per-component count map and the set of unique names.
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

fn print_table_details(node_summary_vec: &[NodeSummary]) {
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
        // node does not contain hardware but it was requested by the user
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
