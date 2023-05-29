use comfy_table::Table;
use serde_json::Value;
use std::fmt;

use crate::shasta;

pub struct Configuration {
    name: String,
    last_updated: String,
    config_layers: Vec<Layer>,
}

impl Configuration {
    pub fn new(name: &str, last_updated: &str, config_layers: Vec<Layer>) -> Self {
        Self {
            name: String::from(name),
            last_updated: String::from(last_updated),
            config_layers,
        }
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nConfig Details:\n - name: {}\n - last updated: {}\nLayers:",
            self.name, self.last_updated
        )?;

        for (i, config_layer) in self.config_layers.iter().enumerate() {
            write!(f, "\n Layer {}:{}", i, config_layer)?;
        }

        Ok(())
    }
}

pub struct Layer {
    pub name: String,
    pub repo_name: String,
    pub commit_id: String,
    pub author: String,
    pub commit_date: String,
}

impl Layer {
    pub fn new(
        name: &str,
        repo_name: &str,
        commit_id: &str,
        author: &str,
        commit_date: &str,
    ) -> Self {
        Self {
            name: String::from(name),
            repo_name: String::from(repo_name),
            commit_id: String::from(commit_id),
            author: String::from(author),
            commit_date: String::from(commit_date),
        }
    }
}

pub async fn get_configuration(
    shasta_token: &str,
    shasta_base_url: &str,
    configuration_name: Option<&String>,
    // contains: Option<&String>,
    most_recent: Option<bool>,
    limit: Option<&u8>,
) -> Vec<Value> {
    // let configuration_name = cli_get_configuration.get_one::<String>("name");

    /* let hsm_group_name = match hsm_group {
        // ref: https://stackoverflow.com/a/32487173/1918003
        None => cli_get_configuration.get_one::<String>("hsm-group"),
        Some(hsm_group_val) => Some(hsm_group_val),
    }; */

    // let most_recent = cli_get_configuration.get_one::<bool>("most-recent");

    let limit_number;

    if let Some(true) = most_recent {
        limit_number = Some(&1);
    } else if let Some(false) = most_recent {
        limit_number = limit;
    } else {
        limit_number = None;
    }

    // Get CFS configurations
    shasta::cfs::configuration::http_client::get(
        shasta_token,
        shasta_base_url,
        // hsm_group_name,
        configuration_name,
        limit_number,
    )
    .await
    .unwrap_or_default()
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n - name: {}\n - repo name: {}\n - commit id: {}\n - commit date: {}\n - author: {}",
            self.name, self.repo_name, self.commit_id, self.commit_date, self.author
        )
    }
}

pub fn print_table(cfs_configuration: Configuration) {
    let mut table = Table::new();

    table.set_header(vec!["Name", "Last updated", "Layers"]);

    let mut layers: String = String::new();

    if !cfs_configuration.config_layers.is_empty() {
        layers = format!(
            "COMMIT ID: {} COMMIT DATE: {} NAME: {} AUTHOR: {}",
            cfs_configuration.config_layers[0].commit_id,
            cfs_configuration.config_layers[0].commit_date,
            cfs_configuration.config_layers[0].name,
            cfs_configuration.config_layers[0].author
        );

        for i in 1..cfs_configuration.config_layers.len() {
            let layer = &cfs_configuration.config_layers[i];
            layers = format!(
                "{}\nCOMMIT ID: {} COMMIT DATE: {} NAME: {} AUTHOR: {}",
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
