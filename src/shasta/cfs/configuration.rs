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
        Layer {
            clone_url,
            commit,
            name,
            playbook
        }
    }
}

impl CfsConfiguration {
    pub fn new() -> Self {
        CfsConfiguration {
            layers: vec![]
        }
    }
}

pub fn add_layer(layer: Layer, mut configuration: CfsConfiguration) -> CfsConfiguration {
    configuration.layers.push(layer);
    configuration
}

pub mod http_client {

    use super::CfsConfiguration;
    use serde_json::Value;

    pub async fn put(shasta_token: &str, shasta_base_url: &str, configuration: CfsConfiguration, configuration_name: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        // // socks5 proxy
        // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // // rest client update cfs configurations
        // let client = reqwest::Client::builder()
        //     .danger_accept_invalid_certs(true)
        //     .proxy(socks5proxy)
        //     .build()?;

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

    pub async fn get(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, configuration_name: &Option<String>, limit_number: &Option<u8>) -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {

        let mut cluster_cfs_configs: Vec<Value> = Vec::new();
    
        // // socks5 proxy
        // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // // rest client get cfs configurations
        // let client = reqwest::Client::builder()
        //     .danger_accept_invalid_certs(true)
        //     .proxy(socks5proxy)
        //     .build()?;

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
    
        // let json_response: Value = serde_json::from_str(&resp)?;

        let json_response: Value;

        if resp.status().is_success() {
            json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    
        if cluster_name.is_some() {

            for cfs_configuration in json_response.as_array().unwrap() {
                if cfs_configuration["name"]
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap()) //TODO: investigate why I need to use this ugly 'as_ref'
                {
                    cluster_cfs_configs.push(cfs_configuration.clone());
                }

                cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].as_str().unwrap().cmp(&b["lastUpdated"].as_str().unwrap()));
            }

        } else if configuration_name.is_some() {

            for cfs_configuration in json_response.as_array().unwrap() {
                if cfs_configuration["name"]
                    .as_str()
                    .unwrap()
                    .eq(configuration_name.as_ref().unwrap()) // TODO: investigate why I ned to use this ugly 'as_ref'
                {
                    cluster_cfs_configs.push(cfs_configuration.clone());
                }
            }

        } else { // Returning all results
            cluster_cfs_configs = json_response.as_array().unwrap().to_vec();

            cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].as_str().unwrap().cmp(&b["lastUpdated"].as_str().unwrap()));
        }
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            // cluster_cfs_configs = json_response.as_array().unwrap().to_vec();
    
            cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].as_str().unwrap().cmp(&b["lastUpdated"].as_str().unwrap()));
    
            // cfs_utils::print_cfs_configurations(&cfs_configurations);
            
            // cluster_cfs_configs.truncate(limit_number.unwrap().into());
            cluster_cfs_configs = cluster_cfs_configs[cluster_cfs_configs.len().saturating_sub(limit_number.unwrap().into())..].to_vec();

            // cluster_cfs_configs = vec![cluster_cfs_configs.last().unwrap().clone()]; // vec! macro for vector initialization!!! https://doc.rust-lang.org/std/vec/struct.Vec.html
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