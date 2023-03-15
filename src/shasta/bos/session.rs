pub mod http_client {

    use serde_json::{Value, json};

    use crate::shasta::bos::template::BosTemplate;

    pub async fn post(
        shasta_token: &str,
        shasta_base_url: &str,
        bos_template_name: &String,
        operation: &str,
        limit: Option<&String>,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        // // socks5 proxy
        // let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // // rest client create new cfs sessions
        // let client = reqwest::Client::builder()
        //     .danger_accept_invalid_certs(true)
        //     .proxy(socks5proxy)
        //     .build()?;

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
            .post(format!("{}{}", shasta_base_url, "/bos/v1/session"))
            .bearer_auth(shasta_token)
            .json(&json!({
                "operation": operation,
                "templateName": bos_template_name,
                "limit": limit
            }))
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

    pub async fn get(
        shasta_token: &str,
        shasta_base_url: &str,
        cluster_name: &Option<String>,
        bos_session_name: &Option<String>,
        limit_number: &Option<u8>,
    ) -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut cluster_bos_session: Vec<Value> = Vec::new();

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

        let mut api_url = shasta_base_url.clone().to_string();
        api_url.push_str("/bos/v1/session");

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/bos/v1/session"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value;

        if resp.status().is_success() {
            json_response = serde_json::from_str(&resp.text().await?)?;
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
        }

        println!("\nBOS SESSIONS:\n{:#?}", json_response);

        if cluster_name.is_some() {
            for bos_template in json_response.as_array().unwrap() {
                if bos_template["name"]
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap())
                // TODO: investigate why I need to use this ugly 'as_ref'
                {
                    cluster_bos_session.push(bos_template.clone());
                }
            }
        } else if bos_session_name.is_some() {
            for bos_session in json_response.as_array().unwrap() {
                if bos_session["name"]
                    .as_str()
                    .unwrap()
                    .eq(bos_session_name.as_ref().unwrap())
                // TODO: investigate why I need to us this ugly 'as_ref'
                {
                    cluster_bos_session.push(bos_session.clone());
                }
            }
        } else {
            // Returning all results
            cluster_bos_session = json_response.as_array().unwrap().to_vec();
        }

        if limit_number.is_some() {
            // Limiting the number of results to return to client

            cluster_bos_session = cluster_bos_session[cluster_bos_session
                .len()
                .saturating_sub(limit_number.unwrap().into())..]
                .to_vec();
        }

        Ok(cluster_bos_session)
    }
}

pub mod utils {

    use comfy_table::Table;
    use serde_json::Value;

    pub fn print_table(bos_sessions: Vec<Value>) {
        let mut table = Table::new();

        table.set_header(vec!["Operation", "Template name", "Job", "Limit"]);

        for bos_session in bos_sessions {
            table.add_row(vec![
                bos_session["operation"].as_str().unwrap(),
                bos_session["templateName"].as_str().unwrap_or_default(),
                bos_session["job"].as_str().unwrap_or_default(),
                bos_session["limit"].as_str().unwrap_or_default(),
            ]);
        }

        println!("{table}");
    }
}
