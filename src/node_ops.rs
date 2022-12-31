use std::collections::HashSet;

/// Checks nodes in ansible-limit belongs to list of nodes from multiple hsm groups
/// Returns (Vec<String>, vec<String>) being left value the list of nodes from ansible limit nodes in hsm groups and right value list of nodes from ansible limit not in hsm groups
pub fn check_hsm_group_and_ansible_limit<'a>(hsm_groups_nodes: HashSet<String>, ansible_limit_nodes: Vec<String>) -> (HashSet<String>, HashSet<String>) {

    let mut included = HashSet::new();
    let mut excluded = HashSet::new();

    for ansible_limit_node in ansible_limit_nodes {
        if hsm_groups_nodes.contains(&ansible_limit_node) {
            included.insert(ansible_limit_node);
        } else {
            excluded.insert(ansible_limit_node);
        }
    }

    (included, excluded)
}