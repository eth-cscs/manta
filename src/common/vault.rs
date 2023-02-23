pub mod http_client {

    use serde_json::{json, Value};

    use crate::config;

    pub async fn auth() -> core::result::Result<String, Box<dyn std::error::Error>> {
        let settings = config::get("config");

        let vault_base_url = settings.get::<String>("vault_base_url").unwrap(); // TODO: move this to an env (which is readden from a config file?)

        // to get role-id run cli --> vault read auth/approle/role/manta/role-id
        let role_id = settings.get::<String>("vault_role_id").unwrap(); // TODO: move this to an env (which is readden from a config file?)

        // rest client create new cfs sessions
        let client = reqwest::Client::builder().build()?;

        let mut api_url = vault_base_url;
        api_url.push_str("/v1/auth/approle/login");

        log::debug!("Accessing/login to {}", api_url);

        let resp = client
            .post(api_url.clone())
            // .post(format!("{}{}", vault_base_url, "/v1/auth/approle/login"))
            .json(&json!({ "role_id": role_id }))
            .send()
            .await?;

        if resp.status().is_success() {
            log::debug!("Login to {} successful", api_url);
            let resp_text: Value = serde_json::from_str(&resp.text().await?)?;
            Ok(String::from(
                resp_text["auth"]["client_token"].as_str().unwrap(),
            ))
        } else {
            eprintln!("{:?}", resp);
            Err(resp.json::<Value>().await?["errors"][0]
                .as_str()
                .unwrap()
                .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    pub async fn fetch_secret(
        auth_token: &str,
        vault_base_url: &str,
        secret_path: &str,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {
        // rest client create new cfs sessions
        let client = reqwest::Client::builder().build()?;

        let mut api_url = vault_base_url.clone().to_string();
        api_url.push_str(secret_path);

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", vault_base_url, secret_path))
            .header("X-Vault-Token", auth_token)
            .send()
            .await?;

        if resp.status().is_success() {
            let resp_text: Value = serde_json::from_str(&resp.text().await?)?;
            Ok(resp_text["data"].clone()) // TODO: investigate why this ugly clone in here
        } else {
            let resp_text: Value = serde_json::from_str(&resp.text().await?)?;
            Err(resp_text["errors"][0].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }

    pub async fn fetch_shasta_vcs_token(
        vault_base_url: &str,
    ) -> core::result::Result<String, Box<dyn std::error::Error>> {
        let vault_token = auth().await;

        match vault_token {
            Ok(_) => {
                let vault_secret =
                    fetch_secret(&vault_token?, vault_base_url, "/v1/shasta/vcs").await?; // this works for hashicorp-vault for fulen may need /v1/secret/data/shasta/vcs
                Ok(String::from(vault_secret["token"].as_str().unwrap())) // this works for vault v1.12.0 for older versions may need vault_secret["data"]["token"]
            }
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
    }

    pub async fn fetch_shasta_k8s_secrets(
        vault_base_url: &str,
    ) -> core::result::Result<Value, Box<dyn std::error::Error>> {
        let vault_token = auth().await;

        match vault_token {
            Ok(_) => {
                let vault_secret =
                    fetch_secret(&vault_token?, vault_base_url, "/v1/shasta/k8s").await?; // this works for hashicorp-vault for fulen may need /v1/secret/data/shasta/k8s
                Ok(serde_json::from_str(
                    vault_secret["value"].as_str().unwrap(),
                )?) // this works for vault v1.12.0 for older versions may need vault_secret["data"]["value"]
            }
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
    }
}
