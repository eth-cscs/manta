use crate::common::authentication::get_api_token;
use crate::common::authorization::get_groups_names_available;
use anyhow::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  common::node_ops, manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cli_get_cluster: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&String>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;
  let hsm_group_name_arg_opt: Option<&String> =
    cli_get_cluster.get_one("HSM_GROUP_NAME");
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  let status: Option<&String> = cli_get_cluster.get_one("status");
  let nids_only = cli_get_cluster.get_flag("nids-only-one-line");
  let xnames_only = cli_get_cluster.get_flag("xnames-only-one-line");
  let output_opt: Option<&String> = cli_get_cluster.get_one("output");
  let summary_status = cli_get_cluster.get_flag("summary-status");

  // Take all nodes for all hsm_groups found and put them in a Vec
  let mut hsm_groups_node_list: Vec<String> = backend
    .get_member_vec_from_group_name_vec(&shasta_token, &target_hsm_group_vec)
    .await
    .unwrap();

  hsm_groups_node_list.sort();

  let node_details_list_rslt = csm_rs::node::utils::get_node_details(
    &shasta_token,
    shasta_base_url,
    shasta_root_cert,
    hsm_groups_node_list,
  )
  .await;

  let mut node_details_list = node_details_list_rslt?;

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

  if summary_status {
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

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
      println!("{}", serde_json::to_string(&node_nid_list).unwrap());
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else if xnames_only {
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
    node_ops::print_summary(node_details_list);
  } else if output_opt.is_some() && output_opt.unwrap().eq("table-wide") {
    node_ops::print_table_wide(node_details_list);
  } else if output_opt.is_some() && output_opt.unwrap().eq("table") {
    node_ops::print_table(node_details_list);
  } else {
    return Err(Error::msg(
      "ERROR - output value not recognized or missing. Exit",
    ));
  }

  Ok(())
}
