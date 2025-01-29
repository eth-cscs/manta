use std::collections::HashMap;

use crate::common::{self, node_ops};

use super::power_on_nodes::is_user_input_nids;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hosts_string: &str,
    silent: bool,
    silent_xname: bool,
    output_opt: Option<&String>,
    status: bool,
    is_regex: bool,
) {
    // Check if user input is 'nid' or 'xname' and convert to 'xname' if needed
    let mut node_list = if is_user_input_nids(hosts_string) {
        log::debug!("User input seems to be NID");
        common::node_ops::nid_to_xname(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            hosts_string,
            is_regex,
        )
        .await
        .expect("Could not convert NID to XNAME")
    } else {
        log::debug!("User input seems to be XNAME");
        let hsm_group_summary: HashMap<String, Vec<String>> = if is_regex {
            common::node_ops::get_curated_hsm_group_from_xname_regex(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &hosts_string,
            )
            .await
        } else {
            // Get HashMap with HSM groups and members curated for this request.
            // NOTE: the list of HSM groups are the ones the user has access to and containing nodes within
            // the hostlist input. Also, each HSM goup member list is also curated so xnames not in
            // hostlist have been removed
            common::node_ops::get_curated_hsm_group_from_xname_hostlist(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &hosts_string,
            )
            .await
        };

        hsm_group_summary.values().flatten().cloned().collect()
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
    } else if silent {
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
