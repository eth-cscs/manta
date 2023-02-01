use serde_json::Value;

pub fn nodes_to_string(nodes: &Vec<Value>) -> String {

    let mut members: String = String::new();

    if !nodes.is_empty() {

        members = nodes[0].as_str().unwrap().to_string();

        for (i, _) in nodes.iter().enumerate().skip(1) {

            if i % 10 == 0 { // breaking the cell content into multiple lines (only 2 xnames per line)
                
                members.push_str(",\n");
                
                //members = format!("{},\n", members);
            } else {
                
                members.push_str(",");
                
                // members = format!("{},", members);
            }

            members.push_str(nodes[i].as_str().unwrap());

            // members = format!("{}{}", members, nodes[i].as_str().unwrap());    
        }
    }

    members
}