//! Application entry point: parses CLI args, loads configuration, and
//! launches either the HTTPS server or the CLI command handler.

mod cli;
mod common;
// `shared`, `backend_dispatcher`, `manta_backend_dispatcher`, and the
// `common::*` partition all live in the `manta-shared` workspace crate.
// `server` and `service` were extracted into the `manta-server` workspace
// crate (its own binary); the CLI no longer compiles them.

use ::manta_backend_dispatcher::types::K8sAuth;
use common::{
  app_context::AppContext,
  config::types::{BackendTechnology, CliConfiguration},
  kafka::Kafka,
};
use manta_shared::manta_backend_dispatcher::StaticBackendDispatcher;

use clap::ArgMatches;

use crate::common::log_ops;

/// URL path suffix for the CSM API endpoint.
const API_URL_SUFFIX: &str = "/apis";

/// URL path suffix for the Gitea VCS endpoint.
const VCS_URL_SUFFIX: &str = "/vcs";

/// Process entry point. Delegates to `run` and prints any error with
/// `Display` (not `Debug`) so multi-line messages aren't escaped.
fn main() {
  if let Err(e) = run() {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}

/// Synchronous entry point. Loads `cli.toml`, resolves the active site,
/// sets the SOCKS5 env var (must happen before the multi-threaded tokio
/// runtime is active), and then launches the async runtime.
fn run() -> core::result::Result<(), Box<dyn std::error::Error>> {
  let cli_matches = crate::cli::build::build_cli().get_matches();

  let settings = common::config::get_cli_configuration()
    .map_err(|e| format!("Could not read CLI configuration file: {}", e))?;
  let configuration: CliConfiguration = settings
    .clone()
    .try_deserialize()
    .map_err(|e| format!("CLI configuration file is not valid: {}", e))?;

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

  // Resolve the active site and set the SOCKS5 proxy env var
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

/// CLI startup — requires a valid active site from the configuration.
async fn run_cli(
  settings: config::Config,
  configuration: CliConfiguration,
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

  let manta_server_url = configuration.manta_server_url.as_deref();
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
    },
    cli: crate::common::app_context::CliConfig {
      settings_hsm_group_name_opt: settings_hsm_group_name_opt.as_deref(),
      kafka_audit_opt: audit_kafka_opt.as_ref(),
      settings: &settings,
      manta_server_url,
    },
  };

  let cli_result =
    crate::cli::process::process_cli(&cli_matches, &app_context).await;

  match cli_result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.into()),
  }
}
