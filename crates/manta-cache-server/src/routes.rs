//! Router, handlers, and wire types for the lookup + management API.
//!
//! Lookup (read-only; open unless `[server] api_token` is set):
//!
//! - `GET /health` — liveness probe, always open.
//! - `GET /api/v1/sites` — cached site names.
//! - `GET /api/v1/lookup/group/{label}` — resolve `group → site`.
//! - `GET /api/v1/lookup/nodes?xnames=…` — resolve an xname list.
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

/// `GET /dump` body — everything the cache holds, for debugging.
///
/// `groups` and `xnames` are a **bulk mirror of the lookup endpoints**:
/// same owners, same member lists, no extra resolution. Whatever
/// `GET /lookup/group/{label}` would answer is what appears here.
///
/// `conflicts` is the one deliberate exception — it is derived from the
/// stored snapshots, not the index, and reports collisions the index
/// cannot express (see [`ConflictsDump`]).
#[derive(Debug, Serialize)]
struct DumpResponse {
  /// When this dump was rendered (RFC3339, UTC).
  generated_at: String,
  /// Every **configured** site — including ones absent from the index,
  /// which is the only way a failed site is visible at all.
  sites: BTreeMap<String, SiteDump>,
  /// `group label → owning site + members`, mirroring the lookups.
  groups: BTreeMap<String, GroupDump>,
  /// `xname → owning site`, mirroring the lookups.
  xnames: BTreeMap<String, String>,
  /// Cross-site collisions behind the entries above.
  conflicts: ConflictsDump,
}

/// One configured site: where it points, how fresh it is, and whether
/// it made it into the index.
#[derive(Debug, Serialize)]
struct SiteDump {
  /// Base URL the refresh fetches from.
  manta_server_url: String,
  /// Where the credential comes from — never the credential itself.
  token_source: TokenSource,
  /// Whether the index currently holds data for this site.
  in_index: bool,
  /// When the held snapshot was fetched (RFC3339, UTC).
  refreshed_at: Option<String>,
  /// Age of the held snapshot, for reading without date arithmetic.
  age_seconds: Option<i64>,
  /// Why the last refresh attempt failed, if it did.
  last_error: Option<String>,
}

/// Provenance of a site's service-account token.
///
/// Serialises as `"inline"` or `{"file": "/path"}`. The token value is
/// never included: catching "it is reading the wrong token file" needs
/// the path, not the secret.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum TokenSource {
  /// `token = "…"` in `cache-server.toml`.
  Inline,
  /// `token_file = "…"`, carrying the configured path.
  File(String),
}

/// One group as the lookups resolve it.
#[derive(Debug, Serialize)]
struct GroupDump {
  /// Site owning the label.
  site: String,
  /// Members, sorted — the owning site's nodes only.
  members: Vec<String>,
}

/// Collisions found in the snapshots, which the index cannot express.
///
/// [`manta_cache::Index`] resolves conflicts at build time and stores
/// only the winner, so a group whose members were discarded when
/// another site claimed the label is indistinguishable from a genuinely
/// small group. This section recovers that context by re-scanning the
/// stored snapshots at dump time. Empty in the common case.
#[derive(Debug, Default, Serialize)]
struct ConflictsDump {
  /// Labels more than one site contributed data for.
  groups: Vec<GroupConflict>,
  /// Xnames reported by more than one site.
  xnames: Vec<XnameConflict>,
}

/// A group label contested by several sites.
#[derive(Debug, Serialize)]
struct GroupConflict {
  /// The contested label.
  label: String,
  /// Every site that listed the label **or** has a node claiming it,
  /// sorted.
  claimed_by: Vec<String>,
  /// The subset of `claimed_by` that listed the label in
  /// `/groups/available`, sorted. Separate from `claimed_by` because
  /// the tie-break runs over *listings*, not contributions — applying
  /// `reason` to `claimed_by` would name the wrong site whenever a
  /// third site merely mentions the label through a node's `hsm`.
  listed_by: Vec<String>,
  /// The site that owns it in the index — what lookups answer.
  owner: String,
  /// Which of the documented tie-break rules applied.
  reason: &'static str,
  /// The xnames non-owning sites claim for this label that are missing
  /// from the served member list, sorted.
  ///
  /// Exactly "the nodes absent from `groups[label].members` because of
  /// this collision" — empty when the sites disagree about ownership but
  /// not about membership, since an xname the owner contributes anyway
  /// was never lost. Listed rather than counted because the dump is
  /// otherwise index-only: these xnames appear nowhere else in the
  /// payload, so a count would say a node went missing without saying
  /// which, leaving the reader to go ask the losing site's
  /// manta-server — the round trip this endpoint exists to save.
  discarded_members: Vec<String>,
}

/// An xname reported by several sites.
#[derive(Debug, Serialize)]
struct XnameConflict {
  /// The contested xname.
  xname: String,
  /// Every site whose snapshot contains it, sorted.
  reported_by: Vec<String>,
  /// The site that owns it in the index — what lookups answer.
  owner: String,
  /// Which tie-break applied. Constant: unlike group labels, xnames
  /// have no listing/mention distinction, so the last writer always
  /// wins. Stated per entry so the payload explains itself.
  reason: &'static str,
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

/// GET /api/v1/dump — everything cached, for debugging. Management-
/// gated: this is the only endpoint that serves group member lists.
async fn dump(State(state): State<Arc<AppState>>) -> Json<DumpResponse> {
  use chrono::SecondsFormat;

  let now = chrono::Utc::now();
  let cache = state.cache.read().await;

  // Both index lookups are total over `groups()` — every label in the
  // membership map also has an owner — so the defaults never apply.
  let groups = cache
    .index
    .groups()
    .map(|label| {
      (
        label.to_owned(),
        GroupDump {
          site: cache
            .index
            .group_to_site(label)
            .unwrap_or_default()
            .to_owned(),
          members: cache
            .index
            .group_members(label)
            .unwrap_or_default()
            .to_vec(),
        },
      )
    })
    .collect();

  // `Index::xnames` is HashMap-ordered; the BTreeMap makes dumps
  // diffable across calls.
  let xnames = cache
    .index
    .xnames()
    .map(|(xname, site)| (xname.to_owned(), site.to_owned()))
    .collect();

  // Keyed by *configured* site, not indexed site: a site that failed
  // its refresh is absent from the index, and its absence is exactly
  // what someone reading this endpoint is trying to explain.
  let indexed: BTreeSet<&str> = cache.index.sites().collect();
  let sites = state
    .config
    .sites
    .iter()
    .map(|(name, site)| {
      let status = cache.status.get(name);
      let refreshed_at = status.and_then(|s| s.refreshed_at);
      (
        name.clone(),
        SiteDump {
          manta_server_url: site.manta_server_url.clone(),
          token_source: match &site.token_file {
            Some(path) => TokenSource::File(path.display().to_string()),
            None => TokenSource::Inline,
          },
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
    sites,
    groups,
    xnames,
    conflicts: conflicts(&cache),
  })
}

/// Re-scan the stored snapshots for cross-site collisions.
///
/// Runs per request rather than being maintained incrementally: this is
/// a hand-called debugging endpoint, and one pass over the snapshots is
/// cheaper than keeping a parallel structure correct across every
/// refresh path.
fn conflicts(cache: &CacheState) -> ConflictsDump {
  // label → site → the xnames that site claims for the label. A listed
  // label always gets an entry (possibly empty), so a label two sites
  // list with no members is still seen as contested.
  let mut membership: BTreeMap<&str, BTreeMap<&str, BTreeSet<&str>>> =
    BTreeMap::new();
  // label → the sites that listed it in `/groups/available`, which is
  // what distinguishes a claim from a bare node mention. A set of
  // *sites*, not a count of listings: one site repeating a label in its
  // own response must not read as two sites claiming it.
  let mut listed: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
  // xname → sites that reported it.
  let mut reported: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();

  for snapshot in cache.snapshots.values() {
    let site = snapshot.site.as_str();
    for label in &snapshot.labels {
      listed.entry(label).or_default().insert(site);
      membership
        .entry(label)
        .or_default()
        .entry(site)
        .or_default();
    }
    for node in &snapshot.nodes {
      reported.entry(&node.xname).or_default().insert(site);
      for label in &node.groups {
        membership
          .entry(label)
          .or_default()
          .entry(site)
          .or_default()
          .insert(&node.xname);
      }
    }
  }

  let groups = membership
    .iter()
    .filter(|(_, by_site)| by_site.len() > 1)
    .filter_map(|(label, by_site)| {
      let owner = cache.index.group_to_site(label)?;
      // What the index actually serves for this label. An xname a
      // non-owning site claims but that still appears here was not lost
      // to the collision, so it must not count as discarded.
      let served: BTreeSet<&str> = cache
        .index
        .group_members(label)
        .unwrap_or_default()
        .iter()
        .map(String::as_str)
        .collect();
      let discarded: BTreeSet<&str> = by_site
        .iter()
        .filter(|(site, _)| **site != owner)
        .flat_map(|(_, xnames)| xnames.iter().copied())
        .filter(|xname| !served.contains(xname))
        .collect();
      static NO_SITES: BTreeSet<&str> = BTreeSet::new();
      let listed_by = listed.get(label).unwrap_or(&NO_SITES);
      Some(GroupConflict {
        label: (*label).to_owned(),
        claimed_by: by_site.keys().map(|s| (*s).to_owned()).collect(),
        listed_by: listed_by.iter().map(|s| (*s).to_owned()).collect(),
        owner: owner.to_owned(),
        // Mirrors the tie-breaks documented on `manta_cache::Index`.
        // Snapshots are folded in site-name order (see refresh::rebuild),
        // so "last write wins" is "last in site-name order" — over the
        // sites in `listed_by`, which is not always all of `claimed_by`.
        reason: match listed_by.len() {
          0 => {
            "no site listed this label — it is known only from node \
             membership, so the first of claimed_by in site-name order \
             owns it"
          }
          1 => {
            "one site listed this label; a listing outranks another \
             site's node membership, whatever the order"
          }
          _ => {
            "several sites listed this label; the last of listed_by in \
             site-name order owns it"
          }
        },
        // Already sorted and de-duplicated: it is a BTreeSet.
        discarded_members: discarded.iter().map(|x| (*x).to_owned()).collect(),
      })
    })
    .collect();

  let xnames = reported
    .iter()
    .filter(|(_, sites)| sites.len() > 1)
    .filter_map(|(xname, sites)| {
      Some(XnameConflict {
        xname: (*xname).to_owned(),
        reported_by: sites.iter().map(|s| (*s).to_owned()).collect(),
        owner: cache.index.xname_to_site(xname)?.to_owned(),
        reason: "reported by several sites; the last in site-name order \
                 owns it",
      })
    })
    .collect();

  ConflictsDump { groups, xnames }
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
            last_error: None,
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

  fn state_from(
    cache: CacheState,
    config: Arc<CacheServerConfiguration>,
  ) -> Arc<AppState> {
    Arc::new(AppState {
      cache: tokio::sync::RwLock::new(cache),
      config,
    })
  }

  /// Config whose `[sites]` match the dump fixtures: two sites that
  /// refreshed and one that never did. Exercises both token forms.
  fn dump_config() -> Arc<CacheServerConfiguration> {
    let config: CacheServerConfiguration = toml::from_str(
      r#"
        [server]
        api_token = "sekrit"

        [sites.alps]
        manta_server_url = "https://alps.example.ch:8443"
        token_file = "/run/secrets/alps"

        [sites.daint]
        manta_server_url = "https://daint.example.ch:8443"
        token = "inline-secret"

        [sites.prealps]
        manta_server_url = "https://prealps.example.ch:8443"
        token = "inline-secret"
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
        refreshed_at: None,
        last_error: Some("connection refused".to_owned()),
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
    for (label, entry) in dump["groups"].as_object().unwrap() {
      let (status, looked_up) = send(
        router.clone(),
        "GET",
        &format!("/api/v1/lookup/group/{label}"),
        Some("sekrit"),
      )
      .await;
      assert_eq!(status, StatusCode::OK, "{label}");
      assert_eq!(looked_up["site"], entry["site"], "{label}");
    }

    assert_eq!(dump["groups"]["compute"]["site"], "alps");
    assert_eq!(
      dump["groups"]["compute"]["members"],
      serde_json::json!(["x1000c0s0b0n0", "x1000c0s0b0n1"])
    );
    // A known-but-empty group is present with an empty member list, not
    // absent — same as `group_members` returning `Some(&[])`.
    assert_eq!(dump["groups"]["empty"]["members"], serde_json::json!([]));
    assert_eq!(dump["xnames"]["x2000c0s0b0n0"], "daint");
    assert!(dump["generated_at"].as_str().unwrap().ends_with('Z'));
    // Nothing collides in this fixture.
    assert_eq!(dump["conflicts"]["groups"], serde_json::json!([]));
    assert_eq!(dump["conflicts"]["xnames"], serde_json::json!([]));
  }

  #[tokio::test]
  async fn dump_lists_configured_sites_with_freshness_and_no_secrets() {
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
      sites["alps"]["manta_server_url"],
      "https://alps.example.ch:8443"
    );

    // Token provenance, never the token itself.
    assert_eq!(sites["alps"]["token_source"]["file"], "/run/secrets/alps");
    assert_eq!(sites["daint"]["token_source"], "inline");
    assert!(!dump.to_string().contains("inline-secret"));

    // A configured site that never refreshed is present and explains
    // itself — the whole reason `sites` is keyed by configured site.
    assert_eq!(sites["prealps"]["in_index"], false);
    assert_eq!(sites["prealps"]["refreshed_at"], serde_json::Value::Null);
    assert_eq!(sites["prealps"]["last_error"], "connection refused");
  }

  #[tokio::test]
  async fn dump_reports_cross_site_collisions() {
    // `nodes_free` is a conventional pool name, so the same label really
    // does show up at several sites. The index keeps only one owner and
    // silently drops the other site's members; `conflicts` says so.
    let snapshots = vec![
      SiteSnapshot {
        site: "alps".to_owned(),
        labels: vec!["nodes_free".to_owned()],
        nodes: vec![
          NodeMembership {
            xname: "x1000c0s0b0n0".to_owned(),
            groups: vec!["nodes_free".to_owned()],
          },
          NodeMembership {
            xname: "x9000c0s0b0n0".to_owned(),
            groups: vec!["nodes_free".to_owned()],
          },
        ],
      },
      SiteSnapshot {
        site: "daint".to_owned(),
        labels: vec!["nodes_free".to_owned()],
        nodes: vec![
          NodeMembership {
            xname: "x2000c0s0b0n0".to_owned(),
            groups: vec!["nodes_free".to_owned()],
          },
          // Also reported by alps above.
          NodeMembership {
            xname: "x9000c0s0b0n0".to_owned(),
            groups: vec!["nodes_free".to_owned()],
          },
        ],
      },
    ];
    let router =
      build_router(state_from(snapshots_to_state(snapshots), dump_config()));
    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;

    // The mirror shows only the winner, with no hint of alps' nodes.
    assert_eq!(dump["groups"]["nodes_free"]["site"], "daint");
    assert_eq!(
      dump["groups"]["nodes_free"]["members"],
      serde_json::json!(["x2000c0s0b0n0", "x9000c0s0b0n0"])
    );

    let conflict = &dump["conflicts"]["groups"][0];
    assert_eq!(conflict["label"], "nodes_free");
    assert_eq!(conflict["claimed_by"], serde_json::json!(["alps", "daint"]));
    assert_eq!(conflict["listed_by"], serde_json::json!(["alps", "daint"]));
    assert_eq!(conflict["owner"], "daint");
    // Only x1000c0s0b0n0 actually went missing: x9000c0s0b0n0 is claimed
    // by alps too but the winner contributes it anyway, so it is still
    // in the served member list and is not a loss.
    assert_eq!(
      conflict["discarded_members"],
      serde_json::json!(["x1000c0s0b0n0"])
    );
    // Full string, not a substring: two of the three arms mention
    // "site-name order", so a `contains` check passes on the wrong one.
    assert_eq!(
      conflict["reason"],
      "several sites listed this label; the last of listed_by in \
       site-name order owns it"
    );

    let xname_conflict = &dump["conflicts"]["xnames"][0];
    assert_eq!(xname_conflict["xname"], "x9000c0s0b0n0");
    assert_eq!(
      xname_conflict["reported_by"],
      serde_json::json!(["alps", "daint"])
    );
    assert_eq!(xname_conflict["owner"], "daint");
    assert_eq!(
      xname_conflict["reason"],
      "reported by several sites; the last in site-name order owns it"
    );
  }

  #[tokio::test]
  async fn dump_distinguishes_a_listing_from_a_bare_node_mention() {
    // The subtlest Index rule: a listing outranks a bare hsm mention
    // *whatever the fold order*. One label is not enough to pin that —
    // it takes both directions, since each rules out a different naive
    // implementation:
    //
    //   `shared` — the listing site (alps) sorts FIRST and still wins,
    //              which a plain last-write-wins would get wrong.
    //   `pool`   — the listing site (santis) sorts LAST and takes the
    //              label off an earlier mention-owner, which a plain
    //              first-claimer-wins would get wrong.
    let snapshots = vec![
      SiteSnapshot {
        site: "alps".to_owned(),
        labels: vec!["shared".to_owned()],
        nodes: vec![
          NodeMembership {
            xname: "x1000c0s0b0n0".to_owned(),
            groups: vec!["shared".to_owned()],
          },
          // Mention only: alps never lists `pool`.
          NodeMembership {
            xname: "x1001c0s0b0n0".to_owned(),
            groups: vec!["pool".to_owned()],
          },
        ],
      },
      SiteSnapshot {
        site: "santis".to_owned(),
        labels: vec!["pool".to_owned()],
        nodes: vec![
          // Mention only: santis never lists `shared`.
          NodeMembership {
            xname: "x3000c0s0b0n0".to_owned(),
            groups: vec!["shared".to_owned()],
          },
          NodeMembership {
            xname: "x3001c0s0b0n0".to_owned(),
            groups: vec!["pool".to_owned()],
          },
        ],
      },
    ];
    let router =
      build_router(state_from(snapshots_to_state(snapshots), dump_config()));
    let (_, dump) = send(router, "GET", "/api/v1/dump", Some("sekrit")).await;

    assert_eq!(dump["groups"]["shared"]["site"], "alps");
    assert_eq!(dump["groups"]["pool"]["site"], "santis");

    // conflicts.groups is label-sorted: "pool" before "shared".
    let pool = &dump["conflicts"]["groups"][0];
    assert_eq!(pool["label"], "pool");
    assert_eq!(pool["owner"], "santis");
    assert_eq!(pool["claimed_by"], serde_json::json!(["alps", "santis"]));
    assert_eq!(pool["listed_by"], serde_json::json!(["santis"]));
    // alps' node lost the label when santis' listing cleared the list.
    assert_eq!(
      pool["discarded_members"],
      serde_json::json!(["x1001c0s0b0n0"])
    );

    let conflict = &dump["conflicts"]["groups"][1];
    assert_eq!(conflict["label"], "shared");
    assert_eq!(conflict["owner"], "alps");
    assert_eq!(
      conflict["claimed_by"],
      serde_json::json!(["alps", "santis"])
    );
    assert_eq!(conflict["listed_by"], serde_json::json!(["alps"]));
    assert_eq!(
      conflict["discarded_members"],
      serde_json::json!(["x3000c0s0b0n0"])
    );
    assert_eq!(
      conflict["reason"],
      "one site listed this label; a listing outranks another site's \
       node membership, whatever the order"
    );
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
    let router =
      build_router(state_from(CacheState::default(), Arc::new(config)));

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
    assert_eq!(dump["groups"]["compute"]["site"], "live");
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
