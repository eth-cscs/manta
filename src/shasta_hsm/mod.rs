

pub mod http_client {

    use std::error::Error;

    use serde_json::Value;


    pub async fn get_component_status(shasta_token: &str, shasta_base_url: &str, xname: &str) -> Result<Value, Box<dyn Error>> {

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // rest client get commit details
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;

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