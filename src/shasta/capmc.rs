use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct PowerStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    xnames: Vec<String>,
    force: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    recursive: Option<bool>
}

impl PowerStatus {
    pub fn new(reason: Option<String>, xnames: Vec<String>, force: bool, recursive: Option<bool>) -> Self {
        Self {
            reason,
            xnames,
            force,
            recursive
        }
    }

    pub fn add_component_id(&mut self, xname: String) {
        self.xnames.push(xname);
    }
}

impl Default for PowerStatus {
    fn default() -> Self {
        Self{
            reason: None,
            xnames: vec![],
            force: false,
            recursive: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xnames: Option<Vec<String>>
}

impl NodeStatus {
    pub fn new(filter: Option<String>, xnames: Option<Vec<String>>, source: Option<String>) -> Self {
        Self { filter, source, xnames }
    }
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self{
            filter: None,
            xnames: None,
            source: None,
        }
    }
}

pub mod http_client {
    pub mod node_power_off {

        use std::error::Error;

        use serde_json::Value;

        use crate::capmc::PowerStatus;

        pub async fn post(shasta_token: String, reason: Option<&String>, xnames: Vec<String>, force: bool)  -> Result<Value, Box<dyn Error>> {

            log::info!("Shutting down {:?}", xnames);

            let power_off = PowerStatus::new(reason.cloned(), xnames, force, None);

            // log::debug!("Payload:\n{:#?}", power_off);

            // // socks5 proxy
            // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // // rest client shutdown node
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
                .post("https://api.cmn.alps.cscs.ch/apis/capmc/capmc/v1/xname_off")
                .bearer_auth(shasta_token)
                .json(&power_off)
                .send()
                .await?;

            // log::info!("{:#?}", resp);

            if resp.status().is_success() {
                Ok(resp.json::<Value>().await?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }

    pub mod node_power_on {
        use std::error::Error;

        use serde_json::Value;

        use crate::capmc::PowerStatus;

        pub async fn post(shasta_token: String, reason: Option<&String>, xnames: Vec<String>, force: bool) -> Result<Value, Box<dyn Error>> {
            
            log::info!("Powering on nodes {:?}", xnames);

            let power_on = PowerStatus::new(reason.cloned(), xnames, force, None);
            
            // log::debug!("Payload:\n{:#?}", power_on);

            // // socks5 proxy
            // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // // rest client start node
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
                .post("https://api.cmn.alps.cscs.ch/apis/capmc/capmc/v1/xname_on")
                .bearer_auth(shasta_token)
                .json(&power_on)
                .send()
                .await?;

            // log::debug!("\n{:#?}", resp);

            if resp.status().is_success() {
                Ok(resp.json::<Value>().await?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }

    pub mod node_restart {
        use std::error::Error;

        use serde_json::Value;

        use crate::capmc::PowerStatus;

        pub async fn post(shasta_token: String, reason: Option<&String>, xnames: Vec<String>, force: bool)  -> Result<Value, Box<dyn Error>> {
            
            log::info!("Restarting {:?}", xnames);

            let node_restart = PowerStatus::new(reason.cloned(), xnames, force, None);
            
            // log::debug!("Payload:\n{:#?}", node_restart);

            // // socks5 proxy
            // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // // rest client restart node
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
                .post("https://api.cmn.alps.cscs.ch/apis/capmc/capmc/v1/xname_reinit")
                .bearer_auth(shasta_token)
                .json(&node_restart)
                .send()
                .await?;

            // log::debug!("\n{:#?}", resp);

            if resp.status().is_success() {
                Ok(resp.json::<Value>().await?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }

    pub mod node_status {
        use std::error::Error;

        use serde_json::Value;

        use crate::capmc::NodeStatus;

        pub async fn post(shasta_token: String, xnames: Vec<String>)  -> core::result::Result<Vec<Value>, Box<dyn Error>> {
            
            let node_status = NodeStatus::new(None, Some(xnames), None);
            
            // // socks5 proxy
            // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // // rest client get node status
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
                .post("https://api.cmn.alps.cscs.ch/apis/capmc/capmc/v1/get_xname_status")
                .bearer_auth(shasta_token)
                .json(&node_status)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(serde_json::from_str(&resp.text().await?)?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }
}