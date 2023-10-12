use mesa::shasta::hsm;
use termion::color;

use crate::common::node_ops;

use mesa::manta::get_nodes_status;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    silent: bool,
    silent_xname: bool,
    output_opt: Option<&String>,
) {
    let hsm_groups_resp =
        hsm::http_client::get_hsm_groups(shasta_token, shasta_base_url, hsm_group_name).await;

    let hsm_group_list = if hsm_groups_resp.is_err() {
        eprintln!(
            "No HSM group {}{}{} found!",
            color::Fg(color::Red),
            hsm_group_name.unwrap(),
            color::Fg(color::Reset)
        );
        std::process::exit(0);
    } else {
        hsm_groups_resp.unwrap()
    };

    if hsm_group_list.is_empty() {
        println!("No HSM group found");
        std::process::exit(0);
    }

    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> =
        hsm::utils::get_members_from_hsm_groups_serde_value(&hsm_group_list)
            .into_iter()
            .collect();

    hsm_groups_node_list.sort();

    let node_details_list =
        get_nodes_status::exec(shasta_token, shasta_base_url, hsm_groups_node_list).await;

    if silent {
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
    } else {
        node_ops::print_table(node_details_list);
    }
}
