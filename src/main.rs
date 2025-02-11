mod backend_dispatcher;
mod cli;
mod common;

use backend_dispatcher::StaticBackendDispatcher;
use common::{config_ops::MantaConfiguration, kafka::Kafka};

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

    let settings = common::config_ops::get_configuration().await;

    let configuration: MantaConfiguration =
        settings.clone().try_deserialize().unwrap_or_else(|e| {
            eprintln!("ERROR - Configuration file is not valid: {}", e);
            std::process::exit(1);
        });

    let site_name: String = configuration.site.clone();
    /* let site_name = settings
    .get_string("site")
    .expect("ERROR - 'site' value missing in configuration file"); */
    /* let site_detail_hashmap = settings
    .get_table("sites")
    .expect("ERROR - 'site' list missing in configuration file"); */
    /* let site_detail_value = site_detail_hashmap
    .get(&site_name)
    .unwrap()
    .clone()
    .into_table()
    .expect(format!("ERROR - site '{}' missing in configuration file", site_name).as_str()); */
    let site_detail_value = configuration.sites.get(&site_name).unwrap();

    /* let backend_tech = site_detail_value
    .get("backend")
    .expect("ERROR - 'backend' value missing in configuration file")
    .to_string(); */
    let backend_tech = &site_detail_value.backend;
    /* let shasta_base_url = site_detail_value
    .get("shasta_base_url")
    .expect("ERROR - 'shasta_base_url' value missing in configuration file")
    .to_string(); */
    let shasta_base_url = &site_detail_value.shasta_base_url;
    let shasta_barebone_url = shasta_base_url // HACK to not break compatibility with
        // old configuration file. TODO: remove this when needed in the future and all users are
        // using the right configuration file
        .strip_suffix("/apis")
        .unwrap_or(&shasta_base_url);
    // let shasta_api_url = shasta_barebone_url.to_owned() + "/apis";
    let shasta_api_url = shasta_barebone_url.to_owned() + "/apis";
    log::debug!("config - shasta_api_url:  {shasta_api_url}");
    // let gitea_base_url = shasta_barebone_url.to_owned() + "/vcs";
    let gitea_base_url = shasta_barebone_url.to_owned() + "/vcs";
    log::debug!("config - gitea_base_url:  {gitea_base_url}");
    /* let k8s_api_url = site_detail_value
    .get("k8s_api_url")
    .expect("ERROR - 'k8s_api_url' value missing in configuration file")
    .to_string(); */
    let k8s_api_url = &site_detail_value.k8s_api_url;
    log::debug!("config - k8s_api_url:  {k8s_api_url}");
    /* let vault_base_url = site_detail_value
    .get("vault_base_url")
    .expect("ERROR - 'vault_base_url' value missing in configuration file")
    .to_string(); */
    let vault_base_url = &site_detail_value.vault_base_url;
    log::debug!("config - vault_base_url:  {vault_base_url}");
    /* let vault_role_id = site_detail_value
    .get("vault_role_id")
    .expect("ERROR - 'vault_role_id' value missing in configuration file")
    .to_string(); */
    let vault_role_id = &site_detail_value.vault_role_id;
    log::debug!("config - vault_role_id:  {vault_role_id}");
    /* let vault_secret_path = site_detail_value
    .get("vault_secret_path")
    .unwrap()
    .to_string(); */
    let vault_secret_path = &site_detail_value.vault_secret_path;

    // let audit_detail = settings.get_table("audit").unwrap();
    let audit_detail = configuration.auditor.clone();
    /* let audit_kafka_detail_hashmap = audit_detail
    .get("kafka")
    .expect("kafka value missing in configuration file")
    .clone()
    .into_table()
    .expect("kafka value must be a table"); */
    let audit_kafka: Kafka = audit_detail
        .kafka
        .expect("kafka value missing in configuration file");
    /* let kafka_audit = Kafka {
        brokers: audit_kafka_detail_hashmap
            .get("brokers")
            .cloned()
            .expect("'brokers' value missing in configuration file")
            .into_array()
            .expect("brokers value must be an array")
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        topic: audit_kafka_detail_hashmap
            .clone()
            .get("topic")
            .expect("topic value missing in configuration file")
            .to_string(),
    }; */

    let log_level = settings.get_string("log").unwrap_or("error".to_string());
    log::debug!("config - log_level:  {log_level}");

    let audit_file_path = if let Ok(audit_file) = settings.get_string("audit_file") {
        audit_file
    } else {
        "/var/log/manta/requests.log".to_string()
    };
    log::debug!("config - audit_file_path:  {audit_file_path}");

    log_ops::configure(log_level, audit_file_path.as_str()); // log4rs programatically configuration

    /* if let Some(socks_proxy) = site_detail_value.get("socks5_proxy") {
        let socks_proxy = socks_proxy.to_string();
        if !socks_proxy.is_empty() {
            std::env::set_var("SOCKS5", socks_proxy.clone());
            log::info!("SOCKS5 enabled: {:?}", std::env::var("SOCKS5"));
            log::debug!("config - socks_proxy:  {socks_proxy}");
        } else {
            log::debug!("config - socks_proxy:  Not defined");
        }
    } */
    if let Some(socks_proxy) = &site_detail_value.socks5_proxy {
        let socks_proxy = socks_proxy.to_string();
        if !socks_proxy.is_empty() {
            std::env::set_var("SOCKS5", socks_proxy.clone());
            log::info!("SOCKS5 enabled: {:?}", std::env::var("SOCKS5"));
            log::debug!("config - socks_proxy:  {socks_proxy}");
        } else {
            log::debug!("config - socks_proxy:  Not defined");
        }
    }

    let settings_hsm_group_name_opt = settings.get_string("hsm_group").ok();

    /* let root_ca_cert_file = site_detail_value
    .get("root_ca_cert_file")
    .expect("ERROR - 'root_ca_cert_file' value missing in configuration file")
    .to_string(); */

    let root_ca_cert_file = &site_detail_value.root_ca_cert_file;

    log::debug!("config - root_ca_cert_file:  {root_ca_cert_file}");

    let shasta_root_cert_rslt = common::config_ops::get_csm_root_cert_content(&root_ca_cert_file);

    let shasta_root_cert = if let Ok(shasta_root_cert) = shasta_root_cert_rslt {
        shasta_root_cert
    } else {
        eprintln!(
            "ERROR - CA public root file '{}' not found. Exit",
            root_ca_cert_file
        );
        std::process::exit(1);
    };

    let backend = StaticBackendDispatcher::new(&backend_tech, &shasta_api_url, &shasta_root_cert);

    // Process input params
    let cli = crate::cli::build::build_cli();

    let cli_result = crate::cli::process::process_cli(
        cli,
        backend,
        &shasta_api_url,
        &shasta_root_cert,
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        &gitea_base_url,
        settings_hsm_group_name_opt.as_ref(),
        k8s_api_url,
        &audit_kafka,
        &settings,
        &configuration,
    )
    .await;

    match cli_result {
        Ok(_) => Ok(()),
        Err(e) => panic!("{}", e),
    }
}
