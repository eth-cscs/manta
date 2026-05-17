//! Manta HTTP server entry point. Loads the configuration, builds a
//! `ServerState` containing one `SiteBackend` per configured site, and
//! starts the TLS server.

mod common;
mod server;
mod service;

use clap::{Arg, Command};
use manta_backend_dispatcher::types::K8sAuth;
use manta_shared::common::{
  config as manta_config,
  config::types::{BackendTechnology, MantaConfiguration},
  log_ops,
};
use manta_shared::manta_backend_dispatcher::StaticBackendDispatcher;

/// URL path suffix for the CSM API endpoint.
const API_URL_SUFFIX: &str = "/apis";

/// URL path suffix for the Gitea VCS endpoint.
const VCS_URL_SUFFIX: &str = "/vcs";

/// Default TLS port (kept as &str so clap's `default_value` accepts it
/// without an extra allocation).
const DEFAULT_PORT: &str = "8443";

/// Default listen address.
const DEFAULT_LISTEN: &str = "0.0.0.0";

fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
  // Install ring as the rustls CryptoProvider before any TLS code runs.
  rustls::crypto::ring::default_provider()
    .install_default()
    .ok();

  let cli = Command::new("manta-server")
    .about("Manta HTTPS server — proxies CLI requests to CSM/Ochami backends.")
    .arg(
      Arg::new("port")
        .long("port")
        .value_parser(clap::value_parser!(u16))
        .default_value(DEFAULT_PORT),
    )
    .arg(
      Arg::new("cert")
        .long("cert")
        .help("Path to the TLS certificate (PEM)."),
    )
    .arg(
      Arg::new("key")
        .long("key")
        .help("Path to the TLS private key (PEM)."),
    )
    .arg(
      Arg::new("listen-address")
        .long("listen-address")
        .default_value(DEFAULT_LISTEN),
    )
    .get_matches();

  let preliminary_rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;
  let settings = preliminary_rt
    .block_on(async { manta_config::get_configuration().await })
    .map_err(|e| format!("Could not read configuration file: {}", e))?;
  let configuration: MantaConfiguration = settings
    .clone()
    .try_deserialize()
    .map_err(|e| format!("Configuration file is not valid: {}", e))?;
  drop(preliminary_rt);

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  rt.block_on(run_server(settings, configuration, cli))
}

async fn run_server(
  settings: ::config::Config,
  configuration: MantaConfiguration,
  cli: clap::ArgMatches,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  log_ops::configure(log_level);

  let port: u16 = *cli.get_one::<u16>("port").expect("port has a default value");
  let cert_path: Option<&str> = cli.get_one::<String>("cert").map(String::as_str);
  let key_path: Option<&str> = cli.get_one::<String>("key").map(String::as_str);
  let listen_addr: &str = cli
    .get_one::<String>("listen-address")
    .expect("listen-address has a default value");

  let mut sites = std::collections::HashMap::new();
  for (name, site) in &configuration.sites {
    let barebone = site
      .shasta_base_url
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
    let root_cert =
      manta_config::get_csm_root_cert_content(&site.root_ca_cert_file)
        .unwrap_or_else(|_| {
          tracing::warn!(
            "CA cert for site '{}' not found, proceeding without it",
            name
          );
          vec![]
        });
    let site_backend_dispatcher = StaticBackendDispatcher::new(
      site.backend.as_str(),
      &api_url,
      &root_cert,
      site.socks5_proxy.as_deref(),
    )?;
    sites.insert(
      name.clone(),
      server::SiteBackend {
        backend: site_backend_dispatcher,
        shasta_base_url: api_url,
        shasta_root_cert: root_cert,
        socks5_proxy: site.socks5_proxy.clone(),
        vault_base_url: vault_url,
        gitea_base_url: gitea,
        k8s_api_url: k8s_url,
      },
    );
  }

  let server_state = std::sync::Arc::new(server::ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(30 * 60),
  });

  server::start_server(server_state, listen_addr, port, cert_path, key_path)
    .await
    .map_err(|e| e.into())
}
