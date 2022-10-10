pub struct 

mod http_client {
    mod node_shutdown {
        pub fn post()  -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {
                    // socks5 proxy
            let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

            // rest client create new cfs sessions
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .proxy(socks5proxy)
                .build()?;
        
            let resp = client
                .post("http://api-gw-service-nmn.local/apis/capmc/capmc/v1/xname_off")
                .bearer_auth(shasta_token)
                .json() // TODO: DYNAMIC JSON WITH SERDE JSON
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(serde_json::from_str(&resp.text().await?)?)
            } else {
                Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
            }
        }
    }
}