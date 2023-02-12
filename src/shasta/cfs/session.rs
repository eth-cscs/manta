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
    fn default() -> Self {
        Self {
            definition: String::from("dynamic"),
            groups: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfsSession {
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
    ansible_verbosity: u8,
    #[serde(rename = "ansiblePassthrough")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ansible_passthrough: Option<String>,
    #[serde(default)]
    target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Tag>
}

impl Default for CfsSession {
    fn default() -> Self {
        Self {
            name: String::from(""),
            configuration_name: String::from(""),
            configuration_limit: None,
            ansible_limit: None,
            ansible_config: None,
            ansible_verbosity: 2,
            ansible_passthrough: None,
            target: Default::default(),
            tags: None,
        }
    }
}

impl CfsSession {
    pub fn new(name: String, configuration_name: String, ansible_limit: Option<String>, ansible_verbosity: u8) -> Self {
        Self {
            name,
            configuration_name,
            ansible_limit,
            ansible_verbosity,
            ..Default::default()
        }
    }
}

pub mod http_client {

    use std::error::Error;
    use serde_json::Value;
    use super::CfsSession;

    pub async fn post(shasta_token: &str, shasta_base_url: &str, session: CfsSession) -> Result<Value, Box<dyn Error>> {

        log::debug!("Session:\n{:#?}", session);

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

        let mut api_url = shasta_base_url.clone().to_string();
        api_url.push_str("/cfs/v2/sessions");
    
        let resp = client
            .post(api_url)
            // .post(format!("{}{}", shasta_base_url, "/cfs/v2/sessions"))
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

    pub async fn get(shasta_token: &str, shasta_base_url: &str, hsm_group_name: Option<&String>, session_name: Option<&String>, limit_number: Option<&u8>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {

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
        
        let mut api_url = shasta_base_url.clone().to_string();
        api_url.push_str("/cfs/v2/sessions");

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/cfs/v2/sessions"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value = if resp.status().is_success() {
            serde_json::from_str(&resp.text().await?)?
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not 
        };

        let mut cluster_cfs_sessions = json_response.as_array().unwrap().clone();
    
        if hsm_group_name.is_some() {

            cluster_cfs_sessions
                .retain(|cfs_session| {
                    cfs_session["configuration"]["name"]
                    .as_str()
                    .unwrap()
                    .contains(hsm_group_name.unwrap())
                });

        }
        
        if session_name.is_some() {

            cluster_cfs_sessions
                .retain(|cfs_session| {
                    cfs_session["name"]
                    .as_str()
                    .unwrap()
                    .eq(session_name.unwrap())
                });

        }

        cluster_cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].as_str().unwrap().cmp(b["status"]["session"]["startTime"].as_str().unwrap()));
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            cluster_cfs_sessions = cluster_cfs_sessions[cluster_cfs_sessions.len().saturating_sub(*limit_number.unwrap() as usize)..].to_vec();
            
        } 

        Ok(cluster_cfs_sessions)
    }
}

pub mod utils {

    use comfy_table::Table;
    use serde_json::Value;

    pub fn print_table(cfs_sessions: Vec<Value>) {
        
        let mut table = Table::new();

        table.set_header(vec!["Name", "Configuration", "Target", "Target groups", "Ansible limit", "Start", "Status", "Succeeded", "Job"]);
    
        for cfs_session in cfs_sessions {

            let mut target_groups: String = String::new();

            if cfs_session["target"]["groups"].as_array().is_some() && (cfs_session["target"]["groups"].as_array().unwrap().iter().len() > 0) {

                let target_groups_json = cfs_session["target"]["groups"].as_array().unwrap();

                target_groups = String::from(target_groups_json[0]["name"].as_str().unwrap());

                for (i, _) in target_groups_json.iter().enumerate().skip(1) {

                    if i % 2 == 0 { // breaking the cell content into multiple lines (only 2 target groups per line)
                        target_groups.push_str(",\n");
                        // target_groups = format!("{},\n", target_groups);
                    } else {
                        target_groups.push_str(", ");
                        // target_groups = format!("{}, ", target_groups);
                    }
                    
                    target_groups.push_str(target_groups_json[i]["name"].as_str().unwrap());

                    // target_groups = format!("{}{}", target_groups, target_groups_json[i]["name"].as_str().unwrap());
                }
            }

            let mut list_ansible_limit = cfs_session["ansible"]["limit"].as_str().unwrap_or_default().split(',');

            let mut ansible_limits: String = String::new();

            let first = list_ansible_limit.next();

            if let Some(inner) = first {
                
                ansible_limits = String::from(inner);

                let mut i = 1;

                for ansible_limit in list_ansible_limit {

                    if i % 2 == 0 { // breaking the cell content into multiple lines (only 2 xnames per line)
                        ansible_limits.push_str(", \n");
                        // ansible_limits = format!("{},\n", ansible_limits);
                    } else {
                        ansible_limits.push_str(", ");
                        // ansible_limits = format!("{}, ", ansible_limits);
                    }
    
                    ansible_limits.push_str(ansible_limit);
                    // ansible_limits = format!("{}{}", ansible_limits, ansible_limit);
    
                    i += 1;
                }
            }

            table.add_row(vec![
                cfs_session["name"].as_str().unwrap(),
                cfs_session["configuration"]["name"].as_str().unwrap(),
                cfs_session["target"]["definition"].as_str().unwrap(),
                &target_groups,
                &ansible_limits,
                cfs_session["status"]["session"]["startTime"].as_str().unwrap(),
                cfs_session["status"]["session"]["status"].as_str().unwrap(),
                cfs_session["status"]["session"]["succeeded"].as_str().unwrap(),
                cfs_session["status"]["session"]["job"].as_str().unwrap()
            ]);
        }
    
        println!("{table}");
    }
}
