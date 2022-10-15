pub mod operations {

    use std::error::Error;

    use crate::shasta_cfs_component::http_client::get;

    pub async fn is_component_scheduled_for_configuration(shasta_token: &str, shasta_base_url: &str, component_id: &String) -> Result<bool, Box<dyn Error>> {

        let json_response = get(shasta_token, shasta_base_url, component_id).await?;

        Ok(json_response["enabled"].as_bool().unwrap())
    }
}

pub mod http_client {

    use std::error::Error;

    use serde_json::Value;

    pub async fn get(shasta_token: &str, shasta_base_url: &str, component_id: &String) -> Result<Value, Box<dyn Error>> {
    
        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // rest client get cfs configurations
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
    
        let resp = client
            .get(format!("{}{}{}", shasta_base_url, "/cfs/v2/components/", component_id))
            .bearer_auth(shasta_token)
            .send()
            .await?
            .text()
            .await?;
    
        let json_response: Value = serde_json::from_str(&resp)?;
    
        Ok(json_response)
    }
}