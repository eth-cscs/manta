//! Application entry point: parses CLI args, loads configuration, and
//! launches either the HTTPS server or the CLI command handler.

mod backend_dispatcher;
mod cli;
mod common;
mod manta_backend_dispatcher;
mod server;
mod service;

use ::manta_backend_dispatcher::types::K8sAuth;
use common::{
  app_context::AppContext,
  config::types::{BackendTechnology, MantaConfiguration},
  kafka::Kafka,
};
use manta_backend_dispatcher::StaticBackendDispatcher;

use clap::ArgMatches;

use crate::common::log_ops;

/// URL path suffix for the CSM API endpoint.
const API_URL_SUFFIX: &str = "/apis";

/// URL path suffix for the Gitea VCS endpoint.
const VCS_URL_SUFFIX: &str = "/vcs";

/// Synchronous entry point. Sets environment variables (which
/// must happen before the multi-threaded tokio runtime is active)
/// and then launches the async runtime.
fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
  // Install ring as the rustls CryptoProvider before any TLS code runs.
  // Required when multiple rustls backends are present in the dependency tree
  // (e.g. kube enables aws-lc-rs while we use ring).
  rustls::crypto::ring::default_provider()
    .install_default()
    .ok();

  // Parse CLI arguments early so we can extract --site before
  // starting the multi-threaded runtime.
  let cli_matches = crate::cli::build::build_cli().get_matches();

  // Build a *single-threaded* runtime just to load the config file.
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

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

  // Server mode: skip site resolution entirely — the server serves all
  // configured sites and does not need a single "active" site.
  if cli_matches.subcommand_matches("serve").is_some() {
    return rt.block_on(run_server(settings, configuration, cli_matches));
  }

  // CLI mode: resolve the active site and set the SOCKS5 proxy env var
  // while we are still single-threaded.
  let site_name: String = cli_matches
    .get_one::<String>("site")
    .cloned()
    .unwrap_or_else(|| configuration.site.clone());

  let site_details_value =
    configuration.sites.get(&site_name).ok_or_else(|| {
      let available: Vec<&String> = configuration.sites.keys().collect();
      format!(
        "Site '{}' not found in configuration file. Available sites: {:?}",
        site_name, available
      )
    })?;

  if let Some(socks_proxy) = &site_details_value.socks5_proxy
    && !socks_proxy.is_empty()
  {
    // SAFETY: no other threads are running yet.
    unsafe {
      std::env::set_var("SOCKS5", socks_proxy);
    }
  }

  rt.block_on(run_cli(settings, configuration, site_name, cli_matches))
}

/// Server startup — does not require a valid `site` selection.
/// Loads every site from the configuration and serves all of them.
async fn run_server(
  settings: config::Config,
  configuration: MantaConfiguration,
  cli_matches: ArgMatches,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  log_ops::configure(log_level);

  let serve_matches = cli_matches
    .subcommand_matches("serve")
    .expect("serve subcommand already confirmed");

  let port: u16 = *serve_matches
    .get_one::<u16>("port")
    .expect("port has a default value");
  let cert_path: Option<&str> = serve_matches
    .get_one::<String>("cert")
    .map(String::as_str);
  let key_path: Option<&str> = serve_matches
    .get_one::<String>("key")
    .map(String::as_str);
  let listen_addr: &str = serve_matches
    .get_one::<String>("listen-address")
    .expect("listen-address has a default value");

  let mut sites = std::collections::HashMap::new();
  for (name, site) in &configuration.sites {
    let barebone = site.shasta_base_url
      .strip_suffix(API_URL_SUFFIX)
      .unwrap_or(&site.shasta_base_url);
    let api_url = match &site.backend {
      BackendTechnology::Csm => barebone.to_owned() + API_URL_SUFFIX,
      BackendTechnology::Ochami => barebone.to_owned(),
    };
    let gitea = barebone.to_owned() + VCS_URL_SUFFIX;
    let k8s_url = site.k8s.as_ref().map(|k| k.api_url.clone());
    let vault_url = site.k8s.as_ref().and_then(|k| match &k.authentication {
      K8sAuth::Vault { base_url, .. } => Some(base_url.clone()),
      K8sAuth::Native { .. } => None,
    });
    let root_cert = common::config::get_csm_root_cert_content(&site.root_ca_cert_file)
      .unwrap_or_else(|_| {
        tracing::warn!("CA cert for site '{}' not found, proceeding without it", name);
        vec![]
      });
    let site_backend_dispatcher = StaticBackendDispatcher::new(
      site.backend.as_str(),
      &api_url,
      &root_cert,
      site.socks5_proxy.as_deref(),
    )?;
    sites.insert(name.clone(), server::SiteBackend {
      backend: site_backend_dispatcher,
      shasta_base_url: api_url,
      shasta_root_cert: root_cert,
      socks5_proxy: site.socks5_proxy.clone(),
      vault_base_url: vault_url,
      gitea_base_url: gitea,
      k8s_api_url: k8s_url,
    });
  }

  let server_state = std::sync::Arc::new(server::ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(30 * 60),
  });

  server::start_server(server_state, listen_addr, port, cert_path, key_path)
    .await
    .map_err(|e| e.into())
}

/// CLI startup — requires a valid active site from the configuration.
async fn run_cli(
  settings: config::Config,
  configuration: MantaConfiguration,
  site_name: String,
  cli_matches: ArgMatches,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  log_ops::configure(log_level);

  let site_details_value =
    configuration.sites.get(&site_name).ok_or_else(|| {
      format!("Site '{}' not found in configuration file", site_name)
    })?;

  if let Some(socks_proxy) = &site_details_value.socks5_proxy {
    if !socks_proxy.is_empty() {
      tracing::info!("SOCKS5 enabled: {:?}", std::env::var("SOCKS5"));
    } else {
      tracing::debug!("config - socks_proxy:  Not defined");
    }
  }

  let backend_tech = &site_details_value.backend;
  let shasta_base_url = &site_details_value.shasta_base_url;
  let shasta_barebone_url = shasta_base_url
    .strip_suffix(API_URL_SUFFIX)
    .unwrap_or(shasta_base_url);
  let shasta_api_url = match backend_tech {
    BackendTechnology::Csm => shasta_barebone_url.to_owned() + API_URL_SUFFIX,
    BackendTechnology::Ochami => shasta_barebone_url.to_owned(),
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
      tracing::warn!("config - Auditor not defined");
      None
    };

  let settings_hsm_group_name_opt = settings.get_string("hsm_group").ok();

  let root_ca_cert_file = &site_details_value.root_ca_cert_file;

  let shasta_root_cert =
    match common::config::get_csm_root_cert_content(root_ca_cert_file) {
      Ok(cert) => cert,
      Err(_) => {
        tracing::warn!(
          "CA public root file '{}' not found. Proceeding without it.",
          root_ca_cert_file
        );
        vec![]
      }
    };

  let socks5_proxy = site_details_value.socks5_proxy.as_deref();

  let backend = StaticBackendDispatcher::new(
    backend_tech.as_str(),
    &shasta_api_url,
    &shasta_root_cert,
    socks5_proxy,
  )?;

  let app_context = AppContext {
    infra: crate::common::app_context::InfraContext {
      backend: &backend,
      site_name: &site_name,
      shasta_base_url: &shasta_api_url,
      shasta_root_cert: &shasta_root_cert,
      socks5_proxy,
      vault_base_url: vault_base_url.map(String::as_str),
      gitea_base_url: &gitea_base_url,
      k8s_api_url: k8s_api_url.map(String::as_str),
      manta_server_url: site_details_value.manta_server_url.as_deref(),
    },
    cli: crate::common::app_context::CliConfig {
      settings_hsm_group_name_opt: settings_hsm_group_name_opt.as_deref(),
      kafka_audit_opt: audit_kafka_opt.as_ref(),
      settings: &settings,
      configuration: &configuration,
    },
  };

  let cli_result =
    crate::cli::process::process_cli(&cli_matches, &app_context).await;

  match cli_result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.into()),
  }
}
