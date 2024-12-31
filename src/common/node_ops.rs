use std::collections::HashMap;

use backend_dispatcher::contracts::BackendTrait;
use comfy_table::{Cell, Table};
use hostlist_parser::parse;
use mesa::{bss::r#struct::BootParameters, hsm, node::r#struct::NodeDetails};
use regex::Regex;

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    cli::commands::config_show::get_hsm_name_available_from_jwt_or_all,
};

/// Get list of xnames user has access to based on input regex.
/// This method will:
/// 1) Break down all regex in user input
/// 2) Fetch all HSM groups user has access to
/// 3) For each HSM group, get the list of xnames and filter the ones that matches the regex
pub async fn get_curated_hsm_group_from_hostregex(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_requested_regex: &str,
) -> HashMap<String, Vec<String>> {
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

    // Get list of regex
    let regex_vec: Vec<Regex> = xname_requested_regex
        .split(",")
        .map(|regex_str| Regex::new(regex_str.trim()).expect("ERROR - regex not valid"))
        .collect();

    let hsm_name_available_vec =
        get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await;

    // Get HSM group user has access to
    let hsm_group_available_map = hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_vec(
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

    // Filter hsm group members
    for (hsm_name, xnames) in hsm_group_available_map {
        for xname in xnames {
            for regex in &regex_vec {
                if regex.is_match(&xname) {
                    hsm_group_summary
                        .entry(hsm_name.clone())
                        .and_modify(|member_vec| member_vec.push(xname.clone()))
                        .or_insert(vec![xname.clone()]);
                }
            }
        }
    }

    hsm_group_summary
}

/// Returns a HashMap with keys HSM group names the user has access to and values a curated list of memembers that matches
/// hostlist
// FIXME: merge this function with 'get_curated_hsm_group_from_hostlist' and remove the following
// input parameters 'shasta_base_url' and 'shasta_root_cert' since should be included in 'backend'
pub async fn get_curated_hsm_group_from_hostlist_backend(
    backend: &StaticBackendDispatcher,
    auth_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_requested_hostlist: &str,
) -> HashMap<String, Vec<String>> {
    // Create a summary of HSM groups and the list of members filtered by the list of nodes the
    // user is targeting
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

    let xname_requested_vec_rslt = parse(xname_requested_hostlist);

    let xname_requested_vec = match xname_requested_vec_rslt {
        Ok(xname_requested_vec) => xname_requested_vec,
        Err(e) => {
            println!(
                "Could not parse list of nodes as a hostlist. Reason:\n{}Exit",
                e
            );
            std::process::exit(1);
        }
    };

    log::info!("hostlist: {}", xname_requested_hostlist);
    log::info!("hostlist expanded: {:?}", xname_requested_vec);

    /* // Get final list of xnames to operate on
    // Get list of HSM groups available
    // NOTE: HSM available are the ones the user has access to
    // let hsm_group_name_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;

    // Get all HSM groups in the system
    // FIXME: client should not fetch all info in backend. Create a method in backend to do provide
    // information already filtered to the client:
    // hsm::groups::utils::get_hsm_group_available_vec(shasta_token, shasta_base_url,
    // shasta_root_cert) -> Vec<HsmGroup> to get the list of HSM available to the user and return
    // a Vec of HsmGroups the user has access to
    let hsm_group_vec_all =
        hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .expect("Error - fetching HSM groups"); */

    let hsm_name_available_vec = backend.get_hsm_name_available(auth_token).await.unwrap();

    // Get HSM group user has access to
    let hsm_group_available_map = backend
        .get_hsm_map_and_filter_by_hsm_name_vec(
            auth_token,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

    // Filter hsm group members
    for (hsm_name, hsm_members) in hsm_group_available_map {
        let xname_filtered: Vec<String> = hsm_members
            .iter()
            .filter(|&xname| xname_requested_vec.contains(&xname))
            .cloned()
            .collect();
        if !xname_filtered.is_empty() {
            hsm_group_summary.insert(hsm_name, xname_filtered);
        }
    }

    hsm_group_summary
}

/// Returns a HashMap with keys HSM group names the user has access to and values a curated list of memembers that matches
/// hostlist
pub async fn get_curated_hsm_group_from_hostlist(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname_requested_hostlist: &str,
) -> HashMap<String, Vec<String>> {
    // Create a summary of HSM groups and the list of members filtered by the list of nodes the
    // user is targeting
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

    let xname_requested_vec_rslt = parse(xname_requested_hostlist);

    let xname_requested_vec = match xname_requested_vec_rslt {
        Ok(xname_requested_vec) => xname_requested_vec,
        Err(e) => {
            println!(
                "Could not parse list of nodes as a hostlist. Reason:\n{}Exit",
                e
            );
            std::process::exit(1);
        }
    };

    log::info!("hostlist: {}", xname_requested_hostlist);
    log::info!("hostlist expanded: {:?}", xname_requested_vec);

    /* // Get final list of xnames to operate on
    // Get list of HSM groups available
    // NOTE: HSM available are the ones the user has access to
    // let hsm_group_name_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;

    // Get all HSM groups in the system
    // FIXME: client should not fetch all info in backend. Create a method in backend to do provide
    // information already filtered to the client:
    // hsm::groups::utils::get_hsm_group_available_vec(shasta_token, shasta_base_url,
    // shasta_root_cert) -> Vec<HsmGroup> to get the list of HSM available to the user and return
    // a Vec of HsmGroups the user has access to
    let hsm_group_vec_all =
        hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .expect("Error - fetching HSM groups"); */

    let hsm_name_available_vec =
        get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await;

    // Get HSM group user has access to
    let hsm_group_available_map = hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_vec(
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

    // Filter hsm group members
    for (hsm_name, hsm_members) in hsm_group_available_map {
        let xname_filtered: Vec<String> = hsm_members
            .iter()
            .filter(|&xname| xname_requested_vec.contains(&xname))
            .cloned()
            .collect();
        if !xname_filtered.is_empty() {
            hsm_group_summary.insert(hsm_name, xname_filtered);
        }
    }

    hsm_group_summary
}

pub fn print_table(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "HSM",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        "Image Configuration",
        "Image ID",
    ]);

    for node_status in nodes_status {
        let mut node_vec: Vec<String> = node_status
            .hsm
            .split(",")
            .map(|xname_str| xname_str.trim().to_string())
            .collect();
        node_vec.sort();

        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(nodes_to_string_format_discrete_columns(Some(&node_vec), 1)),
            Cell::new(node_status.power_status),
            Cell::new(node_status.desired_configuration),
            Cell::new(node_status.configuration_status),
            Cell::new(node_status.enabled),
            Cell::new(node_status.error_count),
            Cell::new(node_status.boot_configuration),
            Cell::new(node_status.boot_image_id),
        ]);
    }

    println!("{table}");
}

pub fn print_table_wide(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "HSM",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        "Image Configuration",
        "Image ID",
        "Kernel Params",
    ]);

    for node_status in nodes_status {
        let kernel_params_vec: Vec<&str> = node_status.kernel_params.split_whitespace().collect();
        let cell_max_width = kernel_params_vec
            .iter()
            .map(|value| value.len())
            .max()
            .unwrap_or(0);

        let mut kernel_params_string: String = kernel_params_vec[0].to_string();
        let mut cell_width = kernel_params_string.len();

        for kernel_param in kernel_params_vec.iter().skip(1) {
            cell_width += kernel_param.len();

            if cell_width + kernel_param.len() >= cell_max_width {
                kernel_params_string.push_str("\n");
                cell_width = 0;
            } else {
                kernel_params_string.push_str(" ");
            }

            kernel_params_string.push_str(kernel_param);
        }

        let mut node_vec: Vec<String> = node_status
            .hsm
            .split(",")
            .map(|xname_str| xname_str.trim().to_string())
            .collect();
        node_vec.sort();

        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(nodes_to_string_format_discrete_columns(Some(&node_vec), 1)),
            Cell::new(node_status.power_status),
            Cell::new(node_status.desired_configuration),
            Cell::new(node_status.configuration_status),
            Cell::new(node_status.enabled),
            Cell::new(node_status.error_count),
            Cell::new(node_status.boot_configuration),
            Cell::new(node_status.boot_image_id),
            Cell::new(kernel_params_string),
        ]);
    }

    println!("{table}");
}

pub fn print_summary(node_details_list: Vec<NodeDetails>) {
    let mut power_status_counters: HashMap<String, usize> = HashMap::new();
    let mut boot_configuration_counters: HashMap<String, usize> = HashMap::new();
    let mut runtime_configuration_counters: HashMap<String, usize> = HashMap::new();
    let mut boot_image_counters: HashMap<String, usize> = HashMap::new();

    for node in node_details_list {
        power_status_counters
            .entry(node.power_status)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        boot_configuration_counters
            .entry(node.boot_configuration)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        runtime_configuration_counters
            .entry(node.desired_configuration)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        boot_image_counters
            .entry(node.boot_image_id)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);
    }

    let mut table = Table::new();

    table.set_header(vec!["Power status", "Num nodes"]);

    for power_status in ["FAILED", "ON", "OFF", "READY", "STANDBY", "UNCONFIGURED"] {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(power_status),
                Cell::new(power_status_counters.get(power_status).unwrap_or(&0))
                    .set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Boot configuration name", "Num nodes"]);

    for (config_name, counter) in boot_configuration_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(config_name),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Boot image id", "Num nodes"]);

    for (image_id, counter) in boot_image_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(image_id),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Runtime configuration name", "Num nodes"]);

    for (config_name, counter) in runtime_configuration_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(config_name),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");
}

pub fn nodes_to_string_format_one_line(nodes: Option<&Vec<String>>) -> String {
    if let Some(nodes_content) = nodes {
        nodes_to_string_format_discrete_columns(nodes, nodes_content.len() + 1)
    } else {
        "".to_string()
    }
}

pub fn nodes_to_string_format_discrete_columns(
    nodes: Option<&Vec<String>>,
    num_columns: usize,
) -> String {
    let mut members: String;

    match nodes {
        Some(nodes) if !nodes.is_empty() => {
            members = nodes[0].clone(); // take first element

            for (i, _) in nodes.iter().enumerate().skip(1) {
                // iterate for the rest of the list
                if i % num_columns == 0 {
                    // breaking the cell content into multiple lines (only 2 xnames per line)

                    members.push_str(",\n");
                } else {
                    members.push(',');
                }

                members.push_str(&nodes[i]);
            }
        }
        _ => members = "".to_string(),
    }

    members
}

/// Given a list of boot params, this function returns the list of hosts booting an image_id
pub fn get_node_vec_booting_image(
    image_id: &str,
    boot_param_vec: &[BootParameters],
) -> Vec<String> {
    let mut node_booting_image_vec = boot_param_vec
        .iter()
        .cloned()
        .filter(|boot_param| boot_param.get_boot_image().eq(image_id))
        .flat_map(|boot_param| boot_param.hosts)
        .collect::<Vec<_>>();

    node_booting_image_vec.sort();

    node_booting_image_vec
}
