use mesa::hsm;

use crate::common::node_ops;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_name_vec: &[String],
    silent: bool,
    silent_xname: bool,
    output_opt: Option<&String>,
    status: bool,
) {
    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> =
        hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_name_vec.to_vec(),
        )
        .await;

    hsm_groups_node_list.sort();

    let node_details_list = mesa::node::utils::get_node_details(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_groups_node_list,
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
