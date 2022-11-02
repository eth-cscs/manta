

pub mod http_client {

    use std::error::Error;

    use serde_json::Value;

    pub async fn get_hsm_groups(shasta_token: &str, shasta_base_url: &str, cluster_name: Option<String>) -> Result<Vec<Value>, Box<dyn Error>> {

        let mut hsm_groups: Vec<Value> = Vec::new();

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

        let json_response: Value;

        let resp = client
            .get(format!("{}/smd/hsm/v2/groups", shasta_base_url))
            .header("Authorization", format!("Bearer {}", shasta_token))
            .send()
            .await?;

        if resp.status().is_success() {
            // Ok(serde_json::from_str(&resp.text().await?)?)
            json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not 
        };

        if cluster_name.is_some() {
            for hsm_group in json_response.as_array().unwrap() {
    
                if hsm_group["label"]
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap()) // TODO: investigate why I need to use this ugly 'as_ref'
                {
                    hsm_groups.push(hsm_group.clone());
                }

                // hsm_groups.sort_by(|a, b| a["status"]["session"]["startTime"].as_str().unwrap().cmp(&b["status"]["session"]["startTime"].as_str().unwrap()));

            }
        }

        Ok(hsm_groups)

    }


    pub async fn get_component_status(shasta_token: &str, shasta_base_url: &str, xname: &str) -> Result<Value, Box<dyn Error>> {

        // // socks5 proxy
        // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // // rest client get commit details
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
            .get(format!("{}/smd/hsm/v2/State/Components/{}", shasta_base_url, xname))
            .header("Authorization", format!("Bearer {}", shasta_token))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(serde_json::from_str(&resp.text().await?)?)
        } else {
            Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    }
}

pub mod utils {
    
    use comfy_table::Table;
    use serde_json::Value;

    pub fn print_table(hsm_groups: Vec<Value>) {
        
        let mut table = Table::new();

        table.set_header(vec!["Label", "Description", "Tags", "Exclusive group", "Members"]);
    
        for hsm_group in hsm_groups {

            let list_members = hsm_group["members"]["ids"].as_array().unwrap();

            let mut members: String = String::new();

            if !list_members.is_empty() {
                
                members = list_members[0].as_str().unwrap().to_string();

                for i in 1..list_members.len() {

                    if i % 10 == 0 { // breaking the cell content into multiple lines (only 2 xnames per line)
                        members = format!("{},\n", members);
                    } else {
                        members = format!("{}, ", members);
                    }
    
                    members = format!("{}{}", members, list_members[i].as_str().unwrap());    
                }
            }

            table.add_row(vec![
                hsm_group["label"].as_str().unwrap(),
                hsm_group["description"].as_str().unwrap(),
                hsm_group["tags"].as_str().unwrap_or_default(),
                hsm_group["exclusiveGroup"].as_str().unwrap_or_default(),
                &members
            ]);
        }
    
        println!("{table}");
    }
}