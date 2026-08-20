//! manta-cache-server binary entry point.
//!
//! Standalone HTTP service exposing the `manta-cache` site-resolution
//! index (ROADMAP Stage 3). Loads `cache-server.toml`, runs the
//! initial cross-site refresh, then serves the lookup API — optionally
//! re-refreshing on a fixed interval.
//!
//! # Startup flow (mirrors `manta-server`)
//!
//! 1. Install the `ring` `rustls` `CryptoProvider`.
//! 2. Parse command-line flags via [`parse_cli_args`] (hand-rolled —
//!    like `manta-server`, deliberately no `clap` dependency).
//! 3. Read `cache-server.toml` (`MANTA_CACHE_SERVER_CONFIG` or the
//!    shared manta config directory).
//! 4. Build a Tokio runtime and hand off to [`run_server`]: enforce the
//!    TLS-or-`allow_http` invariant, run the initial refresh, spawn the
//!    optional periodic refresh, and serve until SIGTERM / Ctrl+C.

mod config;
mod credential;
mod refresh;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum_server::tls_rustls::RustlsConfig;

use config::{CacheServerConfiguration, ServerSettings};
use routes::AppState;

/// Usage text printed for `--help`.
const HELP: &str = "\
manta-cache-server — standalone site-resolution lookup service.

Serves the manta-cache index over HTTP: which site owns a given HSM
group or node xname. Refreshes from every configured manta-server at
startup (and optionally on an interval).

Usage: manta-cache-server [OPTIONS]

Options:
      --port <PORT>            Override [server] port from cache-server.toml.
      --cert <CERT>            Override [server] cert from cache-server.toml.
      --key <KEY>              Override [server] key from cache-server.toml.
      --listen-address <ADDR>  Override [server] listen_address.
      --allow-http             Allow listening over plain HTTP when no cert/key
                               is set. Use only when TLS terminates upstream.
  -h, --help                   Print help.
  -V, --version                Print version.
";

/// Parsed command-line flags; each optional field overrides the
/// matching `[server]` key in `cache-server.toml`.
#[derive(Default)]
struct CliArgs {
  /// `--port <PORT>`.
  port: Option<u16>,
  /// `--cert <PATH>`.
  cert: Option<String>,
  /// `--key <PATH>`.
  key: Option<String>,
  /// `--listen-address <ADDR>`.
  listen_address: Option<String>,
  /// `--allow-http` — opt out of the fail-closed TLS requirement.
  allow_http: bool,
}

/// Pulls the value for `--flag` from either `--flag=value` or the next
/// positional arg.
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

/// Hand-rolled parser over `std::env::args`; supports `--flag value`
/// and `--flag=value`. `--help` / `--version` short-circuit.
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
        println!("manta-cache-server {}", env!("CARGO_PKG_VERSION"));
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
      other => return Err(format!("unknown argument: {other}")),
    }
  }
  Ok(out)
}

/// Print the resolved settings on stdout, visible regardless of the
/// log filter — same operator courtesy as manta-server's summary.
/// Token values are never echoed, only their source.
fn print_startup_summary(
  configuration: &CacheServerConfiguration,
  config_path: &std::path::Path,
  listen_addr: &str,
  port: u16,
  has_tls: bool,
) {
  println!("manta-cache-server configuration");
  println!("================================");
  println!("config file: {}", config_path.display());
  println!();
  println!("[server]");
  println!("  listen_address:              {listen_addr}");
  println!("  port:                        {port}");
  println!(
    "  tls:                         {}",
    if has_tls {
      "enabled"
    } else {
      "DISABLED (allow_http)"
    }
  );
  println!(
    "  api_token:                   {}",
    match (
      &configuration.server.api_token,
      &configuration.server.api_token_file,
    ) {
      // `api_token_file` has already been read into `api_token` by
      // `config::load`, so report the path the operator configured
      // rather than the field it landed in.
      (_, Some(path)) => format!("<set from {}>", path.display()),
      (Some(_), None) => "<set>".to_string(),
      (None, None) => "<none — /api/v1 is unauthenticated>".to_string(),
    }
  );
  println!(
    "  refresh_interval_secs:       {}",
    configuration.server.refresh_interval_secs.map_or_else(
      || "<none — startup refresh only>".to_string(),
      |n| n.to_string()
    )
  );
  println!("  log_filter:                  {}", configuration.log);
  println!();
  let now = chrono::Utc::now();
  let mut names: Vec<&String> = configuration.sites.keys().collect();
  names.sort();
  for name in names {
    let site = &configuration.sites[name];
    println!("[site: {name}]");
    println!("  manta_server_url: {}", site.manta_server_url);
    println!("  token_file:       {}", site.token_file.display());

    // Describe the credential from its own claims, so a cache that is
    // about to refresh as somebody's personal token says so before it
    // ever reaches a site. Read failures are left silent here: the
    // initial refresh reports them properly a moment later, and
    // duplicating that as a summary line would only add noise.
    match std::fs::read_to_string(&site.token_file) {
      Ok(raw) => {
        let info = credential::inspect(raw.trim());
        println!("  credential:       {}", info.summary(now));
        for warning in info.warnings(now) {
          println!("  ⚠ WARNING:        {warning}");
        }
      }
      Err(e) => println!("  credential:       <unreadable: {e}>"),
    }
  }
  println!();
}

/// Process entry point: delegate to [`run`], print errors via
/// `Display`.
fn main() {
  if let Err(e) = run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}

/// Synchronous bootstrap: rustls provider, flags, config, logging,
/// then the Tokio runtime driving [`run_server`].
fn run() -> Result<(), Box<dyn std::error::Error>> {
  rustls::crypto::ring::default_provider()
    .install_default()
    .ok();

  let cli =
    parse_cli_args().map_err(|e| format!("{e}\nRun with --help for usage."))?;
  let (configuration, config_path) = config::load()?;

  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_new(&configuration.log)
        .map_err(|e| format!("invalid `log` filter: {e}"))?,
    )
    .init();

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  rt.block_on(run_server(configuration, config_path, cli))
}

/// Async bootstrap: resolve settings, enforce fail-closed TLS, run the
/// initial refresh, spawn the periodic one, and serve.
async fn run_server(
  configuration: CacheServerConfiguration,
  config_path: std::path::PathBuf,
  cli: CliArgs,
) -> Result<(), Box<dyn std::error::Error>> {
  let cert_path = cli.cert.or_else(|| configuration.server.cert.clone());
  let key_path = cli.key.or_else(|| configuration.server.key.clone());
  let has_tls = cert_path.is_some() && key_path.is_some();
  let allow_http = cli.allow_http || configuration.server.allow_http;
  if !has_tls && !allow_http {
    return Err(
      "Refusing to start without TLS: configure `cert` + `key` in \
       [server] (or pass --cert/--key), or set `allow_http = true` / \
       pass --allow-http if TLS terminates upstream. The default is \
       fail-closed so bearer tokens cannot accidentally land on the \
       wire in cleartext."
        .into(),
    );
  }
  if let (Some(_), None) | (None, Some(_)) = (&cert_path, &key_path) {
    return Err("--cert and --key must be provided together".into());
  }
  let port = cli
    .port
    .or(configuration.server.port)
    .unwrap_or_else(|| ServerSettings::default_port(has_tls));
  let listen_addr = cli
    .listen_address
    .or_else(|| configuration.server.listen_address.clone())
    .unwrap_or_else(|| ServerSettings::DEFAULT_LISTEN_ADDRESS.to_string());

  print_startup_summary(
    &configuration,
    &config_path,
    &listen_addr,
    port,
    has_tls,
  );

  // Read everything needed after `configuration` moves into the state.
  let shutdown_grace_period =
    Duration::from_secs(configuration.server.shutdown_grace_period_secs);
  let refresh_interval = configuration.server.refresh_interval_secs;

  let state = Arc::new(AppState {
    cache: tokio::sync::RwLock::new(routes::CacheState::default()),
    config: Arc::new(configuration),
  });

  // Initial refresh before the listener binds (ROADMAP lifecycle).
  // Per-site failures are tolerated — the failed sites are absent from
  // the index and are retried by the periodic loop or a management
  // POST /refresh; only a refresh that cannot start at all (unreadable
  // token_file, client build) aborts startup, since that is an
  // operator error no retry will fix.
  let failures = refresh::refresh_all(&state).await?;
  if !failures.is_empty() {
    for failure in &failures {
      eprintln!("warning: initial refresh: {failure}");
    }
    if refresh_interval.is_none() {
      eprintln!(
        "warning: no refresh_interval_secs configured — the failed \
         sites stay absent until a POST /api/v1/refresh or a restart"
      );
    }
  }

  if let Some(secs) = refresh_interval {
    refresh::spawn_periodic(state.clone(), Duration::from_secs(secs));
  }

  let app =
    routes::build_router(state).layer(axum::middleware::from_fn(log_requests));
  let addr: SocketAddr = format!("{listen_addr}:{port}")
    .parse()
    .map_err(|e| format!("Invalid listen address: {e}"))?;

  match (cert_path, key_path) {
    (Some(cert), Some(key)) => {
      let tls_config = RustlsConfig::from_pem_file(&cert, &key).await?;
      let handle = axum_server::Handle::new();
      announce_when_listening(handle.clone(), addr, "https");
      install_shutdown_handler(handle.clone(), shutdown_grace_period);
      axum_server::bind_rustls(addr, tls_config)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;
    }
    _ => {
      let handle = axum_server::Handle::new();
      announce_when_listening(handle.clone(), addr, "http");
      install_shutdown_handler(handle.clone(), shutdown_grace_period);
      axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;
    }
  }
  Ok(())
}

/// Request-logging middleware: `method uri → status` at INFO, same
/// format as manta-server.
async fn log_requests(
  request: axum::extract::Request,
  next: axum::middleware::Next,
) -> axum::response::Response {
  let method = request.method().clone();
  let uri = request.uri().clone();
  let response = next.run(request).await;
  tracing::info!("{} {} → {}", method, uri, response.status());
  response
}

/// Print the ready line once the listener is actually accepting.
fn announce_when_listening(
  handle: axum_server::Handle<SocketAddr>,
  addr: SocketAddr,
  scheme: &'static str,
) {
  tokio::spawn(async move {
    handle.listening().await;
    tracing::info!("server ready, accepting requests on {scheme}://{addr}");
    eprintln!("server ready, accepting requests on {scheme}://{addr}");
  });
}

/// Graceful shutdown on SIGTERM / Ctrl+C with a bounded drain window —
/// same shape as manta-server's handler.
fn install_shutdown_handler(
  handle: axum_server::Handle<SocketAddr>,
  grace_period: Duration,
) {
  tokio::spawn(async move {
    let mut sigterm = match tokio::signal::unix::signal(
      tokio::signal::unix::SignalKind::terminate(),
    ) {
      Ok(s) => s,
      Err(e) => {
        tracing::warn!(
          "failed to install SIGTERM handler; falling back to Ctrl+C only: {e}"
        );
        let _ = tokio::signal::ctrl_c().await;
        handle.graceful_shutdown(Some(grace_period));
        return;
      }
    };
    let grace_secs = grace_period.as_secs();
    tokio::select! {
      _ = sigterm.recv() => {
        tracing::info!("SIGTERM received; draining for up to {grace_secs}s");
      }
      _ = tokio::signal::ctrl_c() => {
        tracing::info!("Ctrl+C received; draining for up to {grace_secs}s");
      }
    }
    handle.graceful_shutdown(Some(grace_period));
  });
}
