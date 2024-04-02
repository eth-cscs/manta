use comfy_table::Table;
use mesa::cfs::configuration::mesa::r#struct::{
    cfs_configuration::ConfigurationDetails,
    cfs_configuration_response::v2::CfsConfigurationResponse,
};

pub fn print_table_struct(cfs_configurations: &Vec<CfsConfigurationResponse>) {
    let mut table = Table::new();

    table.set_header(vec!["Config Name", "Last updated", "Layers"]);

    for cfs_configuration in cfs_configurations {
        let mut layers: String = String::new();

        if !cfs_configuration.layers.is_empty() {
            let layers_json = &cfs_configuration.layers;

            layers = format!(
                "Name:     {}\nPlaybook: {}\nCommit:   {}",
                layers_json[0].name,
                layers_json[0].playbook,
                layers_json[0]
                    .commit
                    .as_ref()
                    .unwrap_or(&"Not defined".to_string()),
            );

            for layer in layers_json.iter().skip(1) {
                layers = format!(
                    "{}\n\nName:     {}\nPlaybook: {}\nCommit:   {}",
                    layers,
                    layer.name,
                    layer.playbook,
                    layer.commit.as_ref().unwrap_or(&"Not defined".to_string()),
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

pub fn print_table_details_struct(cfs_configuration: ConfigurationDetails) {
    let mut table = Table::new();

    table.set_header(vec!["Configuration Name", "Last updated", "Layers"]);

    let mut layers: String = String::new();

    for layer in cfs_configuration.config_layers {
        layers = format!(
            "{}\n\nName: {}\nBranch: {}\nTag: {}\nCommit date: {}\nAuthor: {}\nSHA: {}\nPlaybook: {}",
            layers,
            layer.name,
            layer.branch,
            /* if let true = layer.most_recent_commit {
                "(Up to date)"
            } else {
                "(Outdated)"
            }, */
            layer.tag,
            layer.commit_date,
            layer.author,
            layer.commit_id,
            layer.playbook
        );
    }

    layers = layers.trim_start_matches("\n\n").to_string();

    /* if !cfs_configuration.config_layers.is_empty() {
        layers = format!(
            "Name: {}\nCommit date: {}\nAuthor: {}\nBranch: {} Most recent?: {}\nTag: {}\nCommit ID: {}",
            cfs_configuration.config_layers[0].name,
            cfs_configuration.config_layers[0].commit_date,
            cfs_configuration.config_layers[0].author,
            cfs_configuration.config_layers[0]
                .branch
                .as_ref()
                .unwrap_or(&"Not deinfed".to_string()),
            cfs_configuration.config_layers[0].most_recent_commit.as_ref().unwrap_or(&"Not defined".to_string()).to_string(),
            cfs_configuration.config_layers[0].tag.as_ref().unwrap_or(&"Not defined".to_string()),
            cfs_configuration.config_layers[0].commit_id,
        );

        for i in 1..cfs_configuration.config_layers.len() {
            let layer = &cfs_configuration.config_layers[i];
            layers = format!(
                "{}\n\nName: {}\nCommit date: {}\nAuthor: {}\nBranch: {}\nCommit ID: {}",
                layers,
                layer.name,
                layer.commit_date,
                layer.author,
                layer.branch.as_ref().unwrap_or(&"Not defined".to_string()),
                layer.commit_id,
            );
        }
    } */

    table.add_row(vec![
        cfs_configuration.name,
        cfs_configuration.last_updated,
        layers,
    ]);

    println!("{table}");
}
