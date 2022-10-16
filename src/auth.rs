use directories::ProjectDirs;
use serde_json::Value;

use dialoguer::{Input, Password};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
};

/// docs --> https://cray-hpe.github.io/docs-csm/en-12/operations/security_and_authentication/api_authorization/
///      --> https://cray-hpe.github.io/docs-csm/en-12/operations/security_and_authentication/retrieve_an_authentication_token/
pub async fn get_shasta_api_token() -> Result<String, Box<dyn Error>> {

    let mut file;
    let mut shasta_token = "".to_string();

    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut path = PathBuf::from(project_dirs.unwrap().cache_dir());

    create_dir_all(&path)?;

    path.push("http");

    log::debug!("Cache file: {:?}", path);

    let mut attempts = 0;

    while path.exists() && fs::metadata(&path)?.len() == 0 && attempts < 3 {

        log::info!("Please type your Keycloak credentials");
        let username: String = Input::new().with_prompt("username").interact_text()?;

        let password = Password::new().with_prompt("password").interact()?;

        match auth(&username, &password).await {
            Ok(shasta_token) => {
                file = File::create(&path).expect("Error encountered while creating file!");
                file.write_all(shasta_token.as_bytes())
                    .expect("Error while writing to file");
            },
            Err(_) => {
                attempts += 1;
            }
        }
    }

    if path.exists() && fs::metadata(&path)?.len() > 0 {
        File::open(path).unwrap().read_to_string(&mut shasta_token);
        Ok(shasta_token.to_string())
    } else {
        Err("Authentication unsucessful".into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
    }
}

pub async fn auth(username: &str, password: &str) -> Result<String, Box<dyn Error>> {
    
    let json_response: Value;

    // socks5 proxy
    let socks5proxy = reqwest::Proxy::all("socks5h://127.0.0.1:1080")?;

    let mut params = HashMap::new();
    params.insert("grant_type", "password");
    params.insert("client_id", "shasta");
    params.insert("username", &username);
    params.insert("password", &password);
    // params.insert("grant_type", "client_credentials");
    // params.insert("client_id", "admin-client");
    // params.insert("client_secret", shasta_admin_pwd);

    // rest client to authenticate
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .proxy(socks5proxy)
        .build()?;

    let resp = client
        .post(
            "https://api-gw-service-nmn.local/keycloak/realms/shasta/protocol/openid-connect/token",
        )
        .form(&params)
        .send()
        .await?;

    if resp.status().is_success() {
        json_response = serde_json::from_str(&resp.text().await?)?;
        Ok(json_response["access_token"].as_str().unwrap().to_string())
    } else {
        Err(resp.json::<Value>().await?["detail"]
            .as_str()
            .unwrap()
            .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
    }
}
