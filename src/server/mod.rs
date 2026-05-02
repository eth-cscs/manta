pub mod handlers;
pub mod routes;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod integration_tests;

use std::net::SocketAddr;
use std::sync::Arc;

use manta_backend_dispatcher::error::Error;
use std::time::Duration;
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
  /// How long a WebSocket console session may be idle before the server
  /// closes it. Protects against leaked Kubernetes pod attachments.
  pub console_inactivity_timeout: std::time::Duration,
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

/// Start the HTTPS server.
pub async fn start_server(
  state: Arc<ServerState>,
  listen_addr: &str,
  port: u16,
  cert_path: &str,
  key_path: &str,
) -> Result<(), Error> {
  let app = routes::build_router(state)
    .layer(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(60)))
    .layer(axum::middleware::from_fn(log_requests));

  let addr: SocketAddr = format!("{}:{}", listen_addr, port)
    .parse()
    .map_err(|e| Error::Message(format!("Invalid listen address: {e}")))?;

  let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
    .await?;

  tracing::info!("Starting HTTPS server on https://{}", addr);

  axum_server::bind_rustls(addr, tls_config)
    .serve(app.into_make_service())
    .await?;

  Ok(())
}
