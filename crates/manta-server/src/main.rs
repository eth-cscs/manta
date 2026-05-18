//! Manta HTTP server entry point. Loads the configuration, builds a
//! `ServerState` containing one `SiteBackend` per configured site, and
//! starts the TLS server.

mod backend_dispatcher;
mod common;
mod manta_backend_dispatcher;
mod server;
mod service;

use ::manta_backend_dispatcher::types::K8sAuth;
use clap::{Arg, Command};
use manta_shared::common::{
  config as manta_config,
  config::types::{BackendTechnology, ServerConfiguration},
  log_ops,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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

fn run() -> core::result::Result<(), Box<dyn std::error::Error>> {
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
        .help("Override [server] port from server.toml."),
    )
    .arg(
      Arg::new("cert")
        .long("cert")
        .help("Override [server] cert from server.toml."),
    )
    .arg(
      Arg::new("key")
        .long("key")
        .help("Override [server] key from server.toml."),
    )
    .arg(
      Arg::new("listen-address")
        .long("listen-address")
        .help("Override [server] listen_address from server.toml."),
    )
    .get_matches();

  let settings = manta_config::get_server_configuration()
    .map_err(|e| format!("Could not read server configuration: {}", e))?;
  let configuration: ServerConfiguration = settings
    .try_deserialize()
    .map_err(|e| format!("Server configuration file is not valid: {}", e))?;

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  rt.block_on(run_server(configuration, cli))
}

async fn run_server(
  configuration: ServerConfiguration,
  cli: clap::ArgMatches,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  log_ops::configure(configuration.log.clone());

  // Resolution precedence for each setting: CLI flag > config file > fallback.
  let port: u16 = cli
    .get_one::<u16>("port")
    .copied()
    .unwrap_or(configuration.server.port);
  let listen_addr: String = cli
    .get_one::<String>("listen-address")
    .cloned()
    .unwrap_or_else(|| configuration.server.listen_address.clone());
  let cert_path: Option<String> = cli
    .get_one::<String>("cert")
    .cloned()
    .or_else(|| configuration.server.cert.clone());
  let key_path: Option<String> = cli
    .get_one::<String>("key")
    .cloned()
    .or_else(|| configuration.server.key.clone());
  let console_inactivity_timeout = std::time::Duration::from_secs(
    configuration.server.console_inactivity_timeout_secs,
  );

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

  let auditor = configuration.auditor.as_ref().map(|a| a.kafka.clone());
  let server_state = std::sync::Arc::new(server::ServerState {
    sites,
    console_inactivity_timeout,
    auditor,
    auth_rate_limit_per_minute: configuration.server.auth_rate_limit_per_minute,
  });

  server::start_server(
    server_state,
    &listen_addr,
    port,
    cert_path.as_deref(),
    key_path.as_deref(),
  )
  .await
  .map_err(|e| e.into())
}
