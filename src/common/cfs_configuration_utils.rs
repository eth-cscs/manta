use comfy_table::Table;
use mesa::cfs::configuration::mesa::r#struct::{
    cfs_configuration::Configuration, cfs_configuration_response::CfsConfigurationResponse,
};

/* pub fn print_table_value(cfs_configuration_value_vec: &Vec<Value>) {
    let mut table = Table::new();

    table.set_header(vec!["Configuration Name", "Last updated", "Layers"]);

    for cfs_configuration_value in cfs_configuration_value_vec {
        let mut layers: Vec<String> = Vec::new();

        if cfs_configuration_value.get("layers").is_some()
            && cfs_configuration_value["layers"].is_array()
        {
            let cfs_configuration_layer_value_vec =
                cfs_configuration_value["layers"].as_array().unwrap();

            for (i, cfs_configuration_layer_value) in
                cfs_configuration_layer_value_vec.iter().enumerate()
            {
                println!(
                    "cfs_configuration_layer_value: {}",
                    cfs_configuration_layer_value
                );
                layers.push(format!(
                    "Layer {}:\n - commit id: {}\n - branch: {}n\n - name: {}\n - clone url: {}\n - playbook: {}",
                    i,
                    cfs_configuration_layer_value["commit"].as_str().unwrap(),
                    cfs_configuration_layer_value["branch"].as_str().unwrap(),
                    cfs_configuration_layer_value["name"].as_str().unwrap(),
                    cfs_configuration_layer_value["cloneUrl"].as_str().unwrap(),
                    cfs_configuration_layer_value["playbook"].as_str().unwrap(),
                ));
            }
        }

        table.add_row(vec![
            cfs_configuration_value["name"]
                .as_str()
                .unwrap()
                .to_string(),
            cfs_configuration_value["lastUpdated"]
                .as_str()
                .unwrap()
                .to_string(),
            layers.join("\n--------------------------\n").to_string(),
        ]);
    }

    println!("{table}");
} */

pub fn print_table_struct(cfs_configurations: &Vec<CfsConfigurationResponse>) {
    let mut table = Table::new();

    table.set_header(vec!["Config Name", "Last updated", "Layers"]);

    for cfs_configuration in cfs_configurations {
        println!("DEBUG - CFS configuration: {:#?}", cfs_configuration);
        let mut layers: String = String::new();

        if !cfs_configuration.layers.is_empty() {
            let layers_json = &cfs_configuration.layers;

            layers = format!(
                "COMMIT: {} NAME: {}",
                layers_json[0].commit.as_ref().unwrap(),
                layers_json[0].name
            );

            for layer in layers_json.iter().skip(1) {
                println!("DEBUG - layer: {:#?}", layer);
                layers = format!(
                    "{}\nCOMMIT: {} NAME: {}",
                    layers,
                    layer.commit.as_ref().unwrap_or(&"Not defined".to_string()),
                    layer.name
                );
            }
        }

        table.add_row(vec![
            cfs_configuration.name.clone(),
            cfs_configuration.last_updated.clone(),
            layers,
        ]);
    }

    println!("{table}");
}

pub fn print_table_details_struct(cfs_configuration: Configuration) {
    let mut table = Table::new();

    table.set_header(vec!["Configuration Name", "Last updated", "Layers"]);

    let mut layers: String = String::new();

    if !cfs_configuration.config_layers.is_empty() {
        layers = format!(
            "commit id: {} commit date: {} name: {} author: {}",
            cfs_configuration.config_layers[0].commit_id,
            cfs_configuration.config_layers[0].commit_date,
            cfs_configuration.config_layers[0].name,
            cfs_configuration.config_layers[0].author
        );

        for i in 1..cfs_configuration.config_layers.len() {
            let layer = &cfs_configuration.config_layers[i];
            layers = format!(
                "{}\ncommit id: {} commit date: {} name: {} author: {}",
                layers, layer.commit_id, layer.commit_date, layer.name, layer.author
            );
        }
    }

    table.add_row(vec![
        cfs_configuration.name,
        cfs_configuration.last_updated,
        layers,
    ]);

    println!("{table}");
}
