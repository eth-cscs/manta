use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    property1: String,
    property2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    name: String,
    members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    groups: Option<Vec<Group>>,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            definition: String::from("dynamic"),
            groups: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfsSession {
    pub name: String,
    #[serde(rename = "configurationName")]
    pub configuration_name: String,
    #[serde(rename = "configurationLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_limit: Option<String>,
    #[serde(rename = "ansibleLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansible_limit: Option<String>,
    #[serde(rename = "ansibleConfig")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansible_config: Option<String>,
    #[serde(rename = "ansibleVerbosity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansible_verbosity: Option<u8>,
    #[serde(rename = "ansiblePassthrough")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansible_passthrough: Option<String>,
    #[serde(default)]
    pub target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tag>,
    #[serde(skip_serializing)]
    pub base_image_id: Option<String>,
}

impl Default for CfsSession {
    fn default() -> Self {
        Self {
            name: String::default(),
            configuration_name: String::default(),
            configuration_limit: None,
            ansible_limit: None,
            ansible_config: None,
            ansible_verbosity: None,
            ansible_passthrough: None,
            target: Default::default(),
            tags: None,
            base_image_id: Some(String::default()),
        }
    }
}

impl CfsSession {
    pub fn new(
        name: String,
        configuration_name: String,
        ansible_limit: Option<String>,
        ansible_verbosity: Option<u8>,
        is_target_definition_image: bool,
        groups_name: Option<Vec<String>>,
        base_image_id: Option<String>,
    ) -> Self {
        // This code is fine... the fact that I put Self behind a variable is ok, since image param
        // is not a default param, then doing things differently is not an issue. I checked with
        // other Rust developers in their discord https://discord.com/channels/442252698964721669/448238009733742612/1081686300182188207
        let mut cfs_session = Self {
            name,
            configuration_name,
            ansible_limit,
            ansible_verbosity,
            ..Default::default()
        };

        if is_target_definition_image {
            // let base_image_id = "a897aa21-0218-4d07-aefb-13a4c15ccb65"; // TODO: move this to config
            // file ???

            let target_groups: Vec<Group> = groups_name
                .unwrap()
                .into_iter()
                .map(|group_name| Group {
                    name: group_name,
                    members: vec![base_image_id.as_ref().unwrap().to_string()],
                })
                .collect();

            cfs_session.target.definition = "image".to_string();
            cfs_session.target.groups = Some(target_groups);
        }

        cfs_session
    }

    pub fn from_sat_file_serde_yaml(
        session_yaml: &serde_yaml::Value,
        base_image_id: &String,
    ) -> Self {
        let groups_name = session_yaml["configuration_group_names"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|group_name| group_name.as_str().unwrap().to_string())
            .collect();

        let cfs_session = crate::shasta::cfs::session::CfsSession::new(
            session_yaml["name"].as_str().unwrap().to_string(),
            session_yaml["configuration"].as_str().unwrap().to_string(),
            None,
            None,
            true,
            Some(groups_name),
            Some(base_image_id.to_string()),
        );
        cfs_session
    }
}

pub mod http_client {

    use super::CfsSession;
    use serde_json::Value;
    use std::collections::HashSet;
    use std::error::Error;

    use termion::color;

    pub async fn post(
        shasta_token: &str,
        shasta_base_url: &str,
        session: &CfsSession,
    ) -> Result<Value, Box<dyn Error>> {
        log::debug!("Session:\n{:#?}", session);

        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let mut api_url = shasta_base_url.to_string();
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
            Err(resp.json::<Value>().await?["detail"]
                .as_str()
                .unwrap()
                .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    /// Fetch CFS sessions ref --> https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/
    pub async fn get(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group_name: Option<&String>,
        session_name: Option<&String>,
        limit_number: Option<&u8>,
        is_succeded: Option<bool>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let mut api_url = shasta_base_url.to_string();
        api_url.push_str("/cfs/v2/sessions");

        let mut request_payload = Vec::new();

        if is_succeded.is_some() {
            request_payload.push(("succeced", is_succeded));
        }

        let resp = client
            .get(api_url)
            .query(&request_payload)
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
            let hsm_group_resp = crate::shasta::hsm::http_client::get_hsm_group(
                shasta_token,
                shasta_base_url,
                hsm_group_name.unwrap(),
            )
            .await;

            let hsm_group_nodes = if hsm_group_resp.is_ok() {
                crate::shasta::hsm::utils::get_member_ids(&hsm_group_resp.unwrap())
            } else {
                eprintln!(
                    "No HSM group {}{}{} found!",
                    color::Fg(color::Red),
                    hsm_group_name.unwrap(),
                    color::Fg(color::Reset)
                );
                std::process::exit(1);
            };

            // Checks either target.groups contains hsm_group_name or ansible.limit is a subset of
            // hsm_group.members.ids
            cluster_cfs_sessions.retain(|cfs_session| {
                cfs_session["target"]["groups"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .any(|group| {
                        group["name"]
                            .as_str()
                            .unwrap()
                            .to_string()
                            .eq(hsm_group_name.unwrap())
                    })
                    || cfs_session["ansible"]["limit"]
                        .as_str()
                        .unwrap_or("")
                        .split(',')
                        .into_iter()
                        .map(|node| node.trim().to_string())
                        .collect::<HashSet<_>>()
                        .is_subset(&HashSet::from_iter(hsm_group_nodes.clone()))
            });
        }

        if session_name.is_some() {
            cluster_cfs_sessions.retain(|cfs_session| {
                cfs_session["name"]
                    .as_str()
                    .unwrap()
                    .eq(session_name.unwrap())
            });
        }

        cluster_cfs_sessions.sort_by(|a, b| {
            a["status"]["session"]["startTime"]
                .as_str()
                .unwrap()
                .cmp(b["status"]["session"]["startTime"].as_str().unwrap())
        });

        if limit_number.is_some() {
            // Limiting the number of results to return to client

            cluster_cfs_sessions = cluster_cfs_sessions[cluster_cfs_sessions
                .len()
                .saturating_sub(*limit_number.unwrap() as usize)..]
                .to_vec();
        }

        Ok(cluster_cfs_sessions)
    }
}

pub mod utils {

    use comfy_table::Table;
    use serde_json::Value;

    pub fn print_table(cfs_sessions: Vec<Value>) {
        let mut table = Table::new();

        table.set_header(vec![
            "Name",
            "Configuration",
            "Target Def",
            // "Target groups",
            // "Ansible limit",
            "Target",
            "Start",
            "Status",
            "Succeeded",
            "Image ID",
        ]);

        for cfs_session in cfs_sessions {
            let target_groups = if cfs_session["target"]["groups"].as_array().is_some()
                && (cfs_session["target"]["groups"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .len()
                    > 0)
            {
                let target_groups_json = cfs_session["target"]["groups"].as_array().unwrap();

                let mut target_groups_aux = String::from(target_groups_json[0]["name"].as_str().unwrap());

                for (i, _) in target_groups_json.iter().enumerate().skip(1) {
                    if i % 2 == 0 {
                        // breaking the cell content into multiple lines (only 2 target groups per line)
                        target_groups_aux.push_str(",\n");
                        // target_groups = format!("{},\n", target_groups);
                    } else {
                        target_groups_aux.push_str(", ");
                        // target_groups = format!("{}, ", target_groups);
                    }

                    target_groups_aux.push_str(target_groups_json[i]["name"].as_str().unwrap());
                }

                target_groups_aux
            } else {
                "".to_string()
            };

            let mut list_ansible_limit = cfs_session["ansible"]["limit"]
                .as_str()
                .unwrap_or_default()
                .split(',');

            let first = list_ansible_limit.next();

            let ansible_limits = if let Some(first_xname) = first {
                let mut ansible_limits_aux = String::from(first_xname);

                let mut i = 1;

                for ansible_limit in list_ansible_limit {
                    if i % 2 == 0 {
                        // breaking the cell content into multiple lines (only 2 xnames per line)
                        ansible_limits_aux.push_str(", \n");
                        // ansible_limits = format!("{},\n", ansible_limits);
                    } else {
                        ansible_limits_aux.push_str(", ");
                        // ansible_limits = format!("{}, ", ansible_limits);
                    }

                    ansible_limits_aux.push_str(ansible_limit);
                    // ansible_limits = format!("{}{}", ansible_limits, ansible_limit);

                    i += 1;
                }

                ansible_limits_aux
            } else {
                "".to_string()
            };

            let target_definition = cfs_session["target"]["definition"].as_str().unwrap();

            let target = if !target_groups.is_empty() {
                &target_groups
            } else {
                &ansible_limits
            };

            let result_id = if !cfs_session["status"]["artifacts"]
                .as_array()
                .unwrap()
                .is_empty()
            {
                cfs_session["status"]["artifacts"][0]["result_id"]
                    .as_str()
                    .unwrap()
            } else {
                ""
            };

            table.add_row(vec![
                cfs_session["name"].as_str().unwrap(),
                cfs_session["configuration"]["name"].as_str().unwrap(),
                target_definition,
                // &target_groups,
                // &ansible_limits,
                target,
                cfs_session["status"]["session"]["startTime"]
                    .as_str()
                    .unwrap(),
                cfs_session["status"]["session"]["status"].as_str().unwrap(),
                cfs_session["status"]["session"]["succeeded"]
                    .as_str()
                    .unwrap(),
                result_id,
            ]);
        }

        println!("{table}");
    }
}
