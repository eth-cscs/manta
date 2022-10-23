
pub mod http_client {
    
    use serde_json::Value;

    pub async fn get_commit_details(repo_url: &str, commitid: &str, gitea_token: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        let gitea_base_url = "https://api-gw-service-nmn.local/vcs/";
        let gitea_api_base_url = format!("{}{}", gitea_base_url, "api/v1");

        let repo_name = repo_url.trim_start_matches(gitea_base_url).trim_end_matches(".git");

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
            .get(format!("{}/repos/{}/git/commits/{}", gitea_api_base_url, repo_name, commitid))
            .header("Authorization", format!("token {}", gitea_token))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(serde_json::from_str(&resp.text().await?)?)
        } else {
            Err(resp.json::<Value>().await?["detail"].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    }

    pub async fn get_last_commit(repo_name: &str, gitea_token: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        let gitea_base_url = "https://api-gw-service-nmn.local/vcs/";
        let gitea_api_base_url = format!("{}{}", gitea_base_url, "api/v1");

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
            .get(format!("{}/repos/{}/commits", gitea_api_base_url, repo_name))
            .header("Authorization", format!("token {}", gitea_token))
            .send()
            .await?;
        
        let mut json_response: Vec<Value> = serde_json::from_str(&resp.text().await?)?;

        // cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].to_string().cmp(&b["lastUpdated"].to_string()));
        json_response.sort_by(|a, b| a["commit"]["committer"]["date"].to_string().cmp(&b["commit"]["committer"]["date"].to_string()));
        
        Ok(json_response.last().unwrap().clone())
    }
}