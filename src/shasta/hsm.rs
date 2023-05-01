/// Refs:
/// Member/node state --> https://apidocs.svc.cscs.ch/iaas/hardware-state-manager/overview/#section/Valid-State-Transistions

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
struct HsmGroup {
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    members: Option<Vec<Member>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Member {
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<String>>,
}

/* impl HsmGroup {
    pub fn new(
        label: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
        members: Option<Vec<Member>>,
    ) -> Self {
        Self {
            label,
            description,
            tags,
            members,
        }
    }
} */

/* impl Member {
    pub fn new(ids: Option<Vec<String>>) -> Self {
        Self { ids }
    }
} */

pub mod http_client {

    use std::error::Error;

    use reqwest::Url;
    use serde_json::Value;

    pub async fn get_all_hsm_groups(
        shasta_token: &str,
        shasta_base_url: &str,
    ) -> Result<Vec<Value>, Box<dyn Error>> {
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

        let json_response: Value;

        let url_api = shasta_base_url.to_owned() + "/smd/hsm/v2/groups";

        let resp = client
            .get(url_api)
            .header("Authorization", format!("Bearer {}", shasta_token))
            .send()
            .await?;

        if resp.status().is_success() {
            json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
        };

        Ok(json_response.as_array().unwrap().to_owned())
    }

    /// Get list of HSM groups using --> https://apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doGroupsGet/
    /// NOTE: this returns all HSM groups which name contains hsm_groupu_name param value
    pub async fn get_hsm_groups(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group_name: Option<&String>,
    ) -> Result<Vec<Value>, Box<dyn Error>> {
        let json_response = get_all_hsm_groups(shasta_token, shasta_base_url).await?;

        let mut hsm_groups: Vec<Value> = Vec::new();

        if hsm_group_name.is_some() {
            for hsm_group in json_response {
                if hsm_group["label"]
                    .as_str()
                    .unwrap()
                    .contains(hsm_group_name.unwrap())
                {
                    hsm_groups.push(hsm_group.clone());
                }
            }
        }

        Ok(hsm_groups)
    }

    /// Get list of HSM group using --> shttps://apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doGroupsGet/
    pub async fn get_hsm_group(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group_name: &str,
    ) -> Result<Value, Box<dyn Error>> {
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

        let url_api = shasta_base_url.to_owned() + "/smd/hsm/v2/groups/" + hsm_group_name;

        let resp = client
            .get(url_api)
            .header("Authorization", format!("Bearer {}", shasta_token))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
            //json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            Err(resp.text().await?.into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    /// Fetches node/compnent details using HSM v2 ref --> https://apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doComponentsGet/
    pub async fn get_component_status(
        shasta_token: &str,
        shasta_base_url: &str,
        xname: &str,
    ) -> Result<Value, Box<dyn Error>> {
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

        let resp = client
            .get(format!(
                "{}/smd/hsm/v2/State/Components/{}",
                shasta_base_url, xname
            ))
            .header("Authorization", format!("Bearer {}", shasta_token))
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

    /// Fetches nodes/compnents details using HSM v2 ref --> https://apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doComponentsGet/
    pub async fn get_components_status(
        shasta_token: &str,
        shasta_base_url: &str,
        xnames: Vec<String>,
    ) -> Result<Value, Box<dyn Error>> {
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

        let url_params: Vec<_> = xnames.iter().map(|xname| ("id", xname)).collect();
        let api_url = Url::parse_with_params(
            &format!("{}/smd/hsm/v2/State/Components", shasta_base_url),
            &url_params,
        )?;

        let resp = client
            .get(api_url)
            .header("Authorization", format!("Bearer {}", shasta_token))
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
}

// pub mod utils {

//     use comfy_table::Table;
//     use serde_json::Value;

//     use crate::shasta::nodes::nodes_to_string;

//     pub fn print_table(hsm_groups: Vec<Value>) {

//         let mut table = Table::new();

//         table.set_header(vec!["Label", "Description", "Tags", "Exclusive group", "Members"]);

//         for hsm_group in hsm_groups {

//             let list_members = hsm_group["members"]["ids"].as_array().unwrap();

//             let members = nodes_to_string(list_members);

//             table.add_row(vec![
//                 hsm_group["label"].as_str().unwrap(),
//                 hsm_group["description"].as_str().unwrap(),
//                 hsm_group["tags"].as_str().unwrap_or_default(),
//                 hsm_group["exclusiveGroup"].as_str().unwrap_or_default(),
//                 &members
//             ]);
//         }

//         println!("{table}");
//     }
// }

pub mod utils {

    use std::collections::{HashMap, HashSet};

    use serde_json::Value;

    use crate::shasta::hsm::http_client::get_all_hsm_groups;

    use super::http_client;

    pub fn get_members_from_hsm_group_serde_value(hsm_group: &Value) -> Vec<String> {
        // Take all nodes for all hsm_groups found and put them in a Vec
        hsm_group["members"]["ids"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|xname| xname.as_str().unwrap().to_string())
            .collect()
    }

    pub fn get_members_from_hsm_groups_serde_value(hsm_groups: &[Value]) -> HashSet<String> {
        hsm_groups
            .iter()
            .flat_map(get_members_from_hsm_group_serde_value)
            .collect()
    }

    /// Returns a Map with nodes and the list of hsm groups that node belongs to.
    /// eg "x1500b5c1n3 --> [ psi-dev, psi-dev_cn ]"
    pub fn group_members_by_hsm_group_from_hsm_groups_serde_value(
        hsm_groups: &Vec<Value>,
    ) -> HashMap<String, Vec<String>> {
        let mut member_hsm_map: HashMap<String, Vec<String>> = HashMap::new();
        for hsm_group_value in hsm_groups {
            let hsm_group_name = hsm_group_value["label"].as_str().unwrap().to_string();
            for member in get_members_from_hsm_group_serde_value(hsm_group_value) {
                member_hsm_map
                    .entry(member)
                    .and_modify(|hsm_groups| hsm_groups.push(hsm_group_name.clone()))
                    .or_insert_with(|| vec![hsm_group_name.clone()]);
            }
        }

        member_hsm_map
    }

    pub async fn get_members_ids(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group: &str,
    ) -> Vec<String> {
        // Take all nodes for all hsm_groups found and put them in a Vec
        http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group)
            .await
            .unwrap()["members"]["ids"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|xname| xname.as_str().unwrap().to_string())
            .collect()
    }

    pub async fn get_hsm_group_from_xname(
        shasta_token: &str,
        shasta_base_url: &str,
        xname: &String,
    ) -> Option<String> {
        let hsm_groups_details = get_all_hsm_groups(shasta_token, shasta_base_url)
            .await
            .unwrap();

        for hsm_group_details in hsm_groups_details.iter() {
            if hsm_group_details["members"]["ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str().unwrap() == xname)
            {
                /* println!(
                    "hsm group seems to have the member:\n{:#?}",
                    hsm_group_details
                ); */
                return Some(hsm_group_details["label"].as_str().unwrap().to_string());
            }
        }

        None
    }
}
