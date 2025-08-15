use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hosts_expression: &str,
  status: Option<&String>,
  is_include_siblings: bool,
  silent_nid: bool,
  silent_xname: bool,
  output_opt: Option<&String>,
  status_summary: bool,
) {
  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
      std::process::exit(1);
    });

  let mut node_list = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    is_include_siblings,
    node_metadata_available_vec,
  )
  .await
  .unwrap_or_else(|e| {
    eprintln!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
      e
    );
    std::process::exit(1);
  });

  if node_list.is_empty() {
    eprintln!("The list of nodes to operate is empty. Nothing to do. Exit");
    std::process::exit(0);
  }

  node_list.sort();
  node_list.dedup();

  let node_details_list_rslt = csm_rs::node::utils::get_node_details(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    node_list.to_vec(),
  )
  .await;

  let mut node_details_list = match node_details_list_rslt {
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
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

  node_details_list.sort_by_key(|node_details| node_details.xname.clone());

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
  } else if silent_nid {
    let node_nid_list = node_details_list
      .iter()
      .map(|node_details| node_details.nid.clone())
      .collect::<Vec<String>>();

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
      println!("{}", serde_json::to_string(&node_nid_list).unwrap());
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else if silent_xname {
    let node_xname_list = node_details_list
      .iter()
      .map(|node_details| node_details.xname.clone())
      .collect::<Vec<String>>();

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
      println!("{}", serde_json::to_string(&node_xname_list).unwrap());
    } else {
      println!("{}", node_xname_list.join(","));
    }
  } else if output_opt.is_some() && output_opt.unwrap().eq("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&node_details_list).unwrap()
    );
  } else if output_opt.is_some() && output_opt.unwrap().eq("summary") {
    common::node_ops::print_summary(node_details_list);
  } else if output_opt.is_some() && output_opt.unwrap().eq("table-wide") {
    common::node_ops::print_table_wide(node_details_list);
  } else if output_opt.is_some() && output_opt.unwrap().eq("table") {
    common::node_ops::print_table(node_details_list);
  } else {
    eprintln!("ERROR - output value not recognized or missing. Exit");
    std::process::exit(1);
  }
}
