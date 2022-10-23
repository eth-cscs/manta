use serde_json::Value;

// pub fn print_cfs_configurations(cfs_configurations: &Vec<Value>) {

//     let mut table = Table::new();

//     table.set_header(vec!["Name", "Last updated", "Layers"]);

//     for cfs_configuration in cfs_configurations {
//         table.add_row(vec![
//             cfs_configuration["name"].as_str().unwrap(),
//             cfs_configuration["lastUpdated"].as_str().unwrap(),
//         ]);
//         let mut layers: Vec<String> = vec![];
//         if !cfs_configuration["layers"].is_null() {
//             for layer in cfs_configuration["layers"].as_array().unwrap() {
//                 layers.push(format!(" - layer - name: {}; commit: {}", layer["name"].as_str().unwrap(), layer["commit"].as_str().unwrap()));
//             }
//         }
//     }

//     println!("{table}");
//     // println!("*** CFS CONFIGURATIONS ***");
//     // println!("================================");
//     // for cfs_configuration in cfs_configurations {
//     //     print_cfs_configuration(cfs_configuration);
//     //     println!("================================");
//     // }

// }

pub fn print_cfs_configuration(cfs_configuration: &Value) {
    println!("name: {}", cfs_configuration["name"].as_str().unwrap());
    println!("last updated: {}", cfs_configuration["lastUpdated"].as_str().unwrap());
    println!("layers: ");
    if !cfs_configuration["layers"].is_null() {
        for layer in cfs_configuration["layers"].as_array().unwrap() {
            println!(" - layer - name: {}; commit: {}", layer["name"].as_str().unwrap(), layer["commit"].as_str().unwrap());
        }
    }
}

pub fn print_cfs_sessions(cfs_sessions: &Vec<Value>) {
    println!("*** CFS SESSIONS ***");
    println!("================================");
    for cfs_session in cfs_sessions {
        print_cfs_session(cfs_session);
        println!("================================");
    }
}

pub fn print_cfs_session(cfs_session: &Value) {
    println!("name: {}", cfs_session["name"].as_str().unwrap());
    println!("configuration: {}", cfs_session["configuration"]["name"].as_str().unwrap());
    println!("target definition: {}", cfs_session["target"]["definition"].as_str().unwrap());
    let mut target_groups: String = "".to_string();
    if !cfs_session["target"]["groups"].is_null() {
        for target_group in cfs_session["target"]["groups"].as_array().unwrap() {
            target_groups = format!("{} {}", target_groups, target_group["name"].as_str().unwrap());
        }
    }
    println!("target groups name: {}", target_groups);
    println!("ansible - limit: {}", cfs_session["ansible"]["limit"].to_string());
    println!("start time: {}", cfs_session["status"]["session"]["startTime"].as_str().unwrap());
    println!("status: {}", cfs_session["status"]["session"]["status"].as_str().unwrap());
    println!("succeeded: {}", cfs_session["status"]["session"]["succeeded"].as_str().unwrap());
    println!("job: {}", cfs_session["status"]["session"]["job"].as_str().unwrap());
}