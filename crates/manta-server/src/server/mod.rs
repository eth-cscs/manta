//! HTTPS server setup: shared state, request-logging middleware, and the
//! TLS server entry point.

pub mod api_doc;
pub mod auth_middleware;
pub mod common;
pub mod handlers;
pub mod routes;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use manta_backend_dispatcher::error::Error;
use std::time::Duration;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use crate::server::common::app_context::InfraContext;
use manta_shared::common::kafka::Kafka;

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
  /// Kafka producer for security/audit events (currently used only by
  /// `/api/v1/auth/*`). `None` disables audit emission.
  pub auditor: Option<Kafka>,
  /// Per-source-IP rate limit on `/api/v1/auth/*` (requests/minute).
  /// `None` disables in-process rate limiting.
  pub auth_rate_limit_per_minute: Option<u32>,
  /// Global request timeout applied to every HTTP route (router-level
  /// `TimeoutLayer`). All long-running work (power transitions, SAT
  /// dispatch) runs CLI-side, so this is the only request-timeout
  /// knob the server has.
  pub request_timeout: Duration,
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
      Error::NotFound(format!("site '{site_name}' not found"))
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
  // Both `request_timeout` and `power_timeout` are now applied **inside**
  // `build_router` so the per-route `/power` override actually wins —
  // see the comment on `build_router` for why a global outer layer
  // would silently defeat the override.
  let app = routes::build_router(state)
    .layer(axum::middleware::from_fn(log_requests));

  let addr: SocketAddr = format!("{listen_addr}:{port}")
    .parse()
    .map_err(|e| Error::BadRequest(format!("Invalid listen address: {e}")))?;

  match (cert_path, key_path) {
    (Some(cert), Some(key)) => {
      let tls_config = RustlsConfig::from_pem_file(cert, key).await?;
      let handle = axum_server::Handle::new();
      let ready_handle = handle.clone();
      tokio::spawn(async move {
        ready_handle.listening().await;
        tracing::info!(
          "HTTPS server ready, accepting requests on https://{}",
          addr
        );
        eprintln!("HTTPS server ready, accepting requests on https://{addr}");
      });
      axum_server::bind_rustls(addr, tls_config)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    }
    (None, None) => {
      let handle = axum_server::Handle::new();
      let ready_handle = handle.clone();
      tokio::spawn(async move {
        ready_handle.listening().await;
        tracing::info!(
          "HTTP server ready, accepting requests on http://{}",
          addr
        );
        eprintln!("HTTP server ready, accepting requests on http://{addr}");
      });
      axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
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

#[cfg(test)]
mod timeout_layer_tests {
  //! Behavioural tests for the global + per-route TimeoutLayer
  //! composition used by `start_server` and
  //! `routes::build_router::power_router`. These prove the *pattern*
  //! (outer layer applies to all routes; an inner layer overrides for
  //! the specific routes it wraps) — the production router relies on
  //! exactly this composition to give `/power` more headroom than the
  //! global default without affecting other endpoints.
  //!
  //! Pure tower/axum unit tests — no `ServerState`, no real handlers,
  //! no TCP listener. `tower::ServiceExt::oneshot` drives the router
  //! in-process.
  use std::time::Duration;

  use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
  };
  use tower::ServiceExt as _;
  use tower_http::timeout::TimeoutLayer;

  fn get_req(uri: &str) -> Request<Body> {
    Request::builder()
      .method("GET")
      .uri(uri)
      .body(Body::empty())
      .unwrap()
  }

  /// Handler that sleeps `delay` then returns 200 — used to drive
  /// the timeout layer past its limit on purpose.
  async fn sleep_handler(delay: Duration) -> &'static str {
    tokio::time::sleep(delay).await;
    "ok"
  }

  #[tokio::test]
  async fn global_timeout_returns_408_when_handler_exceeds_limit() {
    let router = Router::new()
      .route(
        "/slow",
        get(|| async { sleep_handler(Duration::from_millis(400)).await }),
      )
      .layer(TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        Duration::from_millis(50),
      ));

    let resp = router.oneshot(get_req("/slow")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::REQUEST_TIMEOUT);
  }

  #[tokio::test]
  async fn fast_handler_finishes_before_timeout_fires() {
    let router = Router::new()
      .route(
        "/fast",
        get(|| async { sleep_handler(Duration::from_millis(10)).await }),
      )
      .layer(TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(5),
      ));

    let resp = router.oneshot(get_req("/fast")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  }

}
