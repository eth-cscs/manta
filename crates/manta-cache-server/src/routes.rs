//! Router, handlers, and wire types for the lookup + management API.
//!
//! Lookup (read-only; open unless `[server] api_token` is set):
//!
//! - `GET /health` — liveness probe, always open.
//! - `GET /api/v1/sites` — cached site names.
//! - `GET /api/v1/lookup/group/{label}` — resolve `group → site`.
//! - `GET /api/v1/lookup/nodes?xnames=…` — resolve an xname list.
//!
//! Management (mutating; **requires** a configured `api_token` — the
//! endpoints answer 403 when none is set, because a refresh triggers a
//! cross-site HTTP fan-out and must not be an open amplification
//! lever):
//!
//! - `POST /api/v1/refresh` — full re-sync of every site.
//! - `POST /api/v1/refresh/{site}` — re-sync one site.
//!
//! Error bodies are `{"error": "…"}`, mirroring manta-server's
//! `ErrorResponse` shape.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router, middleware};
use manta_cache::{Index, SiteSnapshot};
use serde::{Deserialize, Serialize};

use crate::config::CacheServerConfiguration;
use crate::refresh::{self, SiteRefreshError};

/// The cache proper: the served index plus the last-good snapshot per
/// site, kept so a single-site refresh can rebuild the index without
/// re-fetching every other site.
#[derive(Default)]
pub struct CacheState {
  /// The routing index served by the lookup endpoints.
  pub index: Index,
  /// Last-good snapshot per site, keyed by site name.
  pub snapshots: BTreeMap<String, SiteSnapshot>,
}

/// Shared application state.
pub struct AppState {
  /// Cache state, swapped/rebuilt by the refresh paths.
  pub cache: tokio::sync::RwLock<CacheState>,
  /// The loaded configuration (sites + api_token), shared with the
  /// refresh plumbing.
  pub config: Arc<CacheServerConfiguration>,
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

/// `POST /refresh` body: what the re-synced index covers and which
/// sites failed (their entries are absent until a refresh reaches
/// them).
#[derive(Debug, Serialize)]
struct RefreshResponse {
  /// Site names present in the rebuilt index, sorted.
  sites: Vec<String>,
  /// One message per site that failed its re-sync.
  failures: Vec<String>,
}

/// Build the full application router over the shared state.
pub fn build_router(state: Arc<AppState>) -> Router {
  let lookup = Router::new()
    .route("/sites", get(get_sites))
    .route("/lookup/group/{label}", get(lookup_group))
    .route("/lookup/nodes", get(lookup_nodes))
    .layer(middleware::from_fn_with_state(
      state.clone(),
      require_bearer,
    ));

  let management = Router::new()
    .route("/refresh", post(refresh_all_handler))
    .route("/refresh/{site}", post(refresh_site_handler))
    .layer(middleware::from_fn_with_state(
      state.clone(),
      require_management,
    ));

  Router::new()
    .route("/health", get(health))
    .nest("/api/v1", lookup.merge(management))
    .with_state(state)
}

/// Reject lookup calls without the configured bearer token. A no-op
/// when `api_token` is unset — the lookups serve read-only routing
/// metadata considered non-sensitive inside the deployment perimeter.
async fn require_bearer(
  State(state): State<Arc<AppState>>,
  request: axum::extract::Request,
  next: middleware::Next,
) -> Response {
  let Some(expected) = &state.config.server.api_token else {
    return next.run(request).await;
  };
  if bearer_matches(&request, expected) {
    next.run(request).await
  } else {
    error_response(
      StatusCode::UNAUTHORIZED,
      "missing or invalid bearer token (the cache's [server] api_token)",
    )
  }
}

/// Gate management calls: 403 when no `api_token` is configured (the
/// mutating endpoints stay disabled rather than open), 401 on a
/// missing/wrong token otherwise.
async fn require_management(
  State(state): State<Arc<AppState>>,
  request: axum::extract::Request,
  next: middleware::Next,
) -> Response {
  match &state.config.server.api_token {
    None => error_response(
      StatusCode::FORBIDDEN,
      "management endpoints are disabled: set [server] api_token in \
       cache-server.toml to enable them",
    ),
    Some(expected) if bearer_matches(&request, expected) => {
      next.run(request).await
    }
    Some(_) => error_response(
      StatusCode::UNAUTHORIZED,
      "missing or invalid bearer token (the cache's [server] api_token)",
    ),
  }
}

/// Does the request carry `Authorization: Bearer <expected>`?
fn bearer_matches(request: &axum::extract::Request, expected: &str) -> bool {
  request
    .headers()
    .get(axum::http::header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "))
    == Some(expected)
}

/// GET /health — liveness probe.
async fn health() -> Json<serde_json::Value> {
  Json(serde_json::json!({ "status": "ok" }))
}

/// GET /api/v1/sites — every site name in the index, sorted.
async fn get_sites(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
  let cache = state.cache.read().await;
  Json(cache.index.sites().map(str::to_owned).collect())
}

/// GET /api/v1/lookup/group/{label} — resolve a group label to its
/// site, or 404 when the label is unknown.
async fn lookup_group(
  State(state): State<Arc<AppState>>,
  Path(label): Path<String>,
) -> Response {
  let cache = state.cache.read().await;
  match cache.index.group_to_site(&label) {
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

  let cache = state.cache.read().await;
  let mut resolutions = BTreeMap::new();
  let mut unknown = Vec::new();
  for xname in xnames {
    match cache.index.xname_to_site(xname) {
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

/// POST /api/v1/refresh — full re-sync of every configured site.
async fn refresh_all_handler(State(state): State<Arc<AppState>>) -> Response {
  match refresh::refresh_all(&state).await {
    Ok(failures) => {
      let cache = state.cache.read().await;
      Json(RefreshResponse {
        sites: cache.index.sites().map(str::to_owned).collect(),
        failures,
      })
      .into_response()
    }
    // Could-not-start failures (unreadable token_file, client build)
    // are operator errors, not upstream ones.
    Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e),
  }
}

/// POST /api/v1/refresh/{site} — re-sync one site; the rest of the
/// index is rebuilt from stored snapshots.
async fn refresh_site_handler(
  State(state): State<Arc<AppState>>,
  Path(site): Path<String>,
) -> Response {
  match refresh::refresh_site(&state, &site).await {
    Ok(()) => Json(serde_json::json!({ "refreshed": site })).into_response(),
    Err(SiteRefreshError::UnknownSite) => error_response(
      StatusCode::NOT_FOUND,
      format!("site '{site}' is not configured"),
    ),
    // The site exists but its manta-server did not deliver: upstream
    // failure, previous state kept.
    Err(SiteRefreshError::Fetch(msg)) => {
      error_response(StatusCode::BAD_GATEWAY, msg)
    }
  }
}

#[cfg(test)]
mod tests {
  use axum::body::Body;
  use axum::http::Request;
  use manta_cache::NodeMembership;
  use tower::ServiceExt as _;

  use super::*;

  /// Config for tests: one unreachable site (nothing listens on the
  /// URL), plus an optional api_token.
  fn test_config(api_token: Option<&str>) -> Arc<CacheServerConfiguration> {
    let mut raw = String::new();
    if let Some(token) = api_token {
      raw.push_str(&format!("[server]\napi_token = \"{token}\"\n\n"));
    }
    raw.push_str(
      "[sites.unreachable]\n\
       manta_server_url = \"http://127.0.0.1:9\"\n\
       token = \"t\"\n",
    );
    let config: CacheServerConfiguration = toml::from_str(&raw).unwrap();
    config.validate().unwrap();
    Arc::new(config)
  }

  fn sample_snapshots() -> Vec<SiteSnapshot> {
    vec![
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
    ]
  }

  fn sample_state(api_token: Option<&str>) -> Arc<AppState> {
    let snapshots = sample_snapshots();
    Arc::new(AppState {
      cache: tokio::sync::RwLock::new(CacheState {
        index: Index::from_snapshots(snapshots.clone()),
        snapshots: snapshots.into_iter().map(|s| (s.site.clone(), s)).collect(),
      }),
      config: test_config(api_token),
    })
  }

  async fn send(
    router: Router,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
  ) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder().method(method).uri(uri);
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

  async fn call(router: Router, uri: &str) -> (StatusCode, serde_json::Value) {
    send(router, "GET", uri, None).await
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
      send(router.clone(), "GET", "/api/v1/sites", Some("wrong")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) =
      send(router.clone(), "GET", "/api/v1/sites", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(router, "/health").await;
    assert_eq!(status, StatusCode::OK);
  }

  #[tokio::test]
  async fn management_is_disabled_without_api_token() {
    let router = build_router(sample_state(None));
    let (status, body) = send(router, "POST", "/api/v1/refresh", None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body["error"].as_str().unwrap().contains("disabled"));
  }

  #[tokio::test]
  async fn management_requires_the_bearer_token() {
    let router = build_router(sample_state(Some("sekrit")));
    let (status, _) =
      send(router.clone(), "POST", "/api/v1/refresh", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) =
      send(router, "POST", "/api/v1/refresh", Some("wrong")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn refresh_all_reports_failures_and_replaces_state() {
    let router = build_router(sample_state(Some("sekrit")));
    // The configured site is unreachable: the refresh succeeds as a
    // call, reports the failure, and the rebuilt index is empty (the
    // sample sites are not in the config, so they drop out).
    let (status, body) =
      send(router, "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["sites"], serde_json::json!([]));
    let failures = body["failures"].as_array().unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].as_str().unwrap().contains("unreachable"));
  }

  #[tokio::test]
  async fn management_refresh_success_paths_rebuild_the_index() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/api/v1/groups/available"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_json(serde_json::json!(["compute"])),
      )
      .mount(&server)
      .await;
    Mock::given(method("GET"))
      .and(path("/api/v1/groups/nodes"))
      .respond_with(ResponseTemplate::new(200).set_body_json(
        serde_json::json!([{"xname": "x1000c0s0b0n0", "hsm": "compute"}]),
      ))
      .mount(&server)
      .await;

    let raw = format!(
      "[server]\napi_token = \"sekrit\"\n\n\
       [sites.live]\nmanta_server_url = \"{}\"\ntoken = \"t\"\n",
      server.uri()
    );
    let config: CacheServerConfiguration = toml::from_str(&raw).unwrap();
    config.validate().unwrap();
    let state = Arc::new(AppState {
      cache: tokio::sync::RwLock::new(CacheState::default()),
      config: Arc::new(config),
    });
    let router = build_router(state);

    // Full refresh populates the empty cache from the mock.
    let (status, body) =
      send(router.clone(), "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["sites"], serde_json::json!(["live"]));
    assert_eq!(body["failures"], serde_json::json!([]));

    // Single-site refresh succeeds and the lookups serve the data.
    let (status, body) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/live",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["refreshed"], "live");

    let (status, body) = send(
      router,
      "GET",
      "/api/v1/lookup/group/compute",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["site"], "live");
  }

  #[tokio::test]
  async fn refresh_site_404s_on_unknown_and_502s_on_unreachable() {
    let router = build_router(sample_state(Some("sekrit")));

    let (status, body) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/nope",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("nope"));

    // Configured but unreachable → 502, and the previous index keeps
    // serving.
    let (status, _) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/unreachable",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let (status, body) =
      send(router, "GET", "/api/v1/sites", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!(["alps", "daint"]));
  }
}
