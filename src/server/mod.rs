pub mod handlers;
pub mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Error};
use axum_server::tls_rustls::RustlsConfig;

use crate::common::app_context::InfraContext;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Owned state shared across all HTTP handlers via `Arc`.
///
/// Unlike `InfraContext` (which borrows), this struct owns all
/// data so it can be shared safely across async tasks.
pub struct ServerState {
  pub backend: StaticBackendDispatcher,
  pub site_name: String,
  pub shasta_base_url: String,
  pub shasta_root_cert: Vec<u8>,
  pub vault_base_url: Option<String>,
  pub gitea_base_url: String,
  pub k8s_api_url: Option<String>,
}

impl ServerState {
  /// Build a borrowed `InfraContext` from the owned state.
  ///
  /// This is called per-request so the service layer can work
  /// with its existing `&InfraContext<'_>` API.
  pub fn infra_context(&self) -> InfraContext<'_> {
    InfraContext {
      backend: &self.backend,
      site_name: &self.site_name,
      shasta_base_url: &self.shasta_base_url,
      shasta_root_cert: &self.shasta_root_cert,
      vault_base_url: self.vault_base_url.as_deref(),
      gitea_base_url: &self.gitea_base_url,
      k8s_api_url: self.k8s_api_url.as_deref(),
    }
  }
}

/// Start the HTTPS server.
pub async fn start_server(
  state: Arc<ServerState>,
  listen_addr: &str,
  port: u16,
  cert_path: &str,
  key_path: &str,
) -> Result<(), Error> {
  let app = routes::build_router(state);

  let addr: SocketAddr = format!("{}:{}", listen_addr, port)
    .parse()
    .context("Invalid listen address")?;

  let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
    .await
    .context("Failed to load TLS certificate/key")?;

  log::info!("Starting HTTPS server on https://{}", addr);

  axum_server::bind_rustls(addr, tls_config)
    .serve(app.into_make_service())
    .await
    .context("Server error")?;

  Ok(())
}
