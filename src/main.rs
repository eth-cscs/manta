mod backend_dispatcher;
mod cli;
mod common;
mod manta_backend_dispatcher;

use ::manta_backend_dispatcher::types::K8sAuth;
use common::{
  app_context::AppContext, config::types::MantaConfiguration, kafka::Kafka,
};
use manta_backend_dispatcher::StaticBackendDispatcher;

use crate::common::log_ops;

/// URL path suffix for the CSM API endpoint.
const API_URL_SUFFIX: &str = "/apis";

/// URL path suffix for the Gitea VCS endpoint.
const VCS_URL_SUFFIX: &str = "/vcs";

/// Synchronous entry point. Sets environment variables (which
/// must happen before the multi-threaded tokio runtime is active)
/// and then launches the async runtime.
fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
  // Build a *single-threaded* runtime just to load the config
  // file so we can read the SOCKS5 proxy value.
  let preliminary_rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;
  let settings = preliminary_rt
    .block_on(async { common::config::get_configuration().await });
  let settings = match settings {
    Ok(s) => s,
    Err(e) => {
      return Err(format!("Could not read configuration file: {}", e).into());
    }
  };
  let configuration: MantaConfiguration = settings
    .clone()
    .try_deserialize()
    .map_err(|e| format!("Configuration file is not valid: {}", e))?;
  // Drop the preliminary runtime before setting env vars — no
  // other threads are alive at this point.
  drop(preliminary_rt);

  // Set SOCKS5 proxy env var while we are still single-threaded.
  let site_name: String = configuration.site.clone();
  let site_details_value =
    configuration.sites.get(&site_name).ok_or_else(|| {
      format!("Site '{}' not found in configuration file", site_name)
    })?;
  if let Some(socks_proxy) = &site_details_value.socks5_proxy
    && !socks_proxy.is_empty()
  {
    // SAFETY: no other threads are running yet, so this is
    // sound even though `set_var` is marked unsafe since
    // Rust 1.66.
    unsafe {
      std::env::set_var("SOCKS5", socks_proxy);
    }
  }

  // Now spin up the full multi-threaded tokio runtime.
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  rt.block_on(run(settings, configuration, site_name))
}

/// Async entry point — runs on the multi-threaded tokio runtime
/// after environment variables have been safely configured.
async fn run(
  settings: config::Config,
  configuration: MantaConfiguration,
  site_name: String,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  // Configure logging
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());

  log_ops::configure(log_level)?;

  let site_details_value =
    configuration.sites.get(&site_name).ok_or_else(|| {
      format!("Site '{}' not found in configuration file", site_name)
    })?;

  if let Some(socks_proxy) = &site_details_value.socks5_proxy {
    if !socks_proxy.is_empty() {
      log::info!("SOCKS5 enabled: {:?}", std::env::var("SOCKS5"));
    } else {
      log::debug!("config - socks_proxy:  Not defined");
    }
  }

  // Extract backend technology and URLs
  let backend_tech = &site_details_value.backend;
  let shasta_base_url = &site_details_value.shasta_base_url;
  let shasta_barebone_url = shasta_base_url // HACK to not break compatibility with
    // old configuration file. TODO: remove this when needed in the future and all users are
    // using the right configuration file
    .strip_suffix(API_URL_SUFFIX)
    .unwrap_or(shasta_base_url);
  let shasta_api_url = match backend_tech.as_str() {
    "csm" => shasta_barebone_url.to_owned() + API_URL_SUFFIX,
    "ochami" => shasta_barebone_url.to_owned(),
    _ => {
      return Err(
        format!("Invalid backend technology: {}", backend_tech).into(),
      );
    }
  };
  let gitea_base_url = shasta_barebone_url.to_owned() + VCS_URL_SUFFIX;
  let k8s_api_url: Option<&String> = site_details_value
    .k8s
    .as_ref()
    .map(|k8s_details| &k8s_details.api_url);
  let vault_base_url =
    site_details_value
      .k8s
      .as_ref()
      .and_then(|k8s| match &k8s.authentication {
        K8sAuth::Vault { base_url, .. } => Some(base_url),
        K8sAuth::Native { .. } => None,
      });

  let audit_kafka_opt: Option<Kafka> =
    if let Some(auditor) = &configuration.auditor {
      Some(auditor.kafka.clone())
    } else {
      log::warn!("config - Auditor not defined");
      None
    };

  let settings_hsm_group_name_opt = settings.get_string("hsm_group").ok();

  let root_ca_cert_file = &site_details_value.root_ca_cert_file;

  let shasta_root_cert_rslt =
    common::config::get_csm_root_cert_content(root_ca_cert_file);

  let shasta_root_cert = if let Ok(shasta_root_cert) = shasta_root_cert_rslt {
    shasta_root_cert
  } else {
    log::warn!(
      "CA public root file '{}' not found. Proceeding without it.",
      root_ca_cert_file
    );
    vec![]
  };

  let backend = StaticBackendDispatcher::new(
    backend_tech,
    &shasta_api_url,
    &shasta_root_cert,
  )?;

  // Process input params
  let cli = crate::cli::build::build_cli();

  let app_context = AppContext {
    backend: &backend,
    site_name: &site_name,
    shasta_base_url: &shasta_api_url,
    shasta_root_cert: &shasta_root_cert,
    vault_base_url: vault_base_url.map(String::as_str),
    gitea_base_url: &gitea_base_url,
    k8s_api_url: k8s_api_url.map(String::as_str),
    settings_hsm_group_name_opt: settings_hsm_group_name_opt.as_deref(),
    kafka_audit_opt: audit_kafka_opt.as_ref(),
    settings: &settings,
    configuration: &configuration,
  };

  let cli_result = crate::cli::process::process_cli(cli, &app_context).await;

  match cli_result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.into()),
  }
}
