pub mod http_client {

    use std::error::Error;

    use serde_json::Value;

    pub async fn get(
        shasta_token: &str,
        shasta_base_url: &str,
        component_id: &str,
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

        let api_url = shasta_base_url.to_owned() + "/cfs/v2/components/" + component_id;

        let resp = client
            .get(api_url)
            // .get(format!("{}{}{}", shasta_base_url, "/cfs/v2/components/", component_id))
            .bearer_auth(shasta_token)
            .send()
            .await?
            .text()
            .await?;

        let json_response: Value = serde_json::from_str(&resp)?;

        Ok(json_response)
    }
}

