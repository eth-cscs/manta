use crate::common::config_ops;

/// Prints Manta's configuration on screen
pub async fn exec(shasta_token: &str, shasta_base_url: &str) {
    // Read configuration file
    let settings = config_ops::get_configuration();

    /* let shasta_base_url = settings.get_string("shasta_base_url").unwrap();
    let vault_base_url = settings.get_string("vault_base_url").unwrap();
    let vault_role_id = settings.get_string("vault_role_id").unwrap();
    let vault_secret_path = settings.get_string("vault_secret_path").unwrap();
    let gitea_base_url = settings.get_string("gitea_base_url").unwrap();
    let keycloak_base_url = settings.get_string("keycloak_base_url").unwrap();
    let k8s_api_url = settings.get_string("k8s_api_url").unwrap();
    let log_level = settings.get_string("log").unwrap_or("error".to_string()); */
    let settings_hsm_group = settings.get_string("hsm_group").unwrap_or("".to_string());
    let settings_hsm_group_available_value_rslt = settings.get_array("hsm_available");

    let hsm_group_available: String =
        if let Ok(hsm_group_available_value) = settings_hsm_group_available_value_rslt {
            hsm_group_available_value
                .into_iter()
                .map(|hsm_group| hsm_group.into_string().unwrap())
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            mesa::shasta::hsm::http_client::get_all_hsm_groups(shasta_token, shasta_base_url)
                .await
                .unwrap()
                .iter()
                .map(|hsm_value| hsm_value["label"].as_str().unwrap().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        };

    // Print configuration file content to stdout
    /* println!("Shasta base URL: {}", shasta_base_url);
    println!("Vault base URL: {}", vault_base_url);
    println!("Vault role: {}", vault_role_id);
    println!("Vault secret path: {}", vault_secret_path);
    println!("Gitea base URL: {}", gitea_base_url);
    println!("Keycloak base URL: {}", keycloak_base_url);
    println!("Kubernetes api URL: {}", k8s_api_url);
    println!("Log: {}", log_level); */
    println!("HSM available: {}", hsm_group_available);
    println!("Current HSM: {}", settings_hsm_group);
}
