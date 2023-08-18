use comfy_table::Table;
use serde_json::Value;

pub fn get_cfs_session_list(cfs_session: Value) -> Vec<String> {
    let mut result = Vec::new();

    result.push(cfs_session["name"].as_str().unwrap_or_default().to_string());
    result.push(
        cfs_session
            .pointer("/configuration/name")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/status/session/startTime")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/ansible/passthrough")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/ansible/verbosity")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/target/definition")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );
    result.push(
        cfs_session
            .pointer("/target/groups")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|group| group["name"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>()
            .join(",\n"),
    );

    result
}

pub fn print_table(get_cfs_session_value_list: &Vec<Value>) {
    let mut table = Table::new();

    table.set_header(vec![
        "Session Name",
        "Configuration Name",
        "Start",
        "Passthrough",
        "Verbosity",
        "Target Def",
        "Target",
    ]);

    for cfs_session_value in get_cfs_session_value_list {
        table.add_row(get_cfs_session_list(cfs_session_value.clone()));
    }

    println!("{table}");
}
