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
pub struct Configuration {
    layers: Vec<Layer>
}

impl Layer {
    pub fn new(clone_url: String, commit: String, name: String, playbook: String) -> Self {
        Layer {
            clone_url,
            commit,
            name,
            playbook
        }
    }
}

impl Configuration {
    pub fn new() -> Self {
        Configuration {
            layers: vec![]
        }
    }
}

pub fn add_layer(layer: Layer, mut configuration: Configuration) -> Configuration {
    configuration.layers.push(layer);
    configuration
}

pub mod http_client {

    use super::Configuration;
    use serde_json::Value;

    pub async fn put(shasta_token: &str, shasta_base_url: &str, configuration: Configuration, configuration_name: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // rest client update cfs configurations
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
            
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

    pub async fn get(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, configuration_name: &Option<String>, limit_number: &Option<u8>) -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {

        let mut cluster_cfs_configs: Vec<Value> = Vec::new();
    
        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // rest client get cfs configurations
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
    
        let resp = client
            .get(format!("{}{}", shasta_base_url, "/cfs/v2/configurations"))
            .bearer_auth(shasta_token)
            .send()
            .await?
            .text()
            .await?;
    
        let json_response: Value = serde_json::from_str(&resp)?;
    
        if cluster_name.is_some() {
            for cfs_configuration in json_response.as_array().unwrap() {
                if cfs_configuration
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap()) //TODO: investigate why I ned to use this ugly 'as_ref'
                {
                    cluster_cfs_configs.push(cfs_configuration.clone());
                }
            }
        } else if configuration_name.is_some() {
            for cfs_configuration in json_response.as_array().unwrap() {
                if cfs_configuration
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq(configuration_name.as_ref().unwrap()) // TODO: investigate why I ned to use this ugly 'as_ref'
                {
                    cluster_cfs_configs.push(cfs_configuration.clone());
                }
            }
        } else { // Returning all results
            cluster_cfs_configs = json_response.as_array().unwrap().to_vec();

            cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].to_string().cmp(&b["lastUpdated"].to_string()));
        }
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            cluster_cfs_configs = json_response.as_array().unwrap().to_vec();
    
            cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].to_string().cmp(&b["lastUpdated"].to_string()));
    
            // cfs_utils::print_cfs_configurations(&cfs_configurations);
            
            // cluster_cfs_configs.truncate(limit_number.unwrap().into());
            cluster_cfs_configs = cluster_cfs_configs[cluster_cfs_configs.len().saturating_sub(limit_number.unwrap().into())..].to_vec();

            // cluster_cfs_configs = vec![cluster_cfs_configs.last().unwrap().clone()]; // vec! macro for vector initialization!!! https://doc.rust-lang.org/std/vec/struct.Vec.html
        }
    
        Ok(cluster_cfs_configs)
    }
}