use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct PowerStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    xnames: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    force: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recursive: Option<bool>
}

impl PowerStatus {
    pub fn new(reason: Option<String>, xnames: String, force: Option<bool>, recursive: Option<bool>) -> Self {
        PowerStatus {
            ..Default::default()
        }
    }
}

impl Default for PowerStatus {
    fn default() -> PowerStatus {
        PowerStatus{
            reason: None,
            xnames: String::from(""),
            force: None,
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
    xnames: Option<String>
}

impl NodeStatus {
    pub fn new(filter: Option<String>, xnames: Option<String>, source: Option<bool>) -> Self {
        NodeStatus {
            ..Default::default()
        }
    }
}

impl Default for NodeStatus {
    fn default() -> NodeStatus {
        NodeStatus{
            filter: None,
            xnames: None,
            source: None,
        }
    }
}

pub mod http_client {
    pub mod node_power_off {
        use serde_json::Value;

        use crate::shasta_capmc::PowerStatus;

        pub async fn post(shasta_token: String, reason: Option<String>, xnames: String, force: Option<bool>)  -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {

            let power_off = PowerStatus::new(reason, xnames, force, None);

            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // rest client shutdown node
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .proxy(socks5proxy)
                .build()?;
        
            let resp = client
                .post("https://api-gw-service-nmn.local/apis/capmc/capmc/v1/xname_off")
                .bearer_auth(shasta_token)
                .json(&power_off)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(serde_json::from_str(&resp.text().await?)?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }

    mod node_power_on {
        use serde_json::Value;

        use crate::shasta_capmc::PowerStatus;

        pub async fn post(shasta_token: String, reason: Option<String>, xnames: String, force: Option<bool>)  -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {
            
            let power_on = PowerStatus::new(reason, xnames, force, None);
            
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // rest client start node
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .proxy(socks5proxy)
                .build()?;
        
            let resp = client
                .post("https://api-gw-service-nmn.local/apis/capmc/capmc/v1/xname_on")
                .bearer_auth(shasta_token)
                .json(&power_on)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(serde_json::from_str(&resp.text().await?)?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }

    pub mod node_restart {
        use serde_json::Value;

        use crate::shasta_capmc::PowerStatus;

        pub async fn post(shasta_token: String, reason: Option<String>, xnames: String, force: Option<bool>)  -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {
            
            let node_restart = PowerStatus::new(reason, xnames, force, None);
            
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // rest client restart node
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .proxy(socks5proxy)
                .build()?;
        
            let resp = client
                .post("https://api-gw-service-nmn.local/apis/capmc/capmc/v1/xname_reinit")
                .bearer_auth(shasta_token)
                .json(&node_restart)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(serde_json::from_str(&resp.text().await?)?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }

    pub mod node_status {
        use serde_json::Value;

        use crate::shasta_capmc::NodeStatus;

        pub async fn post(shasta_token: String, xnames: String)  -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {
            
            let node_status = NodeStatus::new(None, Some(xnames), None);
            
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // rest client get node status
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .proxy(socks5proxy)
                .build()?;
        
            let resp = client
                .post("https://api-gw-service-nmn.local/apis/capmc/capmc/v1/get_xname_status")
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