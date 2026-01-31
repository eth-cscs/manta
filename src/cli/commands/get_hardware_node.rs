use crate::common::{
  authorization::validate_target_hsm_members,
};
use crate::common;
use anyhow::Error;
use comfy_table::{Cell, Table};
use manta_backend_dispatcher::{
  interfaces::hsm::hardware_inventory::HardwareInventory, types::NodeSummary,
};
use serde_json::Value;
use std::string::ToString;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  xnames: &str,
  type_artifact_opt: Option<&String>,
  output_opt: Option<&String>,
) -> Result<(), Error> {
  let shasta_token = common::authentication::get_api_token(backend, site_name).await?;

  let xname_vec: Vec<String> = xnames.split(',').map(str::to_string).collect();

  validate_target_hsm_members(backend, &shasta_token, &xname_vec).await?;

  let mut node_hw_inventory = &backend
    .get_inventory_hardware_query(
      &shasta_token,
      xnames,
      None,
      None,
      None,
      None,
      None,
    )
    .await
    .unwrap();

  node_hw_inventory = node_hw_inventory.pointer("/Nodes/0").ok_or_else(|| {
      Error::msg(format!(
        "ERROR - json section '/Nodes' missing in json response API for node '{}'",
        xnames
      ))
    })?;

  if let Some(type_artifact) = type_artifact_opt {
    node_hw_inventory = &node_hw_inventory
      .as_array()
      .unwrap()
      .iter()
      .find(|&node| node.get("ID").and_then(Value::as_str).unwrap().eq(xnames))
      .unwrap()[type_artifact];
  }

  let node_summary = NodeSummary::from_csm_value(node_hw_inventory.clone());

  if output_opt.is_some() && output_opt.unwrap().eq("json") {
    println!("{}", serde_json::to_string_pretty(&node_summary).unwrap());
  } else {
    print_table(&[node_summary].to_vec());
  }

  Ok(())
}

pub fn print_table(node_summary_vec: &Vec<NodeSummary>) {
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
            .unwrap_or("*** Missing info".to_string()),
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
            .unwrap_or("*** Missing info".to_string()),
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
            .clone()
            .info
            .unwrap_or("*** Missing info".to_string()),
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
            .clone()
            .info
            .unwrap_or("*** Missing info".to_string()),
        ),
      ]);
    }
  }

  println!("{table}");
}
