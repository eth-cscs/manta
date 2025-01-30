use std::collections::HashMap;

use backend_dispatcher::contracts::BackendTrait;
use config::{Config, Value};

use crate::backend_dispatcher::StaticBackendDispatcher;

/// Prints Manta's configuration on screen
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token_opt: Option<String>,
    /* _shasta_base_url: &str,
    _shasta_root_cert: &[u8], */
    settings: &Config,
) {
    // Read configuration file
    let log_level = settings.get_string("log").unwrap_or("error".to_string());
    let settings_hsm_group = settings.get_string("hsm_group").unwrap_or("".to_string());
    let settings_parent_hsm_group = settings
        .get_string("parent_hsm_group")
        .unwrap_or("".to_string());

    // let hsm_group_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;
    let hsm_group_available_opt = if let Some(shasta_token) = shasta_token_opt {
        backend.get_group_name_available(&shasta_token).await.ok()
    } else {
        None
    };

    let site_table: HashMap<String, Value> = settings.get_table("sites").unwrap();

    // println!("\n\nSites: {:#?}", site_table);

    let site_name = settings.get_string("site").unwrap();

    // println!("\n\nsite:\n{:#?}", site);

    // Print configuration file content to stdout
    println!("Log level: {}", log_level);
    println!("Sites: {:?}", site_table.keys().collect::<Vec<&String>>());
    println!("Current site: {}", site_name);
    println!(
        "HSM available: {}",
        hsm_group_available_opt
            .unwrap_or(vec!["Could not get list of groups available".to_string()])
            .join(", ")
    );
    println!("Current HSM: {}", settings_hsm_group);
    println!("Parent HSM: {}", settings_parent_hsm_group);
}

/* #[deprecated(
    since = "v1.54-beta.5",
    note = "use method 'StaticBackendDispatcher.get_hsm_name_available' instead"
)]
pub async fn get_hsm_name_available_from_jwt_or_all(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
) -> Vec<String> {
    log::debug!("Get HSM names available from JWT or all");
    let mut realm_access_role_vec =
        mesa::common::jwt_ops::get_hsm_name_available(shasta_token).unwrap_or(Vec::new());

    realm_access_role_vec
        .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

    if !realm_access_role_vec.is_empty() {
        //FIXME: Get rid of this by making sure CSM admins don't create HSM groups for system
        //wide operations instead of using roles
        let mut realm_access_role_filtered_vec =
            mesa::hsm::group::hacks::filter_system_hsm_group_names(realm_access_role_vec);

        realm_access_role_filtered_vec.sort();

        log::debug!(
            "HSM groups available from JWT: {:?}",
            realm_access_role_filtered_vec
        );

        realm_access_role_filtered_vec
    } else {
        let mut all_hsm_groups =
            hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
                .await
                .unwrap()
                .iter()
                .map(|hsm_value| hsm_value.label.clone())
                .collect::<Vec<String>>();

        all_hsm_groups.sort();

        log::debug!(
            "User has access to all HSM group available: {:?}",
            all_hsm_groups
        );

        all_hsm_groups
    }
} */

/* #[deprecated(note = "use method 'StaticBackendDispatcher.get_hsm_name_available' instead")]
pub async fn get_hsm_name_available_from_jwt(shasta_token: &str) -> Vec<String> {
    let mut realm_access_role_vec =
        mesa::common::jwt_ops::get_hsm_name_available(shasta_token).unwrap_or(Vec::new());

    /* let mut realm_access_role_vec = get_claims_from_jwt_token(shasta_token)
    .unwrap()
    .pointer("/realm_access/roles")
    .unwrap_or(&serde_json::json!([]))
    .as_array()
    .unwrap_or(&Vec::new())
    .iter()
    .map(|role_value| role_value.as_str().unwrap().to_string())
    .collect::<Vec<String>>(); */

    realm_access_role_vec
        .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

    realm_access_role_vec.sort();
    realm_access_role_vec
} */
