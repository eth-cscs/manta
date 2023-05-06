use termion::color;

use crate::{common::node_ops, manta::get_nodes_status, shasta::hsm};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    silent: bool,
    silent_xname: bool,
) {
    let hsm_groups_resp =
        hsm::http_client::get_hsm_groups(shasta_token, shasta_base_url, hsm_group_name).await;

    // println!("hsm_groups_resp: {:#?}", hsm_groups_resp);

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

    // println!("hsm_group_list:\n{:#?}", hsm_group_list);

    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> =
        hsm::utils::get_members_from_hsm_groups_serde_value(&hsm_group_list)
            .into_iter()
            .collect();

    hsm_groups_node_list.sort();

    let node_details_list =
        get_nodes_status::exec(shasta_token, shasta_base_url, hsm_groups_node_list).await;

    if silent {
        println!(
            "{}",
            node_details_list
                .iter()
                .map(|node_details| node_details[1].clone())
                .collect::<Vec<String>>()
                .join(",")
        );
    } else if silent_xname {
        println!(
            "{}",
            node_details_list
                .iter()
                .map(|node_details| node_details[0].clone())
                .collect::<Vec<String>>()
                .join(",")
        );
    } else {
        // println!("node_details_list:\n{:#?}", node_details_list);

        node_ops::print_table(node_details_list);
    }
}
