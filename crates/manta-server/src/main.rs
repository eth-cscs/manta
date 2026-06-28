//! Manta HTTP server binary entry point. Loads the configuration,
//! builds a `ServerState` containing one `SiteBackend` per configured
//! site, and starts the TLS server.
//!
//! The crate is set up as both a library (`src/lib.rs`, used by the
//! integration tests in `tests/`) and a binary (this file). All
//! module logic lives in the library; this file is just bootstrap.
//!
//! # Startup flow
//!
//! 1. Install the `ring` `rustls` `CryptoProvider`.
//! 2. Parse command-line flags via [`parse_cli_args`] (hand-rolled —
//!    `manta-server` deliberately has no `clap` dependency).
//! 3. Short-circuit on `--emit-openapi`: dump the OpenAPI spec and exit
//!    without reading any config file. Used by CI to regenerate
//!    `crates/manta-cli/openapi.json`.
//! 4. Read `server.toml` (path resolved by
//!    `manta_shared::common::config`).
//! 5. Build a Tokio multi-thread runtime and hand off to [`run_server`].
//!
//! [`run_server`] then resolves CLI-vs-config-vs-fallback settings,
//! validates that TLS is configured (or `--allow-http` was given),
//! constructs one `SiteBackend` per `[sites.*]` entry, and finally
//! calls [`server::start_server`].

use ::manta_backend_dispatcher::types::K8sAuth;
use manta_shared::common::{config as manta_config, log_ops};

use manta_server::config::{BackendTechnology, ServerConfiguration};
use manta_server::dispatcher::StaticBackendDispatcher;
use manta_server::server;

/// URL path suffix appended to a site's `shasta_base_url` to reach the
/// CSM API root (e.g. `https://api.cluster.local/apis`). Stripped from
/// the configured value before per-backend reassembly, so operators can
/// equivalently set the base URL with or without the suffix in
/// `server.toml`.
const API_URL_SUFFIX: &str = "/apis";

/// URL path suffix appended to a site's bare base URL to reach the
/// Gitea VCS root (e.g. `https://api.cluster.local/vcs`).
const VCS_URL_SUFFIX: &str = "/vcs";

/// Usage text printed for `--help`.
const HELP: &str = "\
Manta HTTPS server — proxies CLI requests to CSM/Ochami backends.

Usage: manta-server [OPTIONS]

Options:
      --port <PORT>                    Override [server] port from server.toml.
      --cert <CERT>                    Override [server] cert from server.toml.
      --key <KEY>                      Override [server] key from server.toml.
      --listen-address <ADDR>          Override [server] listen_address from server.toml.
      --allow-http                     Allow listening over plain HTTP when no cert/key is
                                       set. Use only when TLS terminates upstream (reverse
                                       proxy, sidecar).
      --emit-openapi                   Dump the OpenAPI spec to stdout as JSON and exit.
                                       Used to regenerate crates/manta-cli/openapi.json
                                       after handler or schema changes — no config file is
                                       read.
  -h, --help                           Print help.
  -V, --version                        Print version.
";

/// Parsed command-line flags. Filled by [`parse_cli_args`]; consumed by
/// [`run`] / [`run_server`]. Each optional field overrides the
/// matching `[server]` key in `server.toml` when set.
#[derive(Default)]
struct CliArgs {
  /// `--port <PORT>` — TCP port to bind. Falls back to
  /// `[server].port`, then `ServerSettings::default_port(has_tls)`.
  port: Option<u16>,
  /// `--cert <PATH>` — PEM cert path for TLS. Falls back to
  /// `[server].cert`.
  cert: Option<String>,
  /// `--key <PATH>` — PEM private-key path for TLS. Falls back to
  /// `[server].key`.
  key: Option<String>,
  /// `--listen-address <ADDR>` — bind address. Falls back to
  /// `[server].listen_address`, then
  /// `ServerSettings::DEFAULT_LISTEN_ADDRESS`.
  listen_address: Option<String>,
  /// `--allow-http` — opt out of the fail-closed TLS requirement.
  /// OR-ed with `[server].allow_http`.
  allow_http: bool,
  /// `--emit-openapi` — dump the OpenAPI spec to stdout and exit.
  /// Skips config-file resolution entirely.
  emit_openapi: bool,
}

/// Pulls the value for `--flag` from either `--flag=value` (already
/// split into `inline`) or the next positional arg. Centralised so
/// every value-taking flag emits the same "requires a value" message.
fn take_value(
  inline: Option<String>,
  rest: &mut impl Iterator<Item = String>,
  flag: &str,
) -> Result<String, String> {
  match inline {
    Some(v) => Ok(v),
    None => rest
      .next()
      .ok_or_else(|| format!("{flag} requires a value")),
  }
}

/// Hand-rolled parser over `std::env::args`. Supports `--flag value` and
/// `--flag=value`. `--help` / `--version` short-circuit with `exit(0)`.
/// Unknown args and value-less value-flags return `Err` so `run` can
/// surface them with a uniform "run with --help" hint.
fn parse_cli_args() -> Result<CliArgs, String> {
  let mut args = std::env::args().skip(1);
  let mut out = CliArgs::default();

  while let Some(raw) = args.next() {
    let (name, inline) = match raw.split_once('=') {
      Some((n, v)) => (n.to_string(), Some(v.to_string())),
      None => (raw, None),
    };
    match name.as_str() {
      "--help" | "-h" => {
        print!("{HELP}");
        std::process::exit(0);
      }
      "--version" | "-V" => {
        println!("manta-server {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
      }
      "--port" => {
        let v = take_value(inline, &mut args, "--port")?;
        out.port = Some(
          v.parse::<u16>()
            .map_err(|e| format!("--port: invalid u16 '{v}': {e}"))?,
        );
      }
      "--cert" => out.cert = Some(take_value(inline, &mut args, "--cert")?),
      "--key" => out.key = Some(take_value(inline, &mut args, "--key")?),
      "--listen-address" => {
        out.listen_address =
          Some(take_value(inline, &mut args, "--listen-address")?);
      }
      "--allow-http" => out.allow_http = true,
      "--emit-openapi" => out.emit_openapi = true,
      other => return Err(format!("unknown argument: {other}")),
    }
  }
  Ok(out)
}

/// Print the resolved server settings, audit configuration, and the
/// config-file path to stdout. Visible regardless of the `[log]`
/// filter, so operators can confirm what the server is running on
/// without first turning logging up. Per-site blocks are emitted
/// separately by [`print_site_summary`] from the `[sites.*]` loop in
/// [`run_server`].
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

/// Print one `[site: <name>]` block on stdout, matching the format
/// produced by [`print_startup_summary`]. Sensitive paths (cert/key)
/// are not echoed here; only the public-facing URLs and the CA cert
/// path are shown.
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

/// Synchronous bootstrap: install the rustls crypto provider, parse
/// flags, handle the `--emit-openapi` short-circuit, load
/// `server.toml`, and spin up the Tokio runtime that drives
/// [`run_server`]. Errors propagate to [`main`] for display.
fn run() -> core::result::Result<(), Box<dyn std::error::Error>> {
  // Install ring as the rustls CryptoProvider before any TLS code runs.
  rustls::crypto::ring::default_provider()
    .install_default()
    .ok();

  let cli =
    parse_cli_args().map_err(|e| format!("{e}\nRun with --help for usage."))?;

  if cli.emit_openapi {
    use utoipa::OpenApi;
    let spec = manta_server::server::api_doc::ApiDoc::openapi()
      .to_pretty_json()
      .map_err(|e| format!("Failed to serialise OpenAPI spec: {e}"))?;
    println!("{spec}");
    return Ok(());
  }

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

/// Async bootstrap: resolve CLI-vs-config-vs-fallback settings,
/// enforce the TLS-or-`allow_http` invariant, build one `SiteBackend`
/// per `[sites.*]` entry (each with its own dispatcher and CA-pinned
/// HTTP client), assemble the shared `ServerState`, then call
/// [`server::start_server`]. Bails out at startup if any per-site CA
/// cert is missing or unreadable — see the inline comment for why we
/// don't silently fall back to the system trust store.
async fn run_server(
  configuration: ServerConfiguration,
  cli: CliArgs,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  log_ops::configure(&configuration.log, true);

  // Resolution precedence for each setting: CLI flag > config file > fallback.
  // cert/key are resolved first so the port fallback can branch on whether
  // TLS is configured.
  let cert_path: Option<String> =
    cli.cert.or_else(|| configuration.server.cert.clone());
  let key_path: Option<String> =
    cli.key.or_else(|| configuration.server.key.clone());
  let has_tls = cert_path.is_some() && key_path.is_some();
  let allow_http = cli.allow_http || configuration.server.allow_http;
  if !has_tls && !allow_http {
    return Err(
      "Refusing to start without TLS: configure `cert` + `key` in \
       [server] (or pass --cert/--key), or set `allow_http = true` / \
       pass --allow-http if TLS terminates upstream. The default \
       is fail-closed so bearer tokens cannot accidentally land on \
       the wire in cleartext."
        .into(),
    );
  }
  let port: u16 = cli.port.or(configuration.server.port).unwrap_or_else(|| {
    manta_server::config::ServerSettings::default_port(has_tls)
  });
  let listen_addr: String = cli
    .listen_address
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
  let shutdown_grace_period = std::time::Duration::from_secs(
    configuration.server.shutdown_grace_period_secs,
  );
  let server_state = std::sync::Arc::new(server::ServerState {
    sites,
    console_inactivity_timeout,
    auditor,
    auth_rate_limit_per_minute: configuration.server.auth_rate_limit_per_minute,
    request_timeout,
    shutdown_grace_period,
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
