use std::collections::HashSet;

use comfy_table::Table;
use serde_json::Value;

use crate::cluster_ops::ClusterDetails;

/// Checks nodes in ansible-limit belongs to list of nodes from multiple hsm groups
/// Returns (Vec<String>, vec<String>) being left value the list of nodes from ansible limit nodes in hsm groups and right value list of nodes from ansible limit not in hsm groups
// TODO: improve by using HashSet::diferent to get excluded and HashSet::intersection to get "included"
pub fn check_hsm_group_and_ansible_limit(hsm_groups_nodes: &HashSet<String>, ansible_limit_nodes: HashSet<String>) -> (HashSet<String>, HashSet<String>) {

    (ansible_limit_nodes.intersection(hsm_groups_nodes).cloned().collect(), ansible_limit_nodes.difference(hsm_groups_nodes).cloned().collect())
}

pub fn print_table(hsm_groups: Vec<ClusterDetails>, node_status: Value) {

    let mut table = Table::new();

    table.set_header(vec!["Hsm Group", "Node", "Status"]);

    for hsm_group in hsm_groups {

        let members: Vec<&str> = hsm_group.members.iter().map(|member| member.as_str().unwrap()).collect();

        for member in members {
            table.add_row(vec![
                hsm_group.hsm_group_label.clone(),
                member.to_string(),
                if node_status.get("on").is_some() && node_status["on"].to_string().contains(member) { "ON".to_string() }
                else if node_status.get("off").is_some() &&  node_status["off"].to_string().contains(member) { "OFF".to_string() }
                else if node_status.get("disabled").is_some() && node_status["disabled"].to_string().contains(member) { "DISABLED".to_string() }
                else if node_status.get("ready").is_some() && node_status["ready"].to_string().contains(member) { "READY".to_string() }
                else if node_status.get("standby").is_some() && node_status["standby"].to_string().contains(member) { "STANDBY".to_string() }
                else { "N/A".to_string() },
            ]);
        }
    }

    println!("{table}");
}