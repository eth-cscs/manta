use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, node_ops},
};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    // mut node_list: Vec<String>,
    hosts_string: &str,
    is_include_siblings: bool,
    silent_nid: bool,
    silent_xname: bool,
    output_opt: Option<&String>,
    status: bool,
    is_regex: bool,
) {
    // Convert user input to xname
    let mut node_list = common::node_ops::resolve_node_list_user_input_to_xname(
        backend,
        shasta_token,
        hosts_string,
        is_include_siblings,
        is_regex,
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
