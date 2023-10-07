#![allow(dead_code, unused_imports)] // TODO: to avoid compiler from complaining about unused methods

pub mod http_client {

    use serde_json::Value;

    pub async fn get_commit_details(
        repo_url: &str,
        commitid: &str,
        gitea_base_url: &str,
        gitea_token: &str,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {
        let gitea_internal_base_url = "https://api-gw-service-nmn.local/vcs/";
        // let gitea_external_base_url = "https://api.cmn.alps.cscs.ch/vcs/";

        // let gitea_api_base_url = gitea_external_base_url.to_owned() + "api/v1";
        let gitea_api_base_url = format!("{}/api/v1", gitea_base_url);

        let repo_name = repo_url
            .trim_start_matches(gitea_internal_base_url)
            .trim_end_matches(".git");
        /* let repo_name = repo_name
            .trim_start_matches(gitea_external_base_url)
            .trim_end_matches(".git"); */

        log::info!("repo_url: {}", repo_url);
        log::info!("gitea_base_url: {}", gitea_internal_base_url);
        log::info!("repo_name: {}", repo_name);

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

        let api_url = format!(
            "{}/repos/{}/git/commits/{}",
            gitea_api_base_url, repo_name, commitid
        );

        log::info!("Request to {}", api_url);

        let resp = client
            .get(api_url)
            .header("Authorization", format!("token {}", gitea_token))
            .send()
            .await?;

        if resp.status().is_success() {
            let json_response = &resp.text().await?;

            Ok(serde_json::from_str(json_response)?)
        } else {
            let error_msg = format!("ERROR: commit {} not found in Shasta CVS. Please check gitea admin or wait sync to finish.", commitid);

            Err(error_msg.into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    pub async fn get_last_commit_from_repo_name(
        gitea_api_base_url: &str,
        repo_name: &str,
        gitea_token: &str,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {
        let repo_url = gitea_api_base_url.to_owned() + "/api/v1/repos" + repo_name + "/commits";

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
            .get(repo_url)
            .header("Authorization", format!("token {}", gitea_token))
            .send()
            .await?;

        if resp.status().is_success() {
            let mut json_response: Vec<Value> = serde_json::from_str(&resp.text().await?)?;
            json_response.sort_by(|a, b| {
                a["commit"]["committer"]["date"]
                    .to_string()
                    .cmp(&b["commit"]["committer"]["date"].to_string())
            });

            println!("last commit: {:#?}", json_response.last().unwrap().clone());

            Ok(json_response.last().unwrap().clone())
        } else {
            eprintln!("FAIL request: {:#?}", resp);
            let response: String = resp.text().await?;
            eprintln!("FAIL response: {:#?}", response);
            Err(response.into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    pub async fn get_last_commit_from_url(
        gitea_api_base_url: &str,
        repo_url: &str,
        gitea_token: &str,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {
        let repo_name = repo_url
            .trim_start_matches("https://api-gw-service-nmn.local/vcs/")
            .trim_end_matches(".git");

        get_last_commit_from_repo_name(gitea_api_base_url, repo_name, gitea_token).await
    }
}
