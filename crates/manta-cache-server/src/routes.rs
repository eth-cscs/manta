//! Router, handlers, and wire types for the lookup + management API.
//!
//! Lookup (read-only; open unless `[server] api_token` is set):
//!
//! - `GET /health` — liveness probe, always open.
//! - `GET /api/v1/sites` — cached site names.
//! - `GET /api/v1/lookup/group/{label}` — resolve `group → site`
//!   (409 with every site when the label exists at several).
//! - `GET /api/v1/lookup/nodes?xnames=…` — resolve an xname list.
//!
//! A group or xname known at several sites is **ambiguous**: the
//! lookups name every site and never pick one.
//!
//! Management (**requires** a configured `api_token` — the endpoints
//! answer 403 when none is set):
//!
//! - `POST /api/v1/refresh` — full re-sync of every site.
//! - `POST /api/v1/refresh/{site}` — re-sync one site.
//! - `GET /api/v1/dump` — debugging dump of everything cached.
//!
//! The refreshes are gated because they trigger a cross-site HTTP
//! fan-out and must not be an open amplification lever. `GET /dump` is
//! gated for a different reason: it is the one endpoint that serves
//! **group member lists**, which the open-by-default stance for the
//! lookups explicitly excludes (ROADMAP "Security stance"). Keeping it
//! operator-only means it stays outside any future per-user filtering
//! question — no user ever sees it.
//!
//! Error bodies are `{"error": "…"}`, mirroring manta-server's
//! `ErrorResponse` shape.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router, middleware};
use manta_cache::{Index, SiteSnapshot};
use serde::{Deserialize, Serialize};

use crate::auth::ServiceAccountTokens;
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
  /// Per-site refresh outcome, keyed by site name. Written by the
  /// refresh paths, read only by `GET /dump` — the lookups do not
  /// consult it.
  pub status: BTreeMap<String, SiteStatus>,
}

/// How the last refresh of one site went.
///
/// The two fields are independent on purpose: a single-site refresh
/// that fails leaves the previous snapshot serving, so a site can
/// legitimately carry both a `refreshed_at` (when its serving data was
/// fetched) and a `last_error` (why it is not newer).
#[derive(Debug, Clone, Default)]
pub struct SiteStatus {
  /// When this site's currently-held snapshot was fetched. `None` when
  /// the cache holds no data for it.
  pub refreshed_at: Option<chrono::DateTime<chrono::Utc>>,
  /// Why the most recent refresh attempt failed. Cleared on success.
  pub last_error: Option<String>,
}

/// Shared application state.
pub struct AppState {
  /// Cache state, swapped/rebuilt by the refresh paths.
  pub cache: tokio::sync::RwLock<CacheState>,
  /// The loaded configuration (sites + api_token), shared with the
  /// refresh plumbing.
  pub config: Arc<CacheServerConfiguration>,
  /// The service account every refresh request gets its token from.
  pub tokens: ServiceAccountTokens,
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

/// `409` body of `GET /lookup/group/{label}` for a label that exists
/// at several sites: the usual `error`, plus every site.
#[derive(Debug, Serialize)]
struct AmbiguousGroupResponse {
  /// Human-readable message.
  error: String,
  /// Every site that has the label, sorted.
  sites: Vec<String>,
}

/// `GET /lookup/nodes` body. The lookup itself always succeeds; the
/// payload states how far resolution got so the caller (manta-server)
/// can decide what an incomplete, split or ambiguous answer means for
/// its request.
#[derive(Debug, Serialize)]
struct NodesLookupResponse {
  /// The single site every xname resolved to; `null` when the list is
  /// split across sites, or any xname is unknown or ambiguous.
  site: Option<String>,
  /// Xnames known at exactly one site, with that site, sorted.
  resolutions: BTreeMap<String, String>,
  /// Xnames known at several sites, with every site, sorted.
  ambiguous: BTreeMap<String, Vec<String>>,
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

/// `GET /dump` body — everything the cache holds, for debugging.
///
/// `groups` and `xnames` are a **bulk mirror of the lookup endpoints**:
/// every site per key, collisions included, no extra resolution. A
/// group or xname with more than one site is exactly what the lookups
/// report as ambiguous.
#[derive(Debug, Serialize)]
struct DumpResponse {
  /// When this dump was rendered (RFC3339, UTC).
  generated_at: String,
  /// The manta-server every site is refreshed from.
  manta_server_url: String,
  /// Who the cache refreshes as. The secret is never included.
  service_account: ServiceAccountDump,
  /// Every **configured** site — including ones absent from the index,
  /// which is the only way a failed site is visible at all.
  sites: BTreeMap<String, SiteDump>,
  /// `group label → site → members` (sorted). More than one site is a
  /// collision.
  groups: BTreeMap<String, BTreeMap<String, Vec<String>>>,
  /// `xname → sites` (sorted). More than one site is a collision.
  xnames: BTreeMap<String, Vec<String>>,
}

/// One configured site: how fresh it is, and whether it made it into
/// the index.
#[derive(Debug, Serialize)]
struct SiteDump {
  /// Whether the index currently holds data for this site.
  in_index: bool,
  /// When the held snapshot was fetched (RFC3339, UTC).
  refreshed_at: Option<String>,
  /// Age of the held snapshot, for reading without date arithmetic.
  age_seconds: Option<i64>,
  /// Why the last refresh attempt failed, if it did.
  last_error: Option<String>,
}

/// The service account the cache refreshes as.
#[derive(Debug, Serialize)]
struct ServiceAccountDump {
  /// Its client id (`MANTA_SA_CLIENT_ID`).
  client_id: String,
  /// The Keycloak issuer it gets its tokens from.
  issuer_url: String,
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
    .route("/dump", get(dump))
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
/// site; 404 when the label is unknown, 409 naming every site when it
/// exists at several.
async fn lookup_group(
  State(state): State<Arc<AppState>>,
  Path(label): Path<String>,
) -> Response {
  let cache = state.cache.read().await;
  match cache.index.group_sites(&label) {
    [] => error_response(
      StatusCode::NOT_FOUND,
      format!("no site found for group '{label}'"),
    ),
    [site] => Json(GroupLookupResponse { site: site.clone() }).into_response(),
    sites => (
      StatusCode::CONFLICT,
      Json(AmbiguousGroupResponse {
        error: format!(
          "group '{label}' exists at several sites: {}",
          sites.join(", ")
        ),
        sites: sites.to_vec(),
      }),
    )
      .into_response(),
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
  let mut ambiguous = BTreeMap::new();
  let mut unknown = Vec::new();
  for xname in xnames {
    match cache.index.xname_sites(xname) {
      [] => unknown.push(xname.to_owned()),
      [site] => {
        resolutions.insert(xname.to_owned(), site.clone());
      }
      sites => {
        ambiguous.insert(xname.to_owned(), sites.to_vec());
      }
    }
  }
  unknown.sort();
  unknown.dedup();

  // Unanimous iff nothing is unknown or ambiguous and all resolutions
  // agree.
  let mut sites = resolutions.values();
  let site = match sites.next() {
    Some(first)
      if unknown.is_empty()
        && ambiguous.is_empty()
        && sites.all(|s| s == first) =>
    {
      Some(first.clone())
    }
    _ => None,
  };

  Json(NodesLookupResponse {
    site,
    resolutions,
    ambiguous,
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
    // A refresh that cannot start (the HTTP client cannot be built) is
    // a local failure, not an upstream one.
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

/// GET /api/v1/dump — everything cached, for debugging. Management-
/// gated: this is the only endpoint that serves group member lists.
async fn dump(State(state): State<Arc<AppState>>) -> Json<DumpResponse> {
  use chrono::SecondsFormat;

  let now = chrono::Utc::now();
  let cache = state.cache.read().await;

  let groups = cache
    .index
    .groups()
    .map(|label| {
      let by_site = cache
        .index
        .group_sites(label)
        .iter()
        .map(|site| {
          let members = cache
            .index
            .group_members(label, site)
            .unwrap_or_default()
            .to_vec();
          (site.clone(), members)
        })
        .collect();
      (label.to_owned(), by_site)
    })
    .collect();

  // `Index::xnames` is HashMap-ordered; the BTreeMap makes dumps
  // diffable across calls.
  let xnames = cache
    .index
    .xnames()
    .map(|(xname, sites)| (xname.to_owned(), sites.to_vec()))
    .collect();

  // Keyed by *configured* site, not indexed site: a site that failed
  // its refresh is absent from the index, and its absence is exactly
  // what someone reading this endpoint is trying to explain.
  let indexed: BTreeSet<&str> = cache.index.sites().collect();
  let sites = state
    .config
    .manta_server
    .sites
    .iter()
    .map(|name| {
      let status = cache.status.get(name);
      let refreshed_at = status.and_then(|s| s.refreshed_at);
      (
        name.clone(),
        SiteDump {
          in_index: indexed.contains(name.as_str()),
          refreshed_at: refreshed_at
            .map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true)),
          age_seconds: refreshed_at.map(|t| (now - t).num_seconds()),
          last_error: status.and_then(|s| s.last_error.clone()),
        },
      )
    })
    .collect();

  Json(DumpResponse {
    generated_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
    manta_server_url: state.config.manta_server.url.clone(),
    service_account: ServiceAccountDump {
      client_id: state.tokens.client_id().to_owned(),
      issuer_url: state.tokens.issuer_url().to_owned(),
    },
    sites,
    groups,
    xnames,
  })
}

#[cfg(test)]
mod tests {
  use axum::body::Body;
  use axum::http::Request;
  use manta_cache::NodeMembership;
  use tower::ServiceExt as _;

  use super::*;
  use crate::auth::test_support;

  /// Config for tests: one unreachable site (nothing listens on the
  /// URL), plus an optional api_token.
  fn test_config(api_token: Option<&str>) -> Arc<CacheServerConfiguration> {
    let mut raw = String::new();
    if let Some(token) = api_token {
      raw.push_str(&format!("[server]\napi_token = \"{token}\"\n\n"));
    }
    raw.push_str(
      "[manta_server]\nurl = \"http://127.0.0.1:9\"\n\
       sites = [\"unreachable\"]\n",
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
    state_from(
      snapshots_to_state(sample_snapshots()),
      test_config(api_token),
    )
  }

  /// Fold snapshots into the served state, dating every site now.
  fn snapshots_to_state(snapshots: Vec<SiteSnapshot>) -> CacheState {
    let status = snapshots
      .iter()
      .map(|s| {
        (
          s.site.clone(),
          SiteStatus {
            refreshed_at: Some(chrono::Utc::now()),
            ..Default::default()
          },
        )
      })
      .collect();
    CacheState {
      index: Index::from_snapshots(snapshots.clone()),
      snapshots: snapshots.into_iter().map(|s| (s.site.clone(), s)).collect(),
      status,
    }
  }

  /// State for tests that never refresh: its service account points at
  /// an issuer nothing listens on.
  fn state_from(
    cache: CacheState,
    config: Arc<CacheServerConfiguration>,
  ) -> Arc<AppState> {
    state_with_tokens(cache, config, test_support::unused_tokens())
  }

  fn state_with_tokens(
    cache: CacheState,
    config: Arc<CacheServerConfiguration>,
    tokens: ServiceAccountTokens,
  ) -> Arc<AppState> {
    Arc::new(AppState {
      cache: tokio::sync::RwLock::new(cache),
      config,
      tokens,
    })
  }

  /// Config whose `[sites]` match the dump fixtures: two sites that
  /// refreshed and one that never did.
  fn dump_config() -> Arc<CacheServerConfiguration> {
    let config: CacheServerConfiguration = toml::from_str(
      r#"
        [server]
        api_token = "sekrit"

        [manta_server]
        url = "https://manta-server.example.ch:8443"
        sites = ["alps", "daint", "prealps"]
      "#,
    )
    .unwrap();
    config.validate().unwrap();
    Arc::new(config)
  }

  /// alps + daint served and 90s old; prealps configured but failed, so
  /// it holds no data at all.
  fn dump_state() -> Arc<AppState> {
    let mut cache = snapshots_to_state(sample_snapshots());
    let ninety_seconds_ago = chrono::Utc::now() - chrono::Duration::seconds(90);
    for status in cache.status.values_mut() {
      status.refreshed_at = Some(ninety_seconds_ago);
    }
    cache.status.insert(
      "prealps".to_owned(),
      SiteStatus {
        last_error: Some("connection refused".to_owned()),
        ..Default::default()
      },
    );
    state_from(cache, dump_config())
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
    assert_eq!(body["ambiguous"], serde_json::json!({}));
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
    let keycloak = test_support::mock_keycloak().await;
    let router = build_router(state_with_tokens(
      snapshots_to_state(sample_snapshots()),
      test_config(Some("sekrit")),
      test_support::tokens_from(&keycloak),
    ));
    // The configured site is unreachable: the refresh succeeds as a
    // call, reports the failure, and the rebuilt index is empty (the
    // sample sites are not in the config, so they drop out).
    let (status, body) =
      send(router, "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["sites"], serde_json::json!([]));
    let failures = body["failures"].as_array().unwrap();
    assert_eq!(failures.len(), 1);
    let failure = failures[0].as_str().unwrap();
    // A transport failure at the site, not a missing token.
    assert!(
      failure.contains("request to site 'unreachable'"),
      "{failure}"
    );
  }

  #[tokio::test]
  async fn management_refresh_success_paths_rebuild_the_index() {
    use wiremock::matchers::{header_regex, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let keycloak = test_support::mock_keycloak().await;
    let server = MockServer::start().await;
    // Only a request carrying a token from the service account's
    // Keycloak is answered.
    Mock::given(method("GET"))
      .and(path("/v2/groups"))
      .and(header_regex("Authorization", r"^Bearer sa-token-\d+$"))
      .respond_with(ResponseTemplate::new(200).set_body_json(
        serde_json::json!([
          {"label": "compute", "members": {"ids": ["x1000c0s0b0n0"]}}
        ]),
      ))
      .mount(&server)
      .await;

    let router = build_router(state_with_tokens(
      CacheState::default(),
      config_for(&server.uri()),
      test_support::tokens_from(&keycloak),
    ));

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

  /// A manta-server that answers `GET /groups` with `200 []` —
  /// what a site looks like when the service account has no HSM roles
  /// there at all.
  async fn mock_site_with_no_groups() -> wiremock::MockServer {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/v2/groups"))
      .respond_with(
        ResponseTemplate::new(200).set_body_json(serde_json::json!([])),
      )
      .mount(&server)
      .await;
    server
  }

  fn config_for(uri: &str) -> Arc<CacheServerConfiguration> {
    let config: CacheServerConfiguration = toml::from_str(&format!(
      "[server]\napi_token = \"sekrit\"\n\n\
       [manta_server]\nurl = \"{uri}\"\nsites = [\"live\"]\n"
    ))
    .unwrap();
    config.validate().unwrap();
    Arc::new(config)
  }

  #[tokio::test]
  async fn a_site_that_returns_no_groups_is_a_failure_not_a_healthy_site() {
    let keycloak = test_support::mock_keycloak().await;
    let server = mock_site_with_no_groups().await;
    let router = build_router(state_with_tokens(
      CacheState::default(),
      config_for(&server.uri()),
      test_support::tokens_from(&keycloak),
    ));

    // The HTTP calls all succeeded, so nothing below comes from a
    // transport error: this is entirely the empty-result rule.
    let (status, body) =
      send(router.clone(), "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["sites"], serde_json::json!([]));
    let failures = body["failures"].as_array().unwrap();
    assert_eq!(failures.len(), 1);
    assert!(
      failures[0].as_str().unwrap().contains("returned no groups"),
      "{failures:?}"
    );

    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    let site = &dump["sites"]["live"];
    // The whole point: it must not read as a fresh, healthy site.
    assert_eq!(site["in_index"], false);
    assert_eq!(site["refreshed_at"], serde_json::Value::Null);
    let error = site["last_error"].as_str().unwrap();
    assert!(error.contains("labels=0 nodes=0"), "{error}");
    assert!(error.contains("token exchange"), "{error}");
    // Which account produced the empty result is most of the diagnosis.
    assert_eq!(
      dump["service_account"]["client_id"],
      test_support::CLIENT_ID
    );
  }

  #[tokio::test]
  async fn a_single_site_refresh_that_returns_no_groups_keeps_the_old_data() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    let keycloak = test_support::mock_keycloak().await;
    let server = mock_site_with_no_groups().await;
    // Start from a good snapshot, then let the site go empty — the
    // sequence an account losing its roles actually produces.
    let mut cache = CacheState::default();
    let snapshot = SiteSnapshot {
      site: "live".to_owned(),
      labels: vec!["compute".to_owned()],
      nodes: vec![NodeMembership {
        xname: "x1000c0s0b0n0".to_owned(),
        groups: vec!["compute".to_owned()],
      }],
    };
    let refreshed_at = chrono::Utc::now() - chrono::Duration::seconds(90);
    cache.index = Index::from_snapshots(vec![snapshot.clone()]);
    cache.snapshots.insert("live".to_owned(), snapshot);
    cache.status.insert(
      "live".to_owned(),
      SiteStatus {
        refreshed_at: Some(refreshed_at),
        ..Default::default()
      },
    );
    let router = build_router(state_with_tokens(
      cache,
      config_for(&server.uri()),
      test_support::tokens_from(&keycloak),
    ));

    let (status, body) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/live",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(
      body["error"]
        .as_str()
        .unwrap()
        .contains("returned no groups"),
      "{body}"
    );

    // Unlike refresh_all, a single-site failure leaves the previous
    // snapshot serving: the group still resolves.
    let (status, looked_up) = send(
      router.clone(),
      "GET",
      "/api/v1/lookup/group/compute",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(looked_up["site"], "live");

    let (_, dump) =
      send(router.clone(), "GET", "/api/v1/dump", Some("sekrit")).await;
    let site = &dump["sites"]["live"];
    assert_eq!(site["in_index"], true);
    assert!(site["refreshed_at"].is_string(), "{site}");
    assert!(
      site["last_error"].as_str().unwrap().contains("no groups"),
      "{site}"
    );

    // And it recovers: once the roles are back, the next refresh clears
    // the error rather than leaving the site flagged forever.
    server.reset().await;
    Mock::given(method("GET"))
      .and(path("/v2/groups"))
      .respond_with(ResponseTemplate::new(200).set_body_json(
        serde_json::json!([
          {"label": "compute", "members": {"ids": ["x1000c0s0b0n0"]}},
          {"label": "gpu", "members": {"ids": ["x1000c0s0b0n0"]}},
        ]),
      ))
      .mount(&server)
      .await;

    let (status, _) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/live",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    assert_eq!(dump["sites"]["live"]["last_error"], serde_json::Value::Null);
  }

  #[tokio::test]
  async fn a_refused_service_account_fails_each_site_without_calling_it() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // A Keycloak that rejects the credentials (wrong secret, disabled
    // client).
    let keycloak = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/.well-known/openid-configuration"))
      .respond_with(ResponseTemplate::new(200).set_body_json(
        serde_json::json!({ "token_endpoint": format!("{}/token", keycloak.uri()) }),
      ))
      .mount(&keycloak)
      .await;
    Mock::given(method("POST"))
      .and(path("/token"))
      .respond_with(ResponseTemplate::new(401).set_body_json(
        serde_json::json!({
          "error": "unauthorized_client",
          "error_description": "Invalid client or Invalid client credentials",
        }),
      ))
      .mount(&keycloak)
      .await;
    let server = MockServer::start().await;
    let router = build_router(state_with_tokens(
      CacheState::default(),
      config_for(&server.uri()),
      test_support::tokens_from(&keycloak),
    ));

    let (status, body) =
      send(router.clone(), "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    let failures = body["failures"].as_array().unwrap();
    assert_eq!(failures.len(), 1);
    let failure = failures[0].as_str().unwrap();
    assert!(failure.contains("no token for site 'live'"), "{failure}");
    assert!(failure.contains("Invalid client"), "{failure}");
    // Nothing goes to manta-server without a token.
    assert!(server.received_requests().await.unwrap().is_empty());

    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    let error = dump["sites"]["live"]["last_error"].as_str().unwrap();
    assert!(error.contains("no token"), "{error}");
  }

  #[tokio::test]
  async fn dump_is_gated_like_the_other_management_endpoints() {
    // No api_token configured: disabled outright, not open.
    let router = build_router(sample_state(None));
    let (status, body) = send(router, "GET", "/api/v1/dump", None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body["error"].as_str().unwrap().contains("disabled"));

    let router = build_router(dump_state());
    let (status, _) = send(router.clone(), "GET", "/api/v1/dump", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) =
      send(router.clone(), "GET", "/api/v1/dump", Some("wrong")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
  }

  #[tokio::test]
  async fn dump_mirrors_what_the_lookups_answer() {
    let router = build_router(dump_state());
    let (_, dump) =
      send(router.clone(), "GET", "/api/v1/dump", Some("sekrit")).await;

    // Every group in the dump resolves identically through the lookup
    // endpoint — the dump is a bulk mirror, not a second resolution.
    // Nothing collides in this fixture, so each has exactly one site.
    for (label, by_site) in dump["groups"].as_object().unwrap() {
      let (status, looked_up) = send(
        router.clone(),
        "GET",
        &format!("/api/v1/lookup/group/{label}"),
        Some("sekrit"),
      )
      .await;
      assert_eq!(status, StatusCode::OK, "{label}");
      let sites: Vec<&String> = by_site.as_object().unwrap().keys().collect();
      assert_eq!(sites, [looked_up["site"].as_str().unwrap()], "{label}");
    }

    assert_eq!(
      dump["groups"]["compute"]["alps"],
      serde_json::json!(["x1000c0s0b0n0", "x1000c0s0b0n1"])
    );
    // A known-but-empty group is present with an empty member list, not
    // absent — same as `group_members` returning `Some(&[])`.
    assert_eq!(dump["groups"]["empty"]["alps"], serde_json::json!([]));
    assert_eq!(
      dump["xnames"]["x2000c0s0b0n0"],
      serde_json::json!(["daint"])
    );
    assert!(dump["generated_at"].as_str().unwrap().ends_with('Z'));
  }

  #[tokio::test]
  async fn dump_lists_configured_sites_and_the_service_account() {
    let router = build_router(dump_state());
    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    let sites = &dump["sites"];

    assert_eq!(sites["alps"]["in_index"], true);
    // A range, not `== 90`: the fixture stamps `now - 90s` and the
    // handler recomputes against its own `now`, so any scheduling gap
    // between the two pushes this to 91 on a loaded runner.
    let age = sites["alps"]["age_seconds"].as_i64().unwrap();
    assert!((90..=95).contains(&age), "age_seconds was {age}");
    assert!(
      sites["alps"]["refreshed_at"]
        .as_str()
        .unwrap()
        .ends_with('Z')
    );
    assert_eq!(sites["alps"]["last_error"], serde_json::Value::Null);
    assert_eq!(
      dump["manta_server_url"],
      "https://manta-server.example.ch:8443"
    );

    // Who the cache refreshes as, never the secret.
    assert_eq!(
      dump["service_account"]["client_id"],
      test_support::CLIENT_ID
    );
    assert_eq!(dump["service_account"]["issuer_url"], "http://127.0.0.1:9");
    assert!(
      !dump.to_string().contains(test_support::CLIENT_SECRET),
      "the dump leaked the client secret"
    );

    // A configured site that never refreshed is present and explains
    // itself — the whole reason `sites` is keyed by configured site.
    assert_eq!(sites["prealps"]["in_index"], false);
    assert_eq!(sites["prealps"]["refreshed_at"], serde_json::Value::Null);
    assert_eq!(sites["prealps"]["last_error"], "connection refused");
  }

  /// `nodes_free` is a conventional pool name, so the same label really
  /// does show up at several sites; `x9000c0s0b0n0` is reported by both.
  fn colliding_state() -> Arc<AppState> {
    let node = |xname: &str| NodeMembership {
      xname: xname.to_owned(),
      groups: vec!["nodes_free".to_owned()],
    };
    let snapshots = vec![
      SiteSnapshot {
        site: "alps".to_owned(),
        labels: vec!["nodes_free".to_owned()],
        nodes: vec![node("x1000c0s0b0n0"), node("x9000c0s0b0n0")],
      },
      SiteSnapshot {
        site: "daint".to_owned(),
        labels: vec!["nodes_free".to_owned()],
        nodes: vec![node("x2000c0s0b0n0"), node("x9000c0s0b0n0")],
      },
    ];
    state_from(snapshots_to_state(snapshots), dump_config())
  }

  #[tokio::test]
  async fn group_lookup_of_a_colliding_label_is_a_409_naming_every_site() {
    let router = build_router(colliding_state());
    let (status, body) = send(
      router,
      "GET",
      "/api/v1/lookup/group/nodes_free",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["sites"], serde_json::json!(["alps", "daint"]));
    let error = body["error"].as_str().unwrap();
    assert!(error.contains("alps, daint"), "{error}");
  }

  #[tokio::test]
  async fn nodes_lookup_lists_colliding_xnames_as_ambiguous() {
    let router = build_router(colliding_state());
    let (status, body) = send(
      router,
      "GET",
      "/api/v1/lookup/nodes?xnames=x1000c0s0b0n0,x9000c0s0b0n0",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // x1000… alone would resolve to alps, but x9000… could be either.
    assert_eq!(body["site"], serde_json::Value::Null);
    assert_eq!(body["resolutions"]["x1000c0s0b0n0"], "alps");
    assert_eq!(
      body["ambiguous"]["x9000c0s0b0n0"],
      serde_json::json!(["alps", "daint"])
    );
    assert_eq!(body["unknown"], serde_json::json!([]));
  }

  #[tokio::test]
  async fn dump_shows_every_site_of_a_collision_with_its_own_members() {
    let router = build_router(colliding_state());
    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;

    assert_eq!(
      dump["groups"]["nodes_free"],
      serde_json::json!({
        "alps": ["x1000c0s0b0n0", "x9000c0s0b0n0"],
        "daint": ["x2000c0s0b0n0", "x9000c0s0b0n0"],
      })
    );
    assert_eq!(
      dump["xnames"]["x9000c0s0b0n0"],
      serde_json::json!(["alps", "daint"])
    );
    assert_eq!(dump["xnames"]["x1000c0s0b0n0"], serde_json::json!(["alps"]));
  }

  #[tokio::test]
  async fn refresh_all_leaves_a_failed_site_with_no_timestamp() {
    // Drives the real refresh path rather than hand-building status: a
    // full refresh replaces the store wholesale, so the failed site is
    // left holding nothing, and the dump has to say why.
    let router = build_router(sample_state(Some("sekrit")));
    let (status, _) =
      send(router.clone(), "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);

    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    let site = &dump["sites"]["unreachable"];
    assert_eq!(site["in_index"], false);
    assert_eq!(site["refreshed_at"], serde_json::Value::Null);
    assert_eq!(site["age_seconds"], serde_json::Value::Null);
    assert!(
      site["last_error"].as_str().unwrap().contains("unreachable"),
      "{site}"
    );
  }

  #[tokio::test]
  async fn failed_single_site_refresh_keeps_the_serving_timestamp() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/v2/groups"))
      .respond_with(ResponseTemplate::new(200).set_body_json(
        serde_json::json!([
          {"label": "compute", "members": {"ids": ["x1000c0s0b0n0"]}}
        ]),
      ))
      .mount(&server)
      .await;

    let keycloak = test_support::mock_keycloak().await;
    let router = build_router(state_with_tokens(
      CacheState::default(),
      config_for(&server.uri()),
      test_support::tokens_from(&keycloak),
    ));

    let (status, _) =
      send(router.clone(), "POST", "/api/v1/refresh", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    let (_, dump) =
      send(router.clone(), "GET", "/api/v1/dump", Some("sekrit")).await;
    let refreshed_at = dump["sites"]["live"]["refreshed_at"]
      .as_str()
      .expect("a successful refresh dates the site")
      .to_owned();

    // The site's manta-server stops answering (reset drops every mount,
    // so the group endpoints now 404). Dropping the MockServer instead
    // is not deterministic here — shutdown is asynchronous and the next
    // request can still be served. The snapshot already fetched keeps
    // serving regardless.
    server.reset().await;
    let (status, _) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/live",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);

    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;
    let site = &dump["sites"]["live"];
    // Still serving, still dated from the fetch that produced the data
    // being served — with the failure recorded *alongside* it, not
    // replacing it. This is the semantic that differs from refresh_all.
    assert_eq!(site["in_index"], true);
    assert_eq!(site["refreshed_at"], refreshed_at);
    assert!(
      site["last_error"].as_str().unwrap().contains("live"),
      "{site}"
    );
    assert!(dump["groups"]["compute"]["live"].is_array(), "{dump}");
  }

  #[tokio::test]
  async fn refresh_site_404s_on_unknown_and_502s_on_unreachable() {
    let keycloak = test_support::mock_keycloak().await;
    let router = build_router(state_with_tokens(
      snapshots_to_state(sample_snapshots()),
      test_config(Some("sekrit")),
      test_support::tokens_from(&keycloak),
    ));

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
    let (status, body) = send(
      router.clone(),
      "POST",
      "/api/v1/refresh/unreachable",
      Some("sekrit"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let error = body["error"].as_str().unwrap();
    assert!(error.contains("request to site 'unreachable'"), "{error}");
    let (status, body) =
      send(router, "GET", "/api/v1/sites", Some("sekrit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!(["alps", "daint"]));
  }
}
