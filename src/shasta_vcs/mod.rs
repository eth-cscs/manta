
pub mod http_client {
    
    use serde_json::Value;

    pub async fn get(repo_url: &str, commitid: &str, gitea_token: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        let gitea_base_url = "https://api-gw-service-nmn.local/vcs/";
        let gitea_api_base_url = format!("{}{}", gitea_base_url, "api/v1");

        let repo_name = repo_url.trim_start_matches(gitea_base_url).trim_end_matches(".git");

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // rest client get commit details
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;

        let resp = client
            .get(format!("{}/repos/{}/git/commits/{}", gitea_api_base_url, repo_name, commitid))
            .header("Authorization", format!("token {}", gitea_token))
            .send()
            .await?;

        let json_response: Value = serde_json::from_str(&resp.text().await?)?;

        Ok(json_response)
    }

    pub async fn get_last_commitid(repo_name: &str, gitea_token: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        let gitea_base_url = "https://api-gw-service-nmn.local/vcs/";
        let gitea_api_base_url = format!("{}{}", gitea_base_url, "api/v1");

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

        // rest client get commit details
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
        
        let resp = client
            .get(format!("{}/repos/{}/git/commits", gitea_api_base_url, repo_name))
            .header("Authorization", format!("token {}", gitea_token))
            .send()
            .await?;
        
        let mut json_response: Vec<Value> = serde_json::from_str(&resp.text().await?)?;

        // cluster_cfs_configs.sort_by(|a, b| a["lastUpdated"].to_string().cmp(&b["lastUpdated"].to_string()));
        json_response.sort_by(|a, b| a["commit"]["committer"]["date"].to_string().cmp(&b["commit"]["committer"]["date"].to_string()));
        
        Ok(json_response.last().unwrap().clone())
    }
}