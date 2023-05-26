pub mod http_client {

    use serde_json::{json, Value};

    pub async fn post(
        shasta_token: &str,
        shasta_base_url: &str,
        bos_template_name: &String,
        operation: &str,
        limit: Option<&String>,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            log::debug!("SOCKS5 enabled");
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
        api_url.push_str("/bos/v1/session");

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/bos/v1/session"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        let json_response: Value = if resp.status().is_success() {
            serde_json::from_str(&resp.text().await?)?
        } else {
            return Err(resp.text().await?.into()); // Black magic conversion from Err(Box::new("my error msg")) which does not
        };

        // println!("\nBOS SESSIONS:\n{:#?}", json_response);

        Ok(json_response.as_array().unwrap_or(&Vec::new()).to_vec())
    }
}

/* pub mod utils {

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
} */
