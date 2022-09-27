pub mod http_client {

    use serde_json::Value;

    pub async fn get(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, session_name: &Option<String>, limit_number: &Option<u8>) -> core::result::Result<Vec<Value>, Box<dyn std::error::Error>> {

        let mut cluster_cfs_sessions: Vec<Value> = Vec::new();

        // socks5 proxy
        let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;
    
        // rest client get cfs sessions
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(socks5proxy)
            .build()?;
    
        let resp = client
            .get(format!("{}{}", shasta_base_url, "/cfs/v2/sessions"))
            .bearer_auth(shasta_token)
            .send()
            .await?
            .text()
            .await?;
            
        let json_response: Value = serde_json::from_str(&resp)?;
    
        if cluster_name.is_some() {
            for cfs_session in json_response.as_array().unwrap() {
    
                if cfs_session["configuration"]["name"]
                    .as_str()
                    .unwrap()
                    .contains(cluster_name.as_ref().unwrap()) // TODO: investigate why I need to use this ugly 'as_ref'
                {
                    cluster_cfs_sessions.push(cfs_session.clone());
                }
            }
        } else if session_name.is_some() {
            for cfs_session in json_response.as_array().unwrap() {
                if cfs_session
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq(session_name.as_ref().unwrap()) // TODO: investigate why I need to us this ugly 'as_ref'
                {
                    cluster_cfs_sessions.push(cfs_session.clone());
                }
            }
        } else { // Returning all results
            cluster_cfs_sessions = json_response.as_array().unwrap().to_vec();

            cluster_cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));
        }
        
        if limit_number.is_some() { // Limiting the number of results to return to client

            cluster_cfs_sessions = json_response.as_array().unwrap().to_vec();
    
            cluster_cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));
    
            // cfs_utils::print_cfs_configurations(&cfs_configurations);
            
            // cluster_cfs_sessions.truncate(limit_number.unwrap().into());
            cluster_cfs_sessions = cluster_cfs_sessions[cluster_cfs_sessions.len().saturating_sub(limit_number.unwrap().into())..].to_vec();
            
            // cluster_cfs_sessions = vec![cluster_cfs_sessions]; // vec! macro for vector initialization!!! https://doc.rust-lang.org/std/vec/struct.Vec.html
        } 

        Ok(cluster_cfs_sessions)
    }
}