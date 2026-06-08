//! Manta HTTP server binary entry point. Loads the configuration,
//! builds a `ServerState` containing one `SiteBackend` per configured
//! site, and starts the TLS server.
//!
//! The crate is set up as both a library (`src/lib.rs`, used by the
//! integration tests in `tests/`) and a binary (this file). All
//! module logic lives in the library; this file is just bootstrap.

use ::manta_backend_dispatcher::types::K8sAuth;
use clap::{Arg, Command};
use manta_shared::common::{config as manta_config, log_ops};

use manta_server::config::{BackendTechnology, ServerConfiguration};
use manta_server::dispatcher::StaticBackendDispatcher;
use manta_server::server;

/// URL path suffix for the CSM API endpoint.
const API_URL_SUFFIX: &str = "/apis";

/// URL path suffix for the Gitea VCS endpoint.
const VCS_URL_SUFFIX: &str = "/vcs";

/// Print the resolved server settings, audit configuration, and the
/// config-file path to stdout. Visible regardless of the `[log]`
/// filter, so operators can confirm what the server is running on
/// without first turning logging up.
fn print_startup_summary(
  configuration: &ServerConfiguration,
  listen_addr: &str,
  port: u16,
  cert_path: &Option<String>,
  key_path: &Option<String>,
) {
  let (config_path, source) = match manta_config::get_server_config_file_path()
  {
    Ok(p) => (
      p.display().to_string(),
      if std::env::var("MANTA_SERVER_CONFIG").is_ok() {
        "MANTA_SERVER_CONFIG env var"
      } else {
        "default lookup (~/.config/manta/server.toml)"
      },
    ),
    Err(_) => ("<unknown>".to_string(), "unresolved"),
  };
  println!("manta-server configuration");
  println!("==========================");
  println!("config file: {config_path}");
  println!("source:      {source}");
  println!();
  println!("[server]");
  println!("  listen_address:                   {listen_addr}");
  println!("  port:                             {port}");
  println!(
    "  cert:                             {}",
    cert_path.as_deref().unwrap_or("<none>")
  );
  println!(
    "  key:                              {}",
    key_path.as_deref().map_or("<none>", |_| "<set>")
  );
  println!(
    "  console_inactivity_timeout_secs:  {}",
    configuration.server.console_inactivity_timeout_secs
  );
  println!(
    "  auth_rate_limit_per_minute:       {}",
    configuration
      .server
      .auth_rate_limit_per_minute
      .map_or_else(|| "<disabled>".to_string(), |n| n.to_string())
  );
  println!("  log_filter:                       {}", configuration.log);
  println!();
  println!("[auditor]");
  match configuration.auditor.as_ref() {
    Some(a) => {
      println!("  Kafka audit forwarder enabled");
      println!("  brokers: {:?}", a.kafka.brokers);
      println!("  topic:   {}", a.kafka.topic);
    }
    None => println!("  disabled (no audit messages will be emitted)"),
  }
  println!();
}

/// Print one site block, matching the format produced by
/// [`print_startup_summary`].
#[allow(clippy::too_many_arguments)]
fn print_site_summary(
  name: &str,
  backend: &str,
  shasta_base_url: &str,
  gitea_base_url: &str,
  k8s_api_url: Option<&str>,
  vault_base_url: Option<&str>,
  has_socks5_proxy: bool,
  root_ca_cert_file: &str,
) {
  println!("[site: {name}]");
  println!("  backend:           {backend}");
  println!("  shasta_base_url:   {shasta_base_url}");
  println!("  gitea_base_url:    {gitea_base_url}");
  println!("  k8s_api_url:       {}", k8s_api_url.unwrap_or("<none>"));
  println!(
    "  vault_base_url:    {}",
    vault_base_url.unwrap_or("<none>")
  );
  println!(
    "  socks5_proxy:      {}",
    if has_socks5_proxy { "<set>" } else { "<none>" }
  );
  println!("  root_ca_cert_file: {root_ca_cert_file}");
  println!();
}

/// Process entry point. Delegates to `run` and prints any error with
/// `Display` (not `Debug`) so multi-line messages aren't escaped.
fn main() {
  if let Err(e) = run() {
    eprintln!("{e}");
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
    .map_err(|e| format!("Could not read server configuration: {e}"))?;
  let configuration: ServerConfiguration = settings
    .try_deserialize()
    .map_err(|e| format!("Server configuration file is not valid: {e}"))?;

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  rt.block_on(run_server(configuration, cli))
}

async fn run_server(
  configuration: ServerConfiguration,
  cli: clap::ArgMatches,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  log_ops::configure(&configuration.log, true);

  // Resolution precedence for each setting: CLI flag > config file > fallback.
  // cert/key are resolved first so the port fallback can branch on whether
  // TLS is configured.
  let cert_path: Option<String> = cli
    .get_one::<String>("cert")
    .cloned()
    .or_else(|| configuration.server.cert.clone());
  let key_path: Option<String> = cli
    .get_one::<String>("key")
    .cloned()
    .or_else(|| configuration.server.key.clone());
  let has_tls = cert_path.is_some() && key_path.is_some();
  let port: u16 = cli
    .get_one::<u16>("port")
    .copied()
    .or(configuration.server.port)
    .unwrap_or_else(|| {
      manta_server::config::ServerSettings::default_port(has_tls)
    });
  let listen_addr: String = cli
    .get_one::<String>("listen-address")
    .cloned()
    .or_else(|| configuration.server.listen_address.clone())
    .unwrap_or_else(|| {
      manta_server::config::ServerSettings::DEFAULT_LISTEN_ADDRESS.to_string()
    });
  let console_inactivity_timeout = std::time::Duration::from_secs(
    configuration.server.console_inactivity_timeout_secs,
  );
  let request_timeout =
    std::time::Duration::from_secs(configuration.server.request_timeout_secs);

  // Resolve `migrate_backup_root` once at startup so per-request
  // path validation is just a `starts_with` against an already-
  // canonical PathBuf. Treating a missing directory as a hard error
  // catches operator typos before the first migrate call.
  let migrate_backup_root = match configuration
    .server
    .migrate_backup_root
    .as_deref()
  {
    Some(raw) => Some(
      std::path::PathBuf::from(raw).canonicalize().map_err(|e| {
        format!(
          "[server] migrate_backup_root '{raw}' could not be canonicalised: {e}. \
           Either point it at an existing directory or remove the entry to keep \
           the /migrate/* endpoints disabled."
        )
      })?,
    ),
    None => None,
  };

  print_startup_summary(
    &configuration,
    &listen_addr,
    port,
    &cert_path,
    &key_path,
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
    // CA cert is required: a missing or unreadable file means every
    // backend HTTPS call would fall through to the empty-vec branch
    // which `reqwest::Certificate::from_pem` accepts as "no extra
    // trust anchors". That silently widens the trust store to the
    // system default, which on most operator workstations does not
    // include the CSM/OpenCHAMI internal CA — so calls work but
    // without the expected pinning. Fail at startup instead.
    let root_cert =
      manta_config::get_csm_root_cert_content(&site.root_ca_cert_file)
        .map_err(|e| {
          format!(
            "CA cert for site '{name}' at '{}' could not be read: {e}. \
             Fix the path under [sites.{name}].root_ca_cert_file in \
             server.toml or remove the site entry.",
            site.root_ca_cert_file
          )
        })?;
    print_site_summary(
      name,
      site.backend.as_str(),
      &api_url,
      &gitea,
      k8s_url.as_deref(),
      vault_url.as_deref(),
      site.socks5_proxy.is_some(),
      &site.root_ca_cert_file,
    );
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
    request_timeout,
    migrate_backup_root,
  });

  server::start_server(
    server_state,
    &listen_addr,
    port,
    cert_path.as_deref(),
    key_path.as_deref(),
  )
  .await
  .map_err(std::convert::Into::into)
}
