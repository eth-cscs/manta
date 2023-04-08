use std::collections::HashSet;

use comfy_table::Table;
use regex::Regex;
use serde_json::Value;

/// Checks nodes in ansible-limit belongs to list of nodes from multiple hsm groups
/// Returns (Vec<String>, vec<String>) being left value the list of nodes from ansible limit nodes in hsm groups and right value list of nodes from ansible limit not in hsm groups
// TODO: improve by using HashSet::diferent to get excluded and HashSet::intersection to get "included"
#[deprecated(note = "Use crate::common::node_ops::validate_xnames instead")]
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
        "XNAME",
        "NID",
        "Status",
        "Image ID (Boot param)",
        "CFS configuration (BOA)",
    ]);

    for node_status in nodes_status {
        table.add_row(node_status);
    }

    println!("{table}");
}

pub fn nodes_to_string_format_one_line(nodes: Option<&Vec<Value>>) -> String {
    if nodes.is_some() {
        nodes_to_string_format_discrete_columns(nodes, nodes.unwrap().len() + 1)
    } else {
        "".to_string()
    }
}

pub fn nodes_to_string_format_discrete_columns(nodes: Option<&Vec<Value>>, num_columns: usize) -> String {
    let mut members: String = String::new();

    if nodes.is_some() && !nodes.unwrap().is_empty() {
        members = nodes.unwrap()[0].as_str().unwrap().to_string(); // take first element

        for (i, _) in nodes.unwrap().iter().enumerate().skip(1) { // iterate for the rest of the list
            if i % num_columns == 0 {
                // breaking the cell content into multiple lines (only 2 xnames per line)

                members.push_str(",\n");
            } else {
                members.push(',');
            }

            members.push_str(nodes.unwrap()[i].as_str().unwrap());
        }
    }

    members
}

/// Validates a list of xnames.
/// Checks xnames strings are valid
/// If hsm_group_name if provided, then checks all xnames belongs to that hsm_group
pub async fn validate_xnames(
    shasta_token: &str,
    shasta_base_url: &str,
    xnames: &[&str],
    hsm_group_name: Option<&String>,
) -> bool {
    let hsm_group_members: Vec<_> = if hsm_group_name.is_some() {
        crate::shasta::hsm::http_client::get_hsm_group(
            shasta_token,
            shasta_base_url,
            hsm_group_name.unwrap(),
        )
        .await
        .unwrap()["members"]["ids"]
            .as_array()
            .unwrap()
            .to_vec()
            .iter()
            .map(|xname| xname.as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    /* println!("hsm_group_members:\n{:#?}", hsm_group_members);
    println!("xnames:\n{:#?}", xnames); */

    let xname_re = Regex::new(r"^x\d{4}c\ds\db\dn\d$").unwrap();

    if xnames.iter().any(|xname| {
        !xname_re.is_match(xname)
            || (!hsm_group_members.is_empty() && !hsm_group_members.contains(&xname.to_string()))
    }) {
        return false;
    }

    /* for xname in xnames {
        if !xname_re.is_match(xname) {
            println!("xname {} not a valid format", xname);
        }

        if !hsm_group_members.contains(&xname.to_string()) {
            println!("xname {} not a member of {:?}", xname, hsm_group_members)
        }
    } */

    true
}
