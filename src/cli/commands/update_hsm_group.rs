use mesa::hsm;

use crate::cli::commands::update_node;

/// Updates boot params and desired configuration for all nodes that belongs to a HSM group
/// If boot params defined, then nodes in HSM group will be rebooted
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    boot_image_configuration_opt: Option<&String>,
    desired_configuration_opt: Option<&String>,
    hsm_group_name: &String,
) {
    // Get nodes members of HSM group
    // Get HSM group details
    let hsm_group_details_rslt = hsm::group::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(hsm_group_name),
    )
    .await;

    let hsm_group_details = if let Ok(hsm_group_details) = hsm_group_details_rslt {
        hsm_group_details
    } else {
        eprintln!("Cluster '{}' not found. Exit", hsm_group_name);
        std::process::exit(1);
    };

    // Get list of xnames in HSM group
    let nodes: Vec<&str> = hsm_group_details[0]["members"]["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node.as_str().unwrap())
        .collect();

    update_node::exec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(hsm_group_name),
        boot_image_configuration_opt,
        desired_configuration_opt,
        nodes.clone(),
    )
    .await;
}
