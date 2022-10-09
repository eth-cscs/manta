use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    property1: String,
    property2: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    name: String,
    members: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    groups: Option<Vec<Group>>
}

impl Default for Target {
    fn default() -> Target {
        Target {
            definition: String::from("dynamic"),
            groups: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    name: String,
    #[serde(rename = "configurationName")]
    configuration_name: String,
    #[serde(rename = "configurationLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    configuration_limit: Option<String>,
    #[serde(rename = "ansibleLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ansible_limit: Option<String>,
    #[serde(rename = "ansibleConfig")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ansible_config: Option<String>,
    #[serde(rename = "ansibleVerbosity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ansible_verbosity: Option<u8>,
    #[serde(rename = "ansiblePassthrough")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ansible_passthrough: Option<String>,
    #[serde(default)]
    target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Tag>
}

impl Default for Session {
    fn default() -> Session {
        Session {
            name: String::from(""),
            configuration_name: String::from(""),
            configuration_limit: None,
            ansible_limit: None,
            ansible_config: None,
            ansible_verbosity: None,
            ansible_passthrough: None,
            target: Default::default(),
            tags: None,
        }
    }
}

impl Session {
    pub fn new(name: String, configuration_name: String, ansibe_limit: Option<String>) -> Self {
        Session {
            name,
            configuration_name,
            ansible_limit: ansibe_limit,
            ..Default::default()
        }
    }
}

pub mod http_client {

    use serde_json::Value;

    use super::Session;

    pub async fn post(shasta_token: &str, shasta_base_url: &str, session: Session) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // rest client create new cfs sessions
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
    
        let resp = client
            .post(format!("{}{}", shasta_base_url, "/cfs/v2/sessions"))
            .bearer_auth(shasta_token)
            .json(&session)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(serde_json::from_str(&resp.text().await?)?)
        } else {
            Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    }

    pub async fn get(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, session_name: &Option<String>, limit_number: &Option<u8>) -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {

        let mut cluster_cfs_sessions: Vec<Value> = Vec::new();

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // rest client get cfs sessions
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
    
        let resp = client
            .get(format!("{}{}", shasta_base_url, "/cfs/v2/sessions"))
            .bearer_auth(shasta_token)
            .send()
            .await?
            .text()
            .await?;
            
        let json_response: Value = serde_json::from_str(&resp)?;
    
        if cluster_name.is_some() {
            for cfs_session in json_response.as_array().unwrap() {
    
                if cfs_session["configuration"]["name"]
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap()) // TODO: investigate why I need to use this ugly 'as_ref'
                {
                    cluster_cfs_sessions.push(cfs_session.clone());
                }
            }
        } else if session_name.is_some() {
            for cfs_session in json_response.as_array().unwrap() {
                if cfs_session
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq(session_name.as_ref().unwrap()) // TODO: investigate why I need to us this ugly 'as_ref'
                {
                    cluster_cfs_sessions.push(cfs_session.clone());
                }
            }
        } else { // Returning all results
            cluster_cfs_sessions = json_response.as_array().unwrap().to_vec();

            cluster_cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));
        }
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            cluster_cfs_sessions = json_response.as_array().unwrap().to_vec();
    
            cluster_cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));
    
            // cfs_utils::print_cfs_configurations(&cfs_configurations);
            
            // cluster_cfs_sessions.truncate(limit_number.unwrap().into());
            cluster_cfs_sessions = cluster_cfs_sessions[cluster_cfs_sessions.len().saturating_sub(limit_number.unwrap().into())..].to_vec();
            
            // cluster_cfs_sessions = vec![cluster_cfs_sessions]; // vec! macro for vector initialization!!! https://doc.rust-lang.org/std/vec/struct.Vec.html
        } 

        Ok(cluster_cfs_sessions)
    }
}