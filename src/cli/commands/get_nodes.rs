use std::collections::HashMap;

use comfy_table::{Cell, Table};
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
    } else {
        node_ops::print_table(node_details_list);
    }
}
