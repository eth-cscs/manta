use std::collections::HashMap;

use crate::common::{self, node_ops};

use super::{
    config_show::get_hsm_name_without_system_wide_available_from_jwt_or_all,
    power_on_nodes::is_user_input_nids,
};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hosts_string: &str,
    is_siblings: bool,
    silent_nids: bool,
    silent_xname: bool,
    output_opt: Option<&String>,
    status: bool,
    is_regex: bool,
) {
    let hsm_name_available_vec = get_hsm_name_without_system_wide_available_from_jwt_or_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await;

    // Get HSM group user has access to
    let hsm_group_available_map =
        mesa::hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_without_system_wide_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

    // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
    let mut node_list: Vec<String> = if is_user_input_nids(hosts_string) {
        log::debug!("User input seems to be NID");
        let xname_requested_vec = common::node_ops::nid_to_xname(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            hosts_string,
            is_regex,
        )
        .await
        .expect("Could not convert NID to XNAME");

        let xname_shelf_vec: Vec<String> = if is_siblings {
            xname_requested_vec
                .iter()
                .map(|xname| xname[0..10].to_string())
                .collect()
        } else {
            xname_requested_vec
        };

        // Filter hsm group members
        let xname_available_iter = hsm_group_available_map.values().flatten().cloned();

        xname_available_iter
            .filter(|xname| {
                xname_shelf_vec
                    .iter()
                    .any(|xname_shelf| xname.starts_with(xname_shelf))
            })
            .collect()
    } else {
        log::debug!("User input seems to be XNAME");
        let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
            common::node_ops::get_curated_hsm_group_from_xname_regex(
                &hosts_string,
                hsm_group_available_map,
                is_siblings,
            )
            .await
        } else {
            // Get HashMap with HSM groups and members curated for this request.
            // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
            // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
            // hostlist have been removed
            common::node_ops::get_curated_hsm_group_from_xname_hostlist(
                &hosts_string,
                hsm_group_available_map,
                is_siblings,
            )
            .await
        };

        hsm_group_summary.values().cloned().flatten().collect()
    };

    if node_list.is_empty() {
        eprintln!("The list of nodes to operate is empty. Nothing to do. Exit");
        std::process::exit(0);
    }

    node_list.sort();
    node_list.dedup();

    let node_details_list = mesa::node::utils::get_node_details(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        node_list.to_vec(),
    )
    .await;

    if status {
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
        } else if node_details_list
            .iter()
            .any(|node_details| node_details.power_status.eq_ignore_ascii_case("standby"))
        {
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
    } else if silent_nids {
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
        node_ops::print_summary(node_details_list);
    } else if output_opt.is_some() && output_opt.unwrap().eq("table-wide") {
        node_ops::print_table_wide(node_details_list);
    } else if output_opt.is_some() && output_opt.unwrap().eq("table") {
        node_ops::print_table(node_details_list);
    } else {
        eprintln!("ERROR - output value not recognized or missing. Exit");
        std::process::exit(1);
    }
}
