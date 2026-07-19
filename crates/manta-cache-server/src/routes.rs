//! Router, handlers, and wire types for the lookup API.
//!
//! - `GET /health` — liveness probe, always open.
//! - `GET /api/v1/sites` — cached site names.
//! - `GET /api/v1/lookup/group/{label}` — resolve `group → site`.
//! - `GET /api/v1/lookup/nodes?xnames=…` — resolve an xname list.
//!
//! When `[server] api_token` is set, every `/api/v1/*` route requires
//! `Authorization: Bearer <token>`; `/health` stays open for probes.
//! Error bodies are `{"error": "…"}`, mirroring manta-server's
//! `ErrorResponse` shape.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, middleware};
use manta_cache::Index;
use serde::{Deserialize, Serialize};

/// Shared state: the current index behind a `RwLock` (swapped whole by
/// the periodic refresh) and the optional caller token.
pub struct AppState {
  /// The routing index served by the lookup endpoints.
  pub index: tokio::sync::RwLock<Index>,
  /// When set, `/api/v1/*` requires this bearer token.
  pub api_token: Option<String>,
}

/// `{"error": "…"}` — same wire shape as manta-server's error body.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
  /// Human-readable explanation of the failure.
  pub error: String,
}

fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
  (
    status,
    Json(ErrorBody {
      error: message.into(),
    }),
  )
    .into_response()
}

/// Successful `GET /lookup/group/{label}` body.
#[derive(Debug, Serialize)]
struct GroupLookupResponse {
  /// Site owning the group.
  site: String,
}

/// `GET /lookup/nodes` body. The lookup itself always succeeds; the
/// payload states how far resolution got so the caller (manta-server's
/// Stage-4 integration) can decide what an incomplete or split answer
/// means for its request.
#[derive(Debug, Serialize)]
struct NodesLookupResponse {
  /// The single site every xname resolved to; `null` when the list is
  /// split across sites or any xname is unknown.
  site: Option<String>,
  /// Per-xname resolution for every known xname, sorted.
  resolutions: BTreeMap<String, String>,
  /// Xnames the cache knows nothing about, sorted.
  unknown: Vec<String>,
}

/// Query parameters of `GET /lookup/nodes`.
#[derive(Debug, Deserialize)]
struct NodesQuery {
  /// Comma-separated xname list.
  xnames: Option<String>,
}

/// Build the full application router over the shared state.
pub fn build_router(state: Arc<AppState>) -> Router {
  let api = Router::new()
    .route("/sites", get(get_sites))
    .route("/lookup/group/{label}", get(lookup_group))
    .route("/lookup/nodes", get(lookup_nodes))
    .layer(middleware::from_fn_with_state(
      state.clone(),
      require_bearer,
    ));

  Router::new()
    .route("/health", get(health))
    .nest("/api/v1", api)
    .with_state(state)
}

/// Reject `/api/v1/*` calls without the configured bearer token. A
/// no-op when `api_token` is unset.
async fn require_bearer(
  State(state): State<Arc<AppState>>,
  request: axum::extract::Request,
  next: middleware::Next,
) -> Response {
  let Some(expected) = &state.api_token else {
    return next.run(request).await;
  };
  let presented = request
    .headers()
    .get(axum::http::header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "));
  if presented == Some(expected.as_str()) {
    next.run(request).await
  } else {
    error_response(
      StatusCode::UNAUTHORIZED,
      "missing or invalid bearer token (the cache's [server] api_token)",
    )
  }
}

/// GET /health — liveness probe.
async fn health() -> Json<serde_json::Value> {
  Json(serde_json::json!({ "status": "ok" }))
}

/// GET /api/v1/sites — every site name in the index, sorted.
async fn get_sites(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
  let index = state.index.read().await;
  Json(index.sites().map(str::to_owned).collect())
}

/// GET /api/v1/lookup/group/{label} — resolve a group label to its
/// site, or 404 when the label is unknown.
async fn lookup_group(
  State(state): State<Arc<AppState>>,
  Path(label): Path<String>,
) -> Response {
  let index = state.index.read().await;
  match index.group_to_site(&label) {
    Some(site) => Json(GroupLookupResponse {
      site: site.to_owned(),
    })
    .into_response(),
    None => error_response(
      StatusCode::NOT_FOUND,
      format!("no site found for group '{label}'"),
    ),
  }
}

/// GET /api/v1/lookup/nodes?xnames=x1,x2 — resolve an xname list.
/// Returns 400 only when the query itself is unusable (no xnames);
/// unknown or split resolutions are stated in the 200 payload.
async fn lookup_nodes(
  State(state): State<Arc<AppState>>,
  Query(query): Query<NodesQuery>,
) -> Response {
  let xnames: Vec<&str> = query
    .xnames
    .as_deref()
    .unwrap_or("")
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .collect();
  if xnames.is_empty() {
    return error_response(
      StatusCode::BAD_REQUEST,
      "missing query parameter: xnames (comma-separated xname list)",
    );
  }

  let index = state.index.read().await;
  let mut resolutions = BTreeMap::new();
  let mut unknown = Vec::new();
  for xname in xnames {
    match index.xname_to_site(xname) {
      Some(site) => {
        resolutions.insert(xname.to_owned(), site.to_owned());
      }
      None => unknown.push(xname.to_owned()),
    }
  }
  unknown.sort();
  unknown.dedup();

  // Unanimous iff nothing is unknown and all resolutions agree.
  let mut sites = resolutions.values();
  let site = match sites.next() {
    Some(first) if unknown.is_empty() && sites.all(|s| s == first) => {
      Some(first.clone())
    }
    _ => None,
  };

  Json(NodesLookupResponse {
    site,
    resolutions,
    unknown,
  })
  .into_response()
}

#[cfg(test)]
mod tests {
  use axum::body::Body;
  use axum::http::Request;
  use manta_cache::{NodeMembership, SiteSnapshot};
  use tower::ServiceExt as _;

  use super::*;

  fn sample_state(api_token: Option<&str>) -> Arc<AppState> {
    let index = Index::from_snapshots(vec![
      SiteSnapshot {
        site: "alps".to_owned(),
        labels: vec!["compute".to_owned(), "empty".to_owned()],
        nodes: vec![
          NodeMembership {
            xname: "x1000c0s0b0n0".to_owned(),
            groups: vec!["compute".to_owned()],
          },
          NodeMembership {
            xname: "x1000c0s0b0n1".to_owned(),
            groups: vec!["compute".to_owned()],
          },
        ],
      },
      SiteSnapshot {
        site: "daint".to_owned(),
        labels: vec!["compute_d".to_owned()],
        nodes: vec![NodeMembership {
          xname: "x2000c0s0b0n0".to_owned(),
          groups: vec!["compute_d".to_owned()],
        }],
      },
    ]);
    Arc::new(AppState {
      index: tokio::sync::RwLock::new(index),
      api_token: api_token.map(str::to_owned),
    })
  }

  async fn call(router: Router, uri: &str) -> (StatusCode, serde_json::Value) {
    call_with_auth(router, uri, None).await
  }

  async fn call_with_auth(
    router: Router,
    uri: &str,
    bearer: Option<&str>,
  ) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder().method("GET").uri(uri);
    if let Some(token) = bearer {
      request = request.header("Authorization", format!("Bearer {token}"));
    }
    let response = router
      .oneshot(request.body(Body::empty()).unwrap())
      .await
      .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let json = if bytes.is_empty() {
      serde_json::Value::Null
    } else {
      serde_json::from_slice(&bytes).unwrap()
    };
    (status, json)
  }

  #[tokio::test]
  async fn health_is_open_and_ok() {
    let router = build_router(sample_state(None));
    let (status, body) = call(router, "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
  }

  #[tokio::test]
  async fn sites_lists_sorted_names() {
    let router = build_router(sample_state(None));
    let (status, body) = call(router, "/api/v1/sites").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!(["alps", "daint"]));
  }

  #[tokio::test]
  async fn group_lookup_resolves_and_404s() {
    let router = build_router(sample_state(None));

    let (status, body) =
      call(router.clone(), "/api/v1/lookup/group/compute").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["site"], "alps");

    let (status, body) = call(router, "/api/v1/lookup/group/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("nope"));
  }

  #[tokio::test]
  async fn nodes_lookup_unanimous_sets_site() {
    let router = build_router(sample_state(None));
    let (status, body) = call(
      router,
      "/api/v1/lookup/nodes?xnames=x1000c0s0b0n0,%20x1000c0s0b0n1",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["site"], "alps");
    assert_eq!(body["resolutions"]["x1000c0s0b0n0"], "alps");
    assert_eq!(body["unknown"], serde_json::json!([]));
  }

  #[tokio::test]
  async fn nodes_lookup_split_and_unknown_null_the_site() {
    let router = build_router(sample_state(None));

    // Split across sites: both resolve, site is null.
    let (status, body) = call(
      router.clone(),
      "/api/v1/lookup/nodes?xnames=x1000c0s0b0n0,x2000c0s0b0n0",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["site"], serde_json::Value::Null);
    assert_eq!(body["resolutions"]["x2000c0s0b0n0"], "daint");

    // Unknown xname: listed, and the site is null even though the
    // known one is unanimous.
    let (status, body) = call(
      router,
      "/api/v1/lookup/nodes?xnames=x1000c0s0b0n0,x9999c9s9b9n9",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["site"], serde_json::Value::Null);
    assert_eq!(body["unknown"], serde_json::json!(["x9999c9s9b9n9"]));
  }

  #[tokio::test]
  async fn nodes_lookup_without_xnames_is_400() {
    let router = build_router(sample_state(None));
    let (status, _) = call(router.clone(), "/api/v1/lookup/nodes").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(router, "/api/v1/lookup/nodes?xnames=%20,%20").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn api_token_guards_api_but_not_health() {
    let router = build_router(sample_state(Some("sekrit")));

    let (status, _) = call(router.clone(), "/api/v1/sites").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) =
      call_with_auth(router.clone(), "/api/v1/sites", Some("wrong")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) =
      call_with_auth(router.clone(), "/api/v1/sites", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(router, "/health").await;
    assert_eq!(status, StatusCode::OK);
  }
}
