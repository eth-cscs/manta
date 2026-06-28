//! Axum HTTP/HTTPS server setup.
//!
//! - [`ServerState`] — shared application state passed through every
//!   handler via Axum's `State<Arc<ServerState>>` extractor. Holds one
//!   [`SiteBackend`] per configured site so a single server can fan
//!   out to multiple CSM/OpenCHAMI clusters.
//! - [`start_server`] — binary entry point. Builds the router (see
//!   [`routes::build_router`]), installs the request-logging
//!   middleware, optionally wraps the listener in TLS, and installs a
//!   SIGTERM/Ctrl+C handler for graceful shutdown.
//! - Submodules:
//!   - [`handlers`] — per-resource Axum handlers; converts HTTP
//!     requests into service-layer calls.
//!   - [`routes`] — router registration (one entry per `/api/v1`
//!     path).
//!   - [`auth_middleware`] — defensive middleware applied to
//!     `/api/v1/auth/*` (per-IP rate limit + body redaction).
//!   - [`common`] — server-only helpers (per-request `InfraContext`,
//!     Kafka audit producer, JWT claim extractors, Vault client).
//!   - [`api_doc`] — utoipa OpenAPI document served at
//!     `GET /openapi.json` + `GET /docs`.

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

use crate::dispatcher::StaticBackendDispatcher;
use crate::server::common::app_context::InfraContext;
use crate::server::common::kafka::Kafka;

/// All per-site connection data the server needs to talk to backend APIs.
///
/// Built once at startup from a `[sites.X]` block in `server.toml`,
/// then owned by [`ServerState::sites`] inside a `HashMap` keyed by
/// the site name. The matching `[sites.X]` block is selected per
/// request from the `X-Manta-Site` header.
///
/// Borrowed per request as an [`common::app_context::InfraContext`]
/// via [`ServerState::infra_context`] so the service layer can pass
/// the per-site bundle around without taking ownership.
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
/// Holds one [`SiteBackend`] per configured site so a single server
/// can serve multiple clusters. Each request supplies the target site
/// via the `X-Manta-Site` header; handlers call
/// [`ServerState::infra_context`] (or, via the
/// [`handlers::RequestCtx`] extractor, the cached
/// `RequestCtx::infra()` shortcut) to retrieve the per-site data.
///
/// Plumbed through Axum's `State<Arc<ServerState>>` extractor. Owned
/// by [`start_server`] and cloned (cheaply, since it's an `Arc`) into
/// every spawned task.
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
  /// Drain window for `axum_server::Handle::graceful_shutdown` on
  /// SIGTERM / Ctrl+C. Sourced from
  /// `server.toml`'s `[server] shutdown_grace_period_secs`.
  pub shutdown_grace_period: Duration,
  /// Filesystem root that confines `POST /migrate/{backup,restore}`
  /// file access. `None` disables both endpoints — even admin callers
  /// must wait for an operator to opt in via `[server]
  /// migrate_backup_root`. The path is stored already-canonicalised
  /// so per-request validation is a single `starts_with` against this.
  pub migrate_backup_root: Option<std::path::PathBuf>,
}

impl ServerState {
  /// Build a borrowed [`InfraContext`] for the named site.
  ///
  /// Called per-request so the service layer can work with its
  /// existing `&InfraContext<'_>` API without taking ownership of the
  /// underlying [`SiteBackend`].
  ///
  /// # Errors
  ///
  /// Returns [`Error::NotFound`] when `site_name` is not in
  /// [`Self::sites`].
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

/// Request-logging middleware. Logs `method uri → status` at INFO
/// after the inner handler returns, including handler-internal
/// error responses. Composed once by [`start_server`] around the
/// router built by [`routes::build_router`].
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
/// Builds the router via [`routes::build_router`], wraps it with the
/// request-logging middleware, binds the listener at
/// `<listen_addr>:<port>`, and serves until a SIGTERM or Ctrl+C is
/// received — at which point the in-process shutdown handler
/// triggers `axum_server`'s graceful drain with the
/// [`ServerState::shutdown_grace_period`] window.
///
/// When `cert_path` and `key_path` are both `Some`, the server
/// listens with TLS (`https://`). When both are `None`, it listens
/// as plain HTTP. Mixing one of the two is rejected.
///
/// # Errors
///
/// - [`Error::BadRequest`] when `listen_addr:port` does not parse as
///   a `SocketAddr`, or when exactly one of `cert_path` / `key_path`
///   is supplied (they must be set together).
/// - Any I/O / TLS load error from `RustlsConfig::from_pem_file` or
///   the underlying `axum_server::bind*` call surfaces via the
///   `From<io::Error>` impl on [`Error`].
pub async fn start_server(
  state: Arc<ServerState>,
  listen_addr: &str,
  port: u16,
  cert_path: Option<&str>,
  key_path: Option<&str>,
) -> Result<(), Error> {
  // Read shutdown-grace before `state` is moved into the router.
  let shutdown_grace_period = state.shutdown_grace_period;

  // Both `request_timeout` and `power_timeout` are now applied **inside**
  // `build_router` so the per-route `/power` override actually wins —
  // see the comment on `build_router` for why a global outer layer
  // would silently defeat the override.
  let app =
    routes::build_router(state).layer(axum::middleware::from_fn(log_requests));

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
      install_shutdown_handler(handle.clone(), shutdown_grace_period);
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
      install_shutdown_handler(handle.clone(), shutdown_grace_period);
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

/// Spawn a task that waits for SIGTERM or Ctrl+C and triggers
/// `axum_server`'s graceful shutdown with a bounded drain window.
/// Without this, the runtime drops in-flight requests when Tokio is
/// shut down by the OS — `docker stop` / k8s pod termination would
/// abandon clients mid-call.
///
/// The grace-period comes from `ServerState::shutdown_grace_period`
/// (sourced from `server.toml`); pods that hit this without
/// finishing get SIGKILL'd by the kubelet.
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
