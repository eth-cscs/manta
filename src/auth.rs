use serde_json::Value;

use std::{collections::HashMap};

pub async fn auth(shasta_admin_pwd: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

    // socks5 proxy
    let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
    let mut params = HashMap::new();
    params.insert("grant_type", "client_credentials");
    params.insert("client_id", "admin-client");
    params.insert("client_secret", shasta_admin_pwd);
    
    // rest client to authenticate
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .proxy(socks5proxy)
        .build()?;
        
    let resp = client
        .post("https://api-gw-service-nmn.local/keycloak/realms/shasta/protocol/openid-connect/token")
        .form(&params)
        .send()
        .await?;
           
    let json_response: Value = serde_json::from_str(&resp.text().await?)?;

    Ok(json_response)
}