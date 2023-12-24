use std::collections::HashMap;

use config::{Config, Value};

use crate::common::jwt_ops;

/// Prints Manta's configuration on screen
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    settings: &Config,
) {
    // Read configuration file
    // let settings = config_ops::get_configuration();

    /* let shasta_base_url = settings.get_string("shasta_base_url").unwrap();
    let vault_base_url = settings.get_string("vault_base_url").unwrap();
    let vault_role_id = settings.get_string("vault_role_id").unwrap();
    let vault_secret_path = settings.get_string("vault_secret_path").unwrap();
    let gitea_base_url = settings.get_string("gitea_base_url").unwrap();
    let keycloak_base_url = settings.get_string("keycloak_base_url").unwrap();
    let k8s_api_url = settings.get_string("k8s_api_url").unwrap();
    let log_level = settings.get_string("log").unwrap_or("error".to_string()); */
    let settings_hsm_group = settings.get_string("hsm_group").unwrap_or("".to_string());
    // let settings_hsm_group_available_value_rslt = settings.get_array("hsm_available");

    /* let mut realm_access_role_vec = jwt_ops::get_claims_from_jwt_token(&shasta_token)
        .unwrap()
        .pointer("/realm_access/roles")
        .unwrap()
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|role_value| role_value.as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    realm_access_role_vec
        .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

    // println!("JWT token resour_access:\n{:?}", realm_access_role_vec);

    let settings_hsm_available_vec = realm_access_role_vec; */

    let hsm_group_available: Vec<String> =
        get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await;

    let site_table: HashMap<String, Value> = settings.get_table("sites").unwrap();

    // println!("\n\nSites: {:#?}", site_table);

    let site_name = settings.get_string("site").unwrap();

    // let site = site_table.get(&site_name);

    // println!("\n\nsite:\n{:#?}", site);

    // Print configuration file content to stdout
    /* println!("Shasta base URL: {}", shasta_base_url);
    println!("Vault base URL: {}", vault_base_url);
    println!("Vault role: {}", vault_role_id);
    println!("Vault secret path: {}", vault_secret_path);
    println!("Gitea base URL: {}", gitea_base_url);
    println!("Keycloak base URL: {}", keycloak_base_url);
    println!("Kubernetes api URL: {}", k8s_api_url);
    println!("Log: {}", log_level); */
    println!("Sites: {:?}", site_table.keys().collect::<Vec<&String>>());
    println!("Current site: {}", site_name);
    println!("HSM available: {:?}", hsm_group_available);
    println!("Current HSM: {}", settings_hsm_group);
}

pub async fn get_hsm_name_available_from_jwt_or_all(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
) -> Vec<String> {
    let mut realm_access_role_vec = jwt_ops::get_claims_from_jwt_token(shasta_token)
        .unwrap()
        .pointer("/realm_access/roles")
        .unwrap()
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|role_value| role_value.as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    realm_access_role_vec
        .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

    if !realm_access_role_vec.is_empty() {
        realm_access_role_vec
    } else {
        mesa::hsm::http_client::get_all_hsm_groups(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .unwrap()
            .iter()
            .map(|hsm_value| hsm_value["label"].as_str().unwrap().to_string())
            .collect::<Vec<String>>()
    }
}
