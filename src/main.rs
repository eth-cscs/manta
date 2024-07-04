mod cli;
mod common;

use crate::common::log_ops;

// DHAT (profiling)
// #[cfg(feature = "dhat-heap")]
// #[global_allocator]
// static ALOC: dhat::Alloc = dhat::Alloc;

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
    //println!("async main");
    // DHAT (profiling)
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();

    let settings = common::config_ops::get_configuration();

    let site_name = settings.get_string("site").unwrap();
    let site_detail_hashmap = settings.get_table("sites").unwrap();
    let site_detail_value = site_detail_hashmap
        .get(&site_name)
        .unwrap()
        .clone()
        .into_table()
        .unwrap();

    let shasta_base_url = site_detail_value
        .get("shasta_base_url")
        .unwrap()
        .to_string();
    let vault_base_url = site_detail_value
        .get("vault_base_url")
        .expect("vault_base_url value missing in configuration file")
        .to_string();
    let vault_role_id = site_detail_value
        .get("vault_role_id")
        .expect("vault_role_id value missing in configuration file")
        .to_string();
    let vault_secret_path = site_detail_value
        .get("vault_secret_path")
        .unwrap()
        .to_string();
    let gitea_base_url = site_detail_value
        .get("gitea_base_url")
        .expect("gitea_base_url value missing in configuration file")
        .to_string();
    let keycloak_base_url = site_detail_value
        .get("keycloak_base_url")
        .expect("keycloak_base_url value missing in configuration file")
        .to_string();
    let k8s_api_url = site_detail_value
        .get("k8s_api_url")
        .expect("k8s_api_url value missing in configuration file")
        .to_string();

    let log_level = settings.get_string("log").unwrap_or("error".to_string());

    let audit_file_path = if let Ok(audit_file) = settings.get_string("audit_file") {
        audit_file
    } else {
        "/var/log/manta/requests.log".to_string()
    };

    log_ops::configure(log_level, audit_file_path.as_str()); // log4rs programatically configuration

    if let Some(socks_proxy) = site_detail_value.get("socks5_proxy") {
        std::env::set_var("SOCKS5", socks_proxy.to_string());
        log::info!("SOCKS5 enabled: {:?}", std::env::var("SOCKS5"));
    }

    let settings_hsm_group_name_opt = settings.get_string("hsm_group").ok();

    let root_ca_cert_file = site_detail_value
        .get("root_ca_cert_file")
        .expect("'root_cert_file' value missing in configuration file")
        .to_string();

    let shasta_root_cert = common::config_ops::get_csm_root_cert_content(&root_ca_cert_file);

    let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
        &vault_base_url,
        &vault_secret_path,
        &vault_role_id,
    )
    .await
    .unwrap();

    // Process input params
    let matches = crate::cli::build::build_cli(settings_hsm_group_name_opt.as_ref()).get_matches();

    let cli_result = crate::cli::process::process_cli(
        matches,
        &keycloak_base_url,
        &shasta_base_url,
        &shasta_root_cert,
        &vault_base_url,
        &vault_secret_path,
        &vault_role_id,
        &gitea_token,
        &gitea_base_url,
        settings_hsm_group_name_opt.as_ref(),
        // settings_hsm_available_vec,
        // &site_available_vec,
        // &base_image_id,
        &k8s_api_url,
        &settings,
    )
    .await;

    match cli_result {
        Ok(_) => Ok(()),
        Err(e) => panic!("{}", e),
    }
}
