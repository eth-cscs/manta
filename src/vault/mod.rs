// use serde::{Serialize, Deserialize};

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Auth {
//     role_id: String
// }

// impl Auth {
//     pub fn new(role_id: &str) -> Self {
//         Self {
//             role_id: String::from(role_id)
//         }
//     }
// }

pub mod http_client {

    use serde_json::{Value, json};

    // use super::Auth;

    use crate::config;

    pub async fn auth() -> core::result::Result<String, Box<dyn std::error::Error>> {

        let settings = config::get("config");

        // let shasta_base_url = settings.get::<String>("shasta_base_url").unwrap();

        // let vault_server_address = std::env::var("VAULT_ADDR").expect("vault address not defined. Please check your configuration");
        let vault_base_url = settings.get::<String>("vault_base_url").unwrap(); // TODO: move this to an env (which is readden from a config file?)

        // to get role-id run cli --> vault read auth/approle/role/manta/role-id
        // let role_id = std::env::var("VAULT_ROLE_ID").expect("vault role id not defined. Please check your configuration");
        let role_id = settings.get::<String>("vault_role_id").unwrap(); // TODO: move this to an env (which is readden from a config file?)

        // let auth_payload = Auth::new(&role_id);

        // rest client create new cfs sessions
        let client = reqwest::Client::builder().build()?;
        
        let resp = client
            .post(format!("{}{}", vault_base_url, "/v1/auth/approle/login"))
            // .json(&auth_payload)
            .json(&json!({ "role_id": String::from(role_id)}))
            .send()
            .await?;

        if resp.status().is_success() {
            let resp_text: Value = serde_json::from_str(&resp.text().await?)?;
            Ok(String::from(resp_text["auth"]["client_token"].as_str().unwrap()))
        } else {
            Err(resp.json::<Value>().await?["errors"][0].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    }

    pub async fn fetch_secret(auth_token: &str, vault_base_url: &str, secret_path: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        // rest client create new cfs sessions
        let client = reqwest::Client::builder().build()?;
        
        let resp = client
            .get(format!("{}{}", vault_base_url, secret_path))
            .header("X-Vault-Token", auth_token)
            .send()
            .await?;

        if resp.status().is_success() {
            let resp_text: Value = serde_json::from_str(&resp.text().await?)?;
            Ok(resp_text["data"].clone()) // TODO: investigate why this ugly clone in here
        } else {
            Err(resp.json::<Value>().await?["errors"][0].as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not 
        }
    }

    pub async fn fetch_shasta_vcs_token(vault_base_url: &str) -> core::result::Result<String, Box<dyn std::error::Error>> {

        let vault_token = auth().await;

        match vault_token {
            Ok(_) => {
                let vault_secret = fetch_secret(&vault_token?, vault_base_url, "/v1/secret/data/shasta/vcs").await?;
                Ok(String::from(vault_secret["data"]["token"].as_str().unwrap()))
            },
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
    }

    pub async fn fetch_shasta_k8s_secrets(vault_base_url: &str) -> core::result::Result<Value, Box<dyn std::error::Error>> {

        let vault_token = auth().await;

        match vault_token {
            Ok(_) => {
                let vault_secret = fetch_secret(&vault_token?, vault_base_url, "/v1/secret/data/shasta/k8s").await?;
                Ok(serde_json::from_str(vault_secret["data"]["value"].as_str().unwrap())?)
            },
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        }
    }
}