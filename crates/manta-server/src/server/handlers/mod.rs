//! Top-level Axum handlers module.
//!
//! `mod.rs` keeps:
//! - request extractors (`BearerToken`, `SiteName`, `RequestCtx`, `SiteHeader`)
//! - the `ErrorResponse` body type + error mappers (`to_handler_error`,
//!   `serialize_or_500`)
//! - guard helpers (`require_vault`, `require_k8s_url`,
//!   `validate_repo_list_lengths`, `parse_iso_datetime`)
//! - the cross-handler `resolve_xnames_from_request` helper
//! - the `health` endpoint
//!
//! Every other handler lives in a per-resource sub-module (mirroring
//! the `service/` layout) and is re-exported here so `routes.rs` and
//! `api_doc.rs` can keep referencing `handlers::X` unchanged.

use std::sync::Arc;

use axum::{
  Json,
  extract::FromRequestParts,
  http::{StatusCode, header, request::Parts},
  response::IntoResponse,
};
use manta_backend_dispatcher::error::Error as BackendError;
use serde::Serialize;
use utoipa::{IntoParams, ToSchema};

use super::ServerState;
use super::common::app_context::InfraContext;

mod auth;
mod boot_parameters;
mod cluster;
mod configuration;
mod console;
mod ephemeral_env;
mod group;
mod hardware;
mod hw_cluster;
mod image;
mod kernel_parameters;
mod migrate;
mod node;
mod power;
mod redfish_endpoints;
mod sat_file;
mod session;
mod template;

pub use auth::*;
pub use boot_parameters::*;
pub use cluster::*;
pub use configuration::*;
pub use console::*;
pub use ephemeral_env::*;
pub use group::*;
pub use hardware::*;
pub use hw_cluster::*;
pub use image::*;
pub use kernel_parameters::*;
pub use migrate::*;
pub use node::*;
pub use power::*;
pub use redfish_endpoints::*;
pub use sat_file::*;
pub use session::*;
pub use template::*;

// ---------------------------------------------------------------------------
// Bearer-token extractor — eliminates token-extraction boilerplate
// ---------------------------------------------------------------------------

/// Axum extractor that pulls the token from `Authorization: Bearer <token>`.
pub struct BearerToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for BearerToken {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let auth_header = parts
      .headers
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .ok_or_else(|| {
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorResponse {
            error: "Missing Authorization header".to_string(),
          }),
        )
      })?;

    let token = auth_header
      .strip_prefix("Bearer ")
      .or_else(|| auth_header.strip_prefix("bearer "))
      .ok_or_else(|| {
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorResponse {
            error: "Authorization header must use Bearer scheme".to_string(),
          }),
        )
      })?;

    Ok(BearerToken(token.to_string()))
  }
}

/// Axum extractor that reads the target site name from `X-Manta-Site`.
///
/// Every handler that touches backend APIs requires this header so the server
/// knows which site's CA certificate, base URL, and credentials to use.
pub struct SiteName(pub String);

impl<S: Send + Sync> FromRequestParts<S> for SiteName {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let site = parts
      .headers
      .get("X-Manta-Site")
      .and_then(|v| v.to_str().ok())
      .ok_or_else(|| {
        (
          StatusCode::BAD_REQUEST,
          Json(ErrorResponse {
            error: "Missing X-Manta-Site header".to_string(),
          }),
        )
      })?;
    Ok(SiteName(site.to_string()))
  }
}

/// Required header parameter present on every authenticated endpoint.
///
/// Tells the server which cluster to route the request to.
/// **Not** an authentication mechanism — documented as a plain header parameter.
///
/// The field is consumed by the `utoipa::IntoParams` derive macro at compile
/// time to generate the OpenAPI spec; the runtime extractor is [`SiteName`].
#[derive(IntoParams)]
#[into_params(parameter_in = Header)]
#[allow(dead_code)]
pub struct SiteHeader {
  /// Name of the target cluster (matches a site configured in the server).
  #[param(required = true, rename = "X-Manta-Site")]
  pub x_manta_site: String,
}

// ---------------------------------------------------------------------------
// RequestCtx — bundles the State + BearerToken + SiteName extractors that
// every authenticated handler opens with. Plus `infra()` for the
// `state.infra_context(&site_name).map_err(to_handler_error)?` line that
// follows. Each handler shrinks by 3-4 lines.
// ---------------------------------------------------------------------------

/// Bundled extractor for `State<Arc<ServerState>>` + `BearerToken` +
/// `SiteName`. Use it in handler signatures instead of the three
/// individual extractors when all three are needed (the typical case).
///
/// The unauthenticated `/auth/*` handlers and the health endpoint
/// still use explicit extractors — they don't need a Bearer token.
pub struct RequestCtx {
  /// Shared server state (backend dispatcher, per-site config, TLS
  /// material, optional Vault + k8s URLs).
  pub state: Arc<ServerState>,
  /// Bearer token extracted from the inbound `Authorization` header.
  pub token: String,
  /// Site name extracted from the inbound `X-Manta-Site` header;
  /// used to pick the right `[sites.X]` entry from `state`.
  pub site_name: String,
}

impl FromRequestParts<Arc<ServerState>> for RequestCtx {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    state: &Arc<ServerState>,
  ) -> Result<Self, Self::Rejection> {
    let BearerToken(token) =
      BearerToken::from_request_parts(parts, state).await?;
    let SiteName(site_name) =
      SiteName::from_request_parts(parts, state).await?;
    // Validate the site resolves to a configured backend NOW, so the
    // per-handler `ctx.infra()` call below cannot fail. Returning the
    // 404-mapped error from extraction is the same shape the handler
    // would have produced.
    state.infra_context(&site_name).map_err(to_handler_error)?;
    Ok(Self {
      state: Arc::clone(state),
      token,
      site_name,
    })
  }
}

impl RequestCtx {
  /// Borrow the per-site infrastructure (backend, base URLs, root
  /// cert, optional Vault + k8s URLs). Infallible — the site was
  /// validated during extraction; a missing site would have failed
  /// the request before the handler body ran.
  pub fn infra(&self) -> InfraContext<'_> {
    self
      .state
      .infra_context(&self.site_name)
      .expect("site validated during RequestCtx extraction")
  }
}

/// Render an error and its `source()` chain as a multi-line string.
///
/// `thiserror`'s `Display` only emits the top-level message; nested
/// errors reached via `std::error::Error::source()` are dropped. This
/// walks the chain so the server log carries the full causal context
/// (e.g. the underlying TLS / connect error behind a `reqwest::Error`).
/// Works uniformly for thiserror-derived and `anyhow::Error` chains.
fn format_with_causes(e: &(dyn std::error::Error + 'static)) -> String {
  let mut out = e.to_string();
  let mut src = e.source();
  while let Some(cause) = src {
    out.push_str("\n  caused by: ");
    out.push_str(&cause.to_string());
    src = cause.source();
  }
  out
}

/// Convert a `BackendError` into the best-fitting HTTP error response.
///
/// `pub` (rather than `pub(crate)`) so the integration tests in
/// `crates/manta-server/tests/` can exercise the mapping directly.
//
// `e` is consumed via `e.to_string()` at the end; technically it could
// take `&BackendError`, but the canonical call shape is
// `.map_err(to_handler_error)?` which threads the value through.
// Switching to a reference would force every site to write
// `.map_err(|e| to_handler_error(&e))?` — losing the point-free form
// across hundreds of handler call sites is a worse trade than the
// ineffectual `Drop` here.
#[allow(clippy::needless_pass_by_value)]
pub fn to_handler_error(e: BackendError) -> (StatusCode, Json<ErrorResponse>) {
  let status = match &e {
    BackendError::NotFound(_)
    | BackendError::SessionNotFound
    | BackendError::ConfigurationNotFound => StatusCode::NOT_FOUND,
    BackendError::Conflict(_)
    | BackendError::ConfigurationAlreadyExistsError(_) => StatusCode::CONFLICT,
    BackendError::BadRequest(_)
    | BackendError::InvalidPattern(_)
    | BackendError::UnsupportedBackend(_)
    | BackendError::InvalidNodeId(_) => StatusCode::BAD_REQUEST,
    BackendError::AuthenticationTokenNotFound(_)
    | BackendError::JwtMalformed(_) => StatusCode::UNAUTHORIZED,
    BackendError::InsufficientResources(_) => StatusCode::UNPROCESSABLE_ENTITY,
    _ => StatusCode::INTERNAL_SERVER_ERROR,
  };
  let chain = format_with_causes(&e);
  if status == StatusCode::INTERNAL_SERVER_ERROR {
    tracing::error!("Internal error: {}", chain);
  } else {
    tracing::debug!("Service error {}: {}", status, chain);
  }
  (
    status,
    Json(ErrorResponse {
      error: e.to_string(),
    }),
  )
}

pub(super) fn serialize_or_500<T: Serialize>(
  v: &T,
) -> Result<serde_json::Value, (StatusCode, Json<ErrorResponse>)> {
  serde_json::to_value(v).map_err(|e| {
    let chain = format_with_causes(&e);
    tracing::error!("Failed to serialize: {}", chain);
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        error: format!("Failed to serialize: {e}"),
      }),
    )
  })
}

pub(super) fn require_vault(
  url: Option<&str>,
) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
  url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "vault_base_url not configured on this server".into(),
      }),
    )
  })
}

pub(super) fn require_k8s_url(
  url: Option<&str>,
) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
  url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "k8s_api_url not configured on this server".into(),
      }),
    )
  })
}

pub(super) fn validate_repo_list_lengths(
  repo_names: &[String],
  repo_last_commit_ids: &[String],
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
  if repo_names.len() != repo_last_commit_ids.len() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!(
          "repo_names ({}) and repo_last_commit_ids ({}) must have the same length",
          repo_names.len(),
          repo_last_commit_ids.len()
        ),
      }),
    ));
  }
  Ok(())
}

pub(super) fn default_true() -> bool {
  true
}

pub(super) fn parse_iso_datetime(
  field: &str,
  value: &str,
) -> Result<chrono::NaiveDateTime, (StatusCode, Json<ErrorResponse>)> {
  chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").map_err(
    |e| {
      (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid '{field}' datetime '{value}': {e}"),
        }),
      )
    },
  )
}

// ---------------------------------------------------------------------------
// Shared response types
// ---------------------------------------------------------------------------

/// Standard JSON error body returned by all failed endpoints.
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
  /// Human-readable explanation of the failure. Never includes
  /// stack traces, credentials, or internal type names.
  pub error: String,
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// GET /health — liveness probe; returns `{"status":"ok"}`.
#[utoipa::path(get, path = "/health", tag = "system",
  responses(
    (status = 200, description = "Server is healthy"),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn health() -> impl IntoResponse {
  Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Resolve target xnames from an explicit list or an HSM group name.
/// Returns 400 if neither is provided.
async fn resolve_xnames_from_request(
  infra: &crate::server::common::app_context::InfraContext<'_>,
  token: &str,
  xnames_expression: Option<&str>,
  hsm_group: Option<&str>,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
  if let Some(expr) = xnames_expression
    && !expr.is_empty()
  {
    return crate::service::node_ops::resolve_hosts_expression(
      infra, token, expr, false,
    )
    .await
    .map_err(to_handler_error);
  }
  if let Some(group) = hsm_group {
    return crate::service::node_ops::resolve_target_nodes(
      infra,
      token,
      None,
      Some(group),
      None,
    )
    .await
    .map_err(to_handler_error);
  }
  Err((
    StatusCode::BAD_REQUEST,
    Json(ErrorResponse {
      error: "At least one of 'xnames' or 'hsm_group' must be provided"
        .to_string(),
    }),
  ))
}

#[cfg(test)]
mod tests {
  //! Pure-logic locks for the helpers in this module that don't need
  //! a live router. Route- and error-mapping coverage lives in
  //! `crates/manta-server/tests/server_routes.rs`.

  use super::format_with_causes;
  use std::error::Error;
  use std::fmt;

  /// Toy error whose `source()` returns the inner error, so we can
  /// build a fixed-depth `Display + Error` chain for the walk test.
  #[derive(Debug)]
  struct Chain {
    msg: &'static str,
    src: Option<Box<Chain>>,
  }
  impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str(self.msg)
    }
  }
  impl Error for Chain {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
      self.src.as_deref().map(|s| s as &(dyn Error + 'static))
    }
  }

  #[test]
  fn format_with_causes_single_error_has_no_caused_by() {
    let e = Chain {
      msg: "boom",
      src: None,
    };
    assert_eq!(format_with_causes(&e), "boom");
  }

  #[test]
  fn format_with_causes_two_level_chain_is_indented() {
    let e = Chain {
      msg: "outer",
      src: Some(Box::new(Chain {
        msg: "inner",
        src: None,
      })),
    };
    assert_eq!(format_with_causes(&e), "outer\n  caused by: inner");
  }

  #[test]
  fn format_with_causes_walks_to_the_root() {
    // Deeply nested chain — emulates anyhow's `with_context()` stack.
    let e = Chain {
      msg: "top",
      src: Some(Box::new(Chain {
        msg: "middle",
        src: Some(Box::new(Chain {
          msg: "root",
          src: None,
        })),
      })),
    };
    assert_eq!(
      format_with_causes(&e),
      "top\n  caused by: middle\n  caused by: root"
    );
  }
}
