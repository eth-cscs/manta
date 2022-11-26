use serde_json::Value;

pub fn nodes_to_string(nodes: &Vec<Value>) -> String {

    let mut members: String = String::new();

    if !nodes.is_empty() {

        members = nodes[0].as_str().unwrap().to_string();

        for i in 1..nodes.len() {

            if i % 10 == 0 { // breaking the cell content into multiple lines (only 2 xnames per line)
                members = format!("{},\n", members);
            } else {
                members = format!("{},", members);
            }

            members = format!("{}{}", members, nodes[i].as_str().unwrap());    
        }
    }

    members
}