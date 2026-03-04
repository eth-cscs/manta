use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cli_get_nodes: &clap::ArgMatches,
) -> Result<(), Error> {
  let shasta_token =
    common::authentication::get_api_token(backend, site_name).await?;

  let xname_requested: &str = cli_get_nodes
    .get_one::<String>("VALUE")
    .context("The 'xnames' argument must have values")?;
  let is_include_siblings = cli_get_nodes.get_flag("include-siblings");
  let nids_only = cli_get_nodes.get_flag("nids-only-one-line");
  let status: Option<&String> = cli_get_nodes.get_one("status");
  let output_opt: Option<&String> = cli_get_nodes.get_one("output");
  let status_summary = cli_get_nodes.get_flag("summary-status");

  // Convert user input to xname
  let node_metadata_available_vec =
    backend.get_node_metadata_available(&shasta_token).await?;

  let mut node_list = common::node_ops::from_hosts_expression_to_xname_vec(
    xname_requested,
    is_include_siblings,
    node_metadata_available_vec,
  )
  .await
  .map_err(|e| {
    Error::msg(format!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
      e
    ))
  })?;

  if node_list.is_empty() {
    bail!(
      "The list of nodes to operate is empty. \
       Nothing to do. Exit",
    );
  }

  node_list.sort();
  node_list.dedup();

  let node_details_list_rslt = csm_rs::node::utils::get_node_details(
    &shasta_token,
    shasta_base_url,
    shasta_root_cert,
    node_list.to_vec(),
  )
  .await;

  let mut node_details_list = match node_details_list_rslt {
    Err(e) => {
      bail!("{e}");
    }
    Ok(node_details_list) => node_details_list,
  };

  node_details_list.retain(|node_details| {
    if let Some(status) = status {
      node_details.power_status.eq_ignore_ascii_case(status)
        || node_details
          .configuration_status
          .eq_ignore_ascii_case(status)
    } else {
      true
    }
  });

  node_details_list.sort_by(|a, b| a.xname.cmp(&b.xname));

  if status_summary {
    let status_output = if node_details_list.iter().any(|node_details| {
      node_details
        .configuration_status
        .eq_ignore_ascii_case("failed")
    }) {
      "FAILED"
    } else if node_details_list
      .iter()
      .any(|node_detail| node_detail.power_status.eq_ignore_ascii_case("OFF"))
    {
      "OFF"
    } else if node_details_list
      .iter()
      .any(|node_details| node_details.power_status.eq_ignore_ascii_case("on"))
    {
      "ON"
    } else if node_details_list.iter().any(|node_details| {
      node_details.power_status.eq_ignore_ascii_case("standby")
    }) {
      "STANDBY"
    } else if node_details_list.iter().any(|node_details| {
      !node_details
        .configuration_status
        .eq_ignore_ascii_case("configured")
    }) {
      "UNCONFIGURED"
    } else {
      "OK"
    };

    println!("{}", status_output);
  } else if nids_only {
    let node_nid_list = node_details_list
      .iter()
      .map(|node_details| node_details.nid.clone())
      .collect::<Vec<String>>();

    if output_opt.is_some_and(|v| v == "json") {
      let json = serde_json::to_string(&node_nid_list)
        .context("Failed to serialize node NID list")?;
      println!("{}", json);
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else if false {
    let node_xname_list = node_details_list
      .iter()
      .map(|node_details| node_details.xname.clone())
      .collect::<Vec<String>>();

    if output_opt.is_some_and(|v| v == "json") {
      let json = serde_json::to_string(&node_xname_list)
        .context("Failed to serialize node xname list")?;
      println!("{}", json);
    } else {
      println!("{}", node_xname_list.join(","));
    }
  } else {
    match output_opt.map(String::as_str) {
      Some("json") => {
        let json = serde_json::to_string_pretty(&node_details_list)
          .context("Failed to serialize node details list")?;
        println!("{}", json);
      }
      Some("summary") => {
        common::node_ops::print_summary(node_details_list);
      }
      Some("table-wide") => {
        common::node_ops::print_table_wide(node_details_list);
      }
      Some("table") => {
        common::node_ops::print_table(node_details_list);
      }
      _ => {
        bail!(
          "ERROR - output value not recognized \
           or missing. Exit",
        );
      }
    }
  }

  Ok(())
}
