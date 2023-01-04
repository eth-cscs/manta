use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    #[serde(rename = "cloneUrl")]
    clone_url: String,
    commit: String,
    name: String,
    playbook: String
}

#[derive(Debug, Serialize)] // TODO: investigate why serde can Deserialize dynamically syzed structs `Vec<Layer>`
pub struct CfsConfiguration {
    layers: Vec<Layer>
}

impl Layer {
    pub fn new(clone_url: String, commit: String, name: String, playbook: String) -> Self {
        Self {
            clone_url,
            commit,
            name,
            playbook
        }
    }
}

impl CfsConfiguration {
    pub fn new() -> Self {
        Self {
            layers: vec![]
        }
    }
}

pub fn add_layer(layer: Layer, mut configuration: CfsConfiguration) -> CfsConfiguration {
    configuration.layers.push(layer);
    configuration
}

pub mod http_client {

    use std::error::Error;

    use super::CfsConfiguration;
    use serde_json::Value;

    pub async fn put(shasta_token: &str, shasta_base_url: &str, configuration: CfsConfiguration, configuration_name: &str) -> Result<Value, Box<dyn Error>> {

        let client;

        let client_builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true);
    
        // Build client
        if std::env::var("SOCKS5").is_ok() {
            
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;
    
            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }
            
        let resp = client
            .put(format!("{}{}{}", shasta_base_url, "/cfs/v2/configurations/", configuration_name))
            .json(&configuration)
            .bearer_auth(shasta_token)
            .send()
            .await?;
        
        if resp.status().is_success() {
            Ok(serde_json::from_str(&resp.text().await?)?)
        } else {
            Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    }

    pub async fn get(shasta_token: &str, shasta_base_url: &str, hsm_group_name: Option<&String>, configuration_name: Option<&String>, limit_number: Option<&u8>) -> Result<Vec<Value>, Box<dyn Error>> {

        let mut cluster_cfs_configs: Vec<Value> = Vec::new();

        let client;

        let client_builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(true);
    
        // Build client
        if std::env::var("SOCKS5").is_ok() {
            
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;
    
            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }
    
        let resp = client
            .get(format!("{}{}", shasta_base_url, "/cfs/v2/configurations"))
            .bearer_auth(shasta_token)
            .send()
            .await?;
    
        let json_response: Value;

        if resp.status().is_success() {
            json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }

        cluster_cfs_configs = json_response.as_array().unwrap().clone();
        // cluster_cfs_configs = Vec::new();
    
        if hsm_group_name.is_some() {

            cluster_cfs_configs = cluster_cfs_configs
                .into_iter()
                .filter(|cfs_configuration| {
                    cfs_configuration["name"]
                    .as_str()
                    .unwrap()
                    .contains(hsm_group_name.unwrap())
                })
                .collect();

            // for cfs_configuration in json_response.as_array().unwrap() {
            //     if cfs_configuration["name"]
            //         .as_str()
            //         .unwrap()
            //         .contains(hsm_group_name.unwrap())
            //     {
            //         cluster_cfs_configs.push(cfs_configuration.clone());
            //     }

            //     // cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].as_str().unwrap().cmp(b["lastUpdated"].as_str().unwrap()));
            // }

        }
        
        if configuration_name.is_some() {

            cluster_cfs_configs = cluster_cfs_configs
                .into_iter()
                .filter(|cfs_configuration| {
                    cfs_configuration["name"]
                    .as_str()
                    .unwrap()
                    .eq(configuration_name.unwrap())
                })
                .collect();

            // for cfs_configuration in json_response.as_array().unwrap() {
            //     if cfs_configuration["name"]
            //         .as_str()
            //         .unwrap()
            //         .eq(configuration_name.unwrap())
            //     {
            //         cluster_cfs_configs.push(cfs_configuration.clone());
            //     }
            // }

        } 
        // else { // Returning all results
        //     cluster_cfs_configs = json_response.as_array().unwrap().clone();
        // }

        cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].as_str().unwrap().cmp(b["lastUpdated"].as_str().unwrap()));
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            cluster_cfs_configs[cluster_cfs_configs.len().saturating_sub(*limit_number.unwrap() as usize)..].to_vec();

        }
    
        Ok(cluster_cfs_configs)
    }
}

pub mod utils {
    
    use comfy_table::Table;
    use serde_json::Value;


    pub fn print_table(cfs_configurations: Vec<Value>) {
        
        let mut table = Table::new();

        table.set_header(vec!["Name", "Last updated", "Layers"]);
    
        for cfs_configuration in cfs_configurations {

            let mut layers: String = String::new();

            if cfs_configuration["layers"].as_array().is_some() {

                let layers_json = cfs_configuration["layers"].as_array().unwrap();

                layers = format!("COMMIT: {} NAME: {}", layers_json[0]["commit"], layers_json[0]["name"]);
                
                for i in 1..layers_json.len() {

                    let layer = &layers_json[i];
                    layers = format!("{}\nCOMMIT: {} NAME: {}", layers, layer["commit"], layer["name"]);
                }
            }

            table.add_row(vec![
                cfs_configuration["name"].as_str().unwrap(),
                cfs_configuration["lastUpdated"].as_str().unwrap(),
                &layers
            ]);
        }
    
        println!("{table}");
    }
}