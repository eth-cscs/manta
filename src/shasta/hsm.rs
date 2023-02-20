pub mod http_client {

    use std::error::Error;

    use serde_json::Value;

    /// Get list of HSM groups using --> https://apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doGroupsGet/
    /// NOTE: this returns all HSM groups which name contains hsm_groupu_name param value
    pub async fn get_hsm_groups(
        shasta_token: &str,
        shasta_base_url: &str,
        hsm_group_name: Option<&String>,
    ) -> Result<Vec<Value>, Box<dyn Error>> {
        let mut hsm_groups: Vec<Value> = Vec::new();

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

        let mut url_api = shasta_base_url.to_string();
        url_api.push_str("/smd/hsm/v2/groups");

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

        if hsm_group_name.is_some() {
            for hsm_group in json_response.as_array().unwrap() {
                if hsm_group["label"]
                    .as_str()
                    .unwrap()
                    .contains(hsm_group_name.unwrap())
                // TODO: investigate why I need to use this ugly 'as_ref'
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
        hsm_group_name: &String,
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

        let mut url_api = shasta_base_url.to_string();
        url_api.push_str("/smd/hsm/v2/groups/");
        url_api.push_str(hsm_group_name);

        let resp = client
            .get(url_api)
            .header("Authorization", format!("Bearer {}", shasta_token))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
            //json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
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
