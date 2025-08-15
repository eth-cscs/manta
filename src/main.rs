mod cli;
mod common;
mod manta_backend_dispatcher;

use ::manta_backend_dispatcher::types::K8sAuth;
use common::{config::types::MantaConfiguration, kafka::Kafka};
use manta_backend_dispatcher::StaticBackendDispatcher;

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

  let settings = common::config::get_configuration().await;

  let configuration: MantaConfiguration =
    settings.clone().try_deserialize().unwrap_or_else(|e| {
      eprintln!("ERROR - Configuration file is not valid: {}", e);
      std::process::exit(1);
    });

  let site_name: String = configuration.site.clone();
  let site_detail_value = configuration.sites.get(&site_name).unwrap();

  let backend_tech = &site_detail_value.backend;
  let shasta_base_url = &site_detail_value.shasta_base_url;
  let shasta_barebone_url = shasta_base_url // HACK to not break compatibility with
    // old configuration file. TODO: remove this when needed in the future and all users are
    // using the right configuration file
    .strip_suffix("/apis")
    .unwrap_or(&shasta_base_url);
  let shasta_api_url = match backend_tech.as_str() {
    "csm" => shasta_barebone_url.to_owned() + "/apis",
    "ochami" => shasta_barebone_url.to_owned(),
    _ => {
      eprintln!("ERROR - Invalid backend technology: {}", backend_tech);
      std::process::exit(1);
    }
  };
  log::debug!("config - shasta_api_url:  {shasta_api_url}");
  let gitea_base_url = shasta_barebone_url.to_owned() + "/vcs";
  log::debug!("config - gitea_base_url:  {gitea_base_url}");
  let k8s_api_url: Option<&String> = site_detail_value
    .k8s
    .as_ref()
    .map(|k8s_details| &k8s_details.api_url);
  log::debug!("config - k8s_api_url:  {k8s_api_url:?}");
  let vault_base_url =
    site_detail_value
      .k8s
      .as_ref()
      .and_then(|k8s| match &k8s.authentication {
        K8sAuth::Vault { base_url, .. } => Some(base_url),
        K8sAuth::Native { .. } => None,
      });
  log::debug!("config - vault_base_url:  {vault_base_url:?}");

  // let audit_detail = settings.get_table("audit").unwrap();
  let audit_detail = configuration.auditor.clone();
  let audit_kafka_opt: Option<Kafka> = if audit_detail.is_some() {
    Some(audit_detail.unwrap().kafka)
  } else {
    None
  };

  let log_level = settings.get_string("log").unwrap_or("error".to_string());
  log::debug!("config - log_level:  {log_level}");

  if audit_kafka_opt.is_none() {
    log::warn!("config - Auditor not defined");
  }

  let audit_file_path =
    if let Ok(audit_file) = settings.get_string("audit_file") {
      audit_file
    } else {
      "/var/log/manta/requests.log".to_string()
    };
  log::debug!("config - audit_file_path:  {audit_file_path}");

  log_ops::configure(log_level, audit_file_path.as_str()); // log4rs programatically configuration

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

  let root_ca_cert_file = &site_detail_value.root_ca_cert_file;

  log::debug!("config - root_ca_cert_file:  {root_ca_cert_file}");

  let shasta_root_cert_rslt =
    common::config::get_csm_root_cert_content(&root_ca_cert_file);

  let shasta_root_cert = if let Ok(shasta_root_cert) = shasta_root_cert_rslt {
    shasta_root_cert
  } else {
    eprintln!(
      "ERROR - CA public root file '{}' not found. Exit",
      root_ca_cert_file
    );
    std::process::exit(1);
  };

  let backend = StaticBackendDispatcher::new(
    &backend_tech,
    &shasta_api_url,
    &shasta_root_cert,
  );

  // Process input params
  let cli = crate::cli::build::build_cli();

  let cli_result = crate::cli::process::process_cli(
    cli,
    backend,
    &shasta_api_url,
    &shasta_root_cert,
    vault_base_url,
    &gitea_base_url,
    settings_hsm_group_name_opt.as_ref(),
    k8s_api_url,
    audit_kafka_opt.as_ref(),
    &settings,
    &configuration,
  )
  .await;

  match cli_result {
    Ok(_) => Ok(()),
    Err(e) => panic!("{}", e),
  }
}
