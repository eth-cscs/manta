use mesa::shasta::hsm;

use crate::common::node_ops;

use mesa::manta::get_nodes_status;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_name_vec: &Vec<String>,
    silent: bool,
    silent_xname: bool,
    output_opt: Option<&String>,
) {
    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> = hsm::utils::get_member_vec_from_hsm_name_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_name_vec,
    )
    .await;

    hsm_groups_node_list.sort();

    let node_details_list = get_nodes_status::exec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_groups_node_list,
    )
    .await;

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
