//! Application entry point: parses CLI args, loads configuration, and
//! launches the CLI command handler. The CLI never talks to CSM /
//! OCHAMI directly — every operation is forwarded to the manta HTTPS
//! server named by `cli.toml`'s `manta_server_url`.

mod build;
mod commands;
mod common;
mod handlers;
mod http_client;
mod output;
mod process;

use crate::common::app_context::AppContext;
use crate::common::config::CliConfiguration;

use clap::ArgMatches;

use manta_shared::common::log_ops;

/// Process entry point. Delegates to `run` and prints any error with
/// `Display` (not `Debug`) so multi-line messages aren't escaped.
fn main() {
  if let Err(e) = run() {
    eprintln!("{e}");
    std::process::exit(1);
  }
}

/// Synchronous entry point. Loads `cli.toml`, resolves the active site,
/// sets the SOCKS5 env var (must happen before the multi-threaded tokio
/// runtime is active), and then launches the async runtime.
fn run() -> core::result::Result<(), Box<dyn std::error::Error>> {
  let cli_matches = crate::build::build_cli().get_matches();

  let settings = manta_shared::common::config::get_cli_configuration()
    .map_err(|e| format!("Could not read CLI configuration file: {e}"))?;
  let configuration: CliConfiguration = settings
    .clone()
    .try_deserialize()
    .map_err(|e| format!("CLI configuration file is not valid: {e}"))?;

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

  // Resolve the active site name (just a header value — the server
  // validates it). Set the SOCKS5 proxy env var while we are still
  // single-threaded; the proxy is used to reach manta-server, not the
  // backends — per-site backend proxying is the server's concern.
  let site_name: String = cli_matches
    .get_one::<String>("site")
    .cloned()
    .unwrap_or_else(|| configuration.site.clone());

  if let Some(socks_proxy) = &configuration.socks5_proxy
    && !socks_proxy.is_empty()
  {
    // SAFETY: no other threads are running yet.
    unsafe {
      std::env::set_var("SOCKS5", socks_proxy);
    }
  }

  rt.block_on(run_cli(settings, configuration, site_name, cli_matches))
}

/// CLI startup — takes the resolved site name and forwards it on every
/// request via the `X-Manta-Site` header.
async fn run_cli(
  settings: config::Config,
  configuration: CliConfiguration,
  site_name: String,
  cli_matches: ArgMatches,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  log_ops::configure(log_level, false);

  if let Some(socks_proxy) = &configuration.socks5_proxy {
    if !socks_proxy.is_empty() {
      tracing::info!("SOCKS5 enabled: {:?}", std::env::var("SOCKS5"));
    } else {
      tracing::debug!("config - socks5_proxy: not defined");
    }
  }

  let settings_hsm_group_name_opt = settings.get_string("hsm_group").ok();
  let manta_server_url = configuration.manta_server_url.as_str();

  let app_context = AppContext {
    site_name: &site_name,
    manta_server_url,
    settings_hsm_group_name_opt: settings_hsm_group_name_opt.as_deref(),
    request_timeout_secs: configuration.request_timeout_secs,
    settings: &settings,
  };

  let cli_result =
    crate::process::process_cli(&cli_matches, &app_context).await;

  match cli_result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.into()),
  }
}
