use std::collections::HashSet;

use comfy_table::Table;

/// Checks nodes in ansible-limit belongs to list of nodes from multiple hsm groups
/// Returns (Vec<String>, vec<String>) being left value the list of nodes from ansible limit nodes in hsm groups and right value list of nodes from ansible limit not in hsm groups
// TODO: improve by using HashSet::diferent to get excluded and HashSet::intersection to get "included"
pub fn check_hsm_group_and_ansible_limit(
    hsm_groups_nodes: &HashSet<String>,
    ansible_limit_nodes: HashSet<String>,
) -> (HashSet<String>, HashSet<String>) {
    (
        ansible_limit_nodes
            .intersection(hsm_groups_nodes)
            .cloned()
            .collect(),
        ansible_limit_nodes
            .difference(hsm_groups_nodes)
            .cloned()
            .collect(),
    )
}

pub fn print_table(nodes_status: Vec<Vec<String>>) {
    let mut table = Table::new();

    table.set_header(vec![
        "Node",
        "Status",
        "Image ID (Boot param)",
        "Image updated?",
    ]);

    for node_status in nodes_status {
        table.add_row(node_status);
    }

    println!("{table}");
}
