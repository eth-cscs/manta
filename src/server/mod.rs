//! HTTPS server setup: shared state, request-logging middleware, and the
//! TLS server entry point.

pub mod handlers;
pub mod routes;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod integration_tests;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use manta_backend_dispatcher::error::Error;
use std::time::Duration;
use axum_server::tls_rustls::RustlsConfig;

use crate::common::app_context::InfraContext;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// All per-site connection data the server needs to talk to backend APIs.
///
/// Owned by `ServerState` inside a `HashMap` keyed by site name.
pub struct SiteBackend {
  /// Dispatches API calls to the configured CSM or OpenCHAMI backend.
  pub backend: StaticBackendDispatcher,
  /// Base URL for the CSM/OpenCHAMI API (e.g. `https://api.cluster/apis`).
  pub shasta_base_url: String,
  /// PEM-encoded root CA certificate for the backend; empty vec skips verification.
  pub shasta_root_cert: Vec<u8>,
  /// SOCKS5 proxy URL; `None` means direct connections.
  pub socks5_proxy: Option<String>,
  /// HashiCorp Vault base URL; `None` means features requiring vault return 501.
  pub vault_base_url: Option<String>,
  /// Gitea VCS base URL derived from the site base URL.
  pub gitea_base_url: String,
  /// Kubernetes API URL; `None` means console and log-streaming endpoints return 501.
  pub k8s_api_url: Option<String>,
}

/// Shared state for all HTTP handlers.
///
/// Holds one `SiteBackend` per configured site so that the server can serve
/// multiple clusters.  Each request supplies the target site via the
/// `X-Manta-Site` header; handlers call [`ServerState::infra_context`] to
/// retrieve the per-site data.
pub struct ServerState {
  /// Per-site connection data, keyed by site name.
  pub sites: HashMap<String, SiteBackend>,
  /// How long a WebSocket console session may be idle before the server
  /// closes it.  Protects against leaked Kubernetes pod attachments.
  pub console_inactivity_timeout: Duration,
}

impl ServerState {
  /// Build a borrowed `InfraContext` for the named site.
  ///
  /// Returns `Err(Error::NotFound)` when `site_name` is not in the map.
  /// Called per-request so the service layer can work with its existing
  /// `&InfraContext<'_>` API.
  pub fn infra_context<'a>(
    &'a self,
    site_name: &'a str,
  ) -> Result<InfraContext<'a>, Error> {
    let site = self.sites.get(site_name).ok_or_else(|| {
      Error::NotFound(format!("site '{}' not found", site_name))
    })?;
    Ok(InfraContext {
      backend: &site.backend,
      site_name,
      shasta_base_url: &site.shasta_base_url,
      shasta_root_cert: &site.shasta_root_cert,
      socks5_proxy: site.socks5_proxy.as_deref(),
      vault_base_url: site.vault_base_url.as_deref(),
      gitea_base_url: &site.gitea_base_url,
      k8s_api_url: site.k8s_api_url.as_deref(),
      manta_server_url: None,
    })
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

/// Start the HTTP or HTTPS server.
///
/// When `cert_path` and `key_path` are both `Some`, the server listens with
/// TLS (`https://`).  When either is `None`, it listens as plain HTTP.
pub async fn start_server(
  state: Arc<ServerState>,
  listen_addr: &str,
  port: u16,
  cert_path: Option<&str>,
  key_path: Option<&str>,
) -> Result<(), Error> {
  let app = routes::build_router(state)
    .layer(tower_http::timeout::TimeoutLayer::with_status_code(axum::http::StatusCode::REQUEST_TIMEOUT, Duration::from_secs(60)))
    .layer(axum::middleware::from_fn(log_requests));

  let addr: SocketAddr = format!("{}:{}", listen_addr, port)
    .parse()
    .map_err(|e| Error::BadRequest(format!("Invalid listen address: {e}")))?;

  match (cert_path, key_path) {
    (Some(cert), Some(key)) => {
      let tls_config = RustlsConfig::from_pem_file(cert, key).await?;
      tracing::info!("Starting HTTPS server on https://{}", addr);
      axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;
    }
    (None, None) => {
      tracing::info!("Starting HTTP server on http://{}", addr);
      axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;
    }
    _ => {
      return Err(Error::BadRequest(
        "--cert and --key must be provided together".to_string(),
      ));
    }
  }

  Ok(())
}
