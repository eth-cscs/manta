use serde_json::Value;

pub fn nodes_to_string_format_mid(nodes: &Vec<Value>) -> String {
    nodes_to_string(nodes, 10)
}

pub fn nodes_to_string_format_unlimited(nodes: &Vec<Value>) -> String {
    nodes_to_string(nodes, nodes.len() + 1)
}

pub fn nodes_to_string(nodes: &Vec<Value>, limit: usize) -> String {
    let mut members: String = String::new();

    if !nodes.is_empty() {
        members = nodes[0].as_str().unwrap().to_string();

        for (i, _) in nodes.iter().enumerate().skip(1) {
            if i % limit == 0 {
                // breaking the cell content into multiple lines (only 2 xnames per line)

                members.push_str(",\n");
            } else {
                members.push_str(",");
            }

            members.push_str(nodes[i].as_str().unwrap());
        }
    }

    members
}
