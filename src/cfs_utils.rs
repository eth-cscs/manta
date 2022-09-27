use serde_json::Value;

pub fn print_cfs_configurations(cfs_configurations: &Vec<Value>) {
    log::info!("*** CFS CONFIGURATIONS ***");
    log::info!("================================");
    for cfs_configuration in cfs_configurations {
        print_cfs_configuration(cfs_configuration);
        log::info!("================================");
    }
}

pub fn print_cfs_configuration(cfs_configuration: &Value) {
    log::info!("name: {}", cfs_configuration["name"].as_str().unwrap());
    log::info!("last updated: {}", cfs_configuration["lastUpdated"].as_str().unwrap());
    log::info!("layers: ");
    if !cfs_configuration["layers"].is_null() {
        for layer in cfs_configuration["layers"].as_array().unwrap() {
            log::info!(" - layer - name: {}; commit: {}", layer["name"].as_str().unwrap(), layer["commit"].as_str().unwrap());
        }
    }
}

pub fn print_cfs_sessions(cfs_sessions: &Vec<Value>) {
    log::info!("*** CFS SESSIONS ***");
    log::info!("================================");
    for cfs_session in cfs_sessions {
        print_cfs_session(&cfs_session);
        log::info!("================================");
    }
}

pub fn print_cfs_session(cfs_session: &Value) {
    log::info!("name: {}", cfs_session["name"].as_str().unwrap());
    log::info!("configuration: {}", cfs_session["configuration"]["name"].as_str().unwrap());
    log::info!("target definition: {}", cfs_session["target"]["definition"].as_str().unwrap());
    let mut target_groups: String = "".to_string();
    if !cfs_session["target"]["groups"].is_null() {
        for target_group in cfs_session["target"]["groups"].as_array().unwrap() {
            target_groups = format!("{} {}", target_groups, target_group["name"].as_str().unwrap());
        }
    }
    log::info!("target groups name: {}", target_groups);
    log::info!("ansible - limit: {}", cfs_session["ansible"]["limit"].to_string());
    log::info!("start time: {}", cfs_session["status"]["session"]["startTime"].as_str().unwrap());
    log::info!("status: {}", cfs_session["status"]["session"]["status"].as_str().unwrap());
    log::info!("succeeded: {}", cfs_session["status"]["session"]["succeeded"].as_str().unwrap());
    log::info!("job: {}", cfs_session["status"]["session"]["job"].as_str().unwrap());
}