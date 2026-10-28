//! Server-side site resolution through `manta-cache-server`.
//!
//! A request that names a group or a plain xname list but carries no
//! `X-Manta-Site` header has its site looked up here (see
//! [`super::handlers::SiteResolvingCtx`]). The cache is an
//! accelerator, but a wrong guess could aim a destructive command at
//! the wrong cluster, so every answer short of a single named site
//! fails the request with a message that tells the caller to pass
//! `--site`:
//!
//! - unknown group / xname → 404;
//! - xname list split across sites → 409, naming the per-xname sites;
//! - cache down, slow or garbled → 503;
//! - cache rejects this server's token → 500 (an operator problem, not
//!   the caller's).
//!
//! Targets the cache cannot know — hostlist bracket expressions, NIDs
//! — are not looked up at all ([`Target::nodes`] returns `None`).

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::{
  Json,
  extract::Request,
  http::{HeaderValue, StatusCode},
  middleware::Next,
  response::Response,
};
use serde::Deserialize;

use super::handlers::ErrorResponse;
use crate::config::SiteResolutionConfig;

/// TCP connect timeout for a lookup. Short on purpose: a cache that
/// is down makes every site-less request pay this before failing.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Whole-request timeout for a lookup — lookups are O(1) reads, so
/// anything slow means the cache is unhealthy.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// What a request targets, as far as the cache can resolve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
  /// A single HSM group label.
  Group(String),
  /// A plain xname list (no hostlist brackets, no NIDs).
  Nodes(Vec<String>),
}

impl Target {
  /// A group target, or `None` for an absent/blank label.
  pub fn group(label: Option<&str>) -> Option<Self> {
    label
      .map(str::trim)
      .filter(|l| !l.is_empty())
      .map(|l| Self::Group(l.to_owned()))
  }

  /// Split a comma-separated node expression into xnames, or `None`
  /// when any token is not a plain xname (hostlist brackets, NIDs, …)
  /// — the cache indexes xnames only.
  pub fn nodes<S: AsRef<str>>(tokens: &[S]) -> Option<Self> {
    let xnames: Vec<String> = tokens
      .iter()
      .flat_map(|t| t.as_ref().split(','))
      .map(str::trim)
      .filter(|t| !t.is_empty())
      .map(str::to_owned)
      .collect();
    if xnames.is_empty() || !xnames.iter().all(|t| looks_like_xname(t)) {
      return None;
    }
    Some(Self::Nodes(xnames))
  }

  /// One-line description for log lines and audit details.
  pub fn describe(&self) -> String {
    match self {
      Self::Group(label) => format!("group '{label}'"),
      Self::Nodes(xnames) => format!("nodes {}", xnames.join(",")),
    }
  }
}

/// Cheap shape check: `x` followed by a digit, no hostlist brackets.
/// Deliberately loose — a wrong-but-xname-shaped value simply comes
/// back from the cache as unknown.
fn looks_like_xname(token: &str) -> bool {
  let mut chars = token.chars();
  chars.next() == Some('x')
    && chars.next().is_some_and(|c| c.is_ascii_digit())
    && !token.contains(['[', ']'])
}

/// Why a site could not be resolved. Each variant maps to one HTTP
/// status via [`ResolveError::into_handler_error`].
#[derive(Debug)]
pub enum ResolveError {
  /// No site header and nothing in the request the cache could
  /// resolve.
  NoTarget,
  /// No site header and this server has no `[site_resolution]`.
  NotConfigured,
  /// The cache knows no such group.
  UnknownGroup(String),
  /// The xnames do not resolve to one site; carries the per-xname
  /// resolutions and the unknown xnames.
  NotSingleSite {
    /// Known xnames and their sites.
    resolutions: BTreeMap<String, String>,
    /// Xnames the cache knows nothing about.
    unknown: Vec<String>,
  },
  /// The cache rejected this server's bearer token.
  Rejected(u16),
  /// The cache is unusable (unreachable, slow, unexpected reply).
  Unavailable(String),
}

impl std::fmt::Display for ResolveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NoTarget => write!(
        f,
        "Missing X-Manta-Site header, and this request names no group \
         or plain xname list to resolve the site from. Pass --site <name>."
      ),
      Self::NotConfigured => write!(
        f,
        "Missing X-Manta-Site header, and site resolution is not \
         configured on this server. Pass --site <name>."
      ),
      Self::UnknownGroup(label) => write!(
        f,
        "The site-resolution cache knows no group '{label}'. Pass \
         --site <name> explicitly (or refresh the cache if the group \
         is new)."
      ),
      Self::NotSingleSite {
        resolutions,
        unknown,
      } => {
        let mut detail: Vec<String> = resolutions
          .iter()
          .map(|(xname, site)| format!("{xname} → {site}"))
          .collect();
        if !unknown.is_empty() {
          detail.push(format!("unknown: {}", unknown.join(", ")));
        }
        write!(
          f,
          "The xnames do not resolve to a single site ({}). Split the \
           command per site or pass --site <name> explicitly.",
          detail.join("; ")
        )
      }
      Self::Rejected(status) => write!(
        f,
        "The site-resolution cache rejected manta-server's token (HTTP \
         {status}); check [site_resolution] api_token_file against the \
         cache's [server] api_token. Pass --site <name> meanwhile."
      ),
      Self::Unavailable(reason) => write!(
        f,
        "The site could not be resolved: {reason}. Pass --site <name>."
      ),
    }
  }
}

impl ResolveError {
  /// The HTTP status this failure is reported with.
  pub fn status(&self) -> StatusCode {
    match self {
      Self::NoTarget | Self::NotConfigured => StatusCode::BAD_REQUEST,
      Self::UnknownGroup(_) => StatusCode::NOT_FOUND,
      Self::NotSingleSite { .. } => StatusCode::CONFLICT,
      Self::Rejected(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
  }

  /// Render as the `(status, body)` pair every handler returns.
  pub fn into_handler_error(self) -> (StatusCode, Json<ErrorResponse>) {
    let status = self.status();
    if status.is_server_error() {
      tracing::warn!("site resolution failed: {self}");
    } else {
      tracing::debug!("site resolution failed: {self}");
    }
    (
      status,
      Json(ErrorResponse {
        error: self.to_string(),
      }),
    )
  }
}

/// `GET /lookup/group/{label}` reply body.
#[derive(Deserialize)]
struct GroupReply {
  site: String,
}

/// `GET /lookup/nodes` reply body.
#[derive(Deserialize)]
struct NodesReply {
  site: Option<String>,
  #[serde(default)]
  resolutions: BTreeMap<String, String>,
  #[serde(default)]
  unknown: Vec<String>,
}

/// Client for `manta-cache-server`'s lookup endpoints, built once at
/// startup from `[site_resolution]`.
#[derive(Debug)]
pub struct SiteResolver {
  client: reqwest::Client,
  /// `<scheme>://host[:port]/api/v1`.
  base: reqwest::Url,
  api_token: Option<String>,
}

impl SiteResolver {
  /// Build the resolver: normalise the URL, read the token file and
  /// the optional CA certificate.
  ///
  /// # Errors
  ///
  /// A human-readable message naming the offending
  /// `[site_resolution]` key.
  pub fn new(config: &SiteResolutionConfig) -> Result<Self, String> {
    let base = normalize_base(&config.url).map_err(|e| {
      format!("[site_resolution] url '{}' is not a URL: {e}", config.url)
    })?;
    let api_token = config
      .api_token_file
      .as_deref()
      .map(|path| {
        let token = std::fs::read_to_string(path).map_err(|e| {
          format!(
            "[site_resolution] api_token_file '{path}' could not be \
             read: {e}"
          )
        })?;
        let token = token.trim().to_owned();
        if token.is_empty() {
          return Err(format!(
            "[site_resolution] api_token_file '{path}' is empty"
          ));
        }
        Ok(token)
      })
      .transpose()?;
    let mut builder = reqwest::Client::builder()
      .connect_timeout(CONNECT_TIMEOUT)
      .timeout(REQUEST_TIMEOUT);
    if let Some(path) = &config.root_ca_cert_file {
      let pem = manta_shared::common::config::get_csm_root_cert_content(path)
        .map_err(|e| {
        format!(
          "[site_resolution] root_ca_cert_file '{path}' could not be \
             read: {e}"
        )
      })?;
      let cert = reqwest::Certificate::from_pem(&pem).map_err(|e| {
        format!(
          "[site_resolution] root_ca_cert_file '{path}' is not a PEM \
           certificate: {e}"
        )
      })?;
      builder = builder.add_root_certificate(cert);
    }
    let client = builder.build().map_err(|e| {
      format!("failed to build the site-resolution HTTP client: {e}")
    })?;
    Ok(Self {
      client,
      base,
      api_token,
    })
  }

  /// Ask the cache which single site owns `target`.
  ///
  /// # Errors
  ///
  /// Every [`ResolveError`] variant except `NoTarget` /
  /// `NotConfigured`, which the caller decides before getting here.
  pub async fn resolve(&self, target: &Target) -> Result<String, ResolveError> {
    let mut url = self.base.clone();
    {
      let mut segments = url
        .path_segments_mut()
        .expect("base URL normalised to http(s), so it has a path");
      match target {
        Target::Group(label) => {
          segments.extend(["lookup", "group", label]);
        }
        Target::Nodes(_) => {
          segments.extend(["lookup", "nodes"]);
        }
      }
    }
    if let Target::Nodes(xnames) = target {
      url
        .query_pairs_mut()
        .append_pair("xnames", &xnames.join(","));
    }

    let mut request = self.client.get(url.clone());
    if let Some(token) = &self.api_token {
      request = request.bearer_auth(token);
    }
    let response = request.send().await.map_err(|e| {
      ResolveError::Unavailable(format!("the cache is unreachable ({e})"))
    })?;
    let status = response.status().as_u16();
    let body = response.text().await.map_err(|e| {
      ResolveError::Unavailable(format!("the cache reply was cut off ({e})"))
    })?;

    let site = match target {
      Target::Group(label) => interpret_group_reply(status, &body, label)?,
      Target::Nodes(_) => interpret_nodes_reply(status, &body)?,
    };
    tracing::info!(site, target = %target.describe(), "site resolved via manta-cache");
    Ok(site)
  }
}

/// Per-request slot through which [`super::handlers::SiteResolvingCtx`]
/// hands the resolved site back to [`echo_resolved_site`]. Cheap to
/// clone; every clone shares the one slot.
#[derive(Clone, Default)]
pub struct ResolvedSite(Arc<OnceLock<String>>);

impl ResolvedSite {
  /// Record the resolved site. A second call is ignored — a request
  /// resolves at most once.
  pub fn set(&self, site: &str) {
    let _ = self.0.set(site.to_owned());
  }
}

/// Router middleware: give each request a [`ResolvedSite`] slot and,
/// when the handler filled it, return the site in the `X-Manta-Site`
/// response header. The CLI pins that site for the rest of the
/// command (e.g. the power-transition polling that follows
/// `POST /power`), and the audit layer records it. Requests that named
/// their site get no echo — the caller already knows it.
pub async fn echo_resolved_site(mut request: Request, next: Next) -> Response {
  let slot = ResolvedSite::default();
  request.extensions_mut().insert(slot.clone());
  let mut response = next.run(request).await;
  if let Some(value) = slot.0.get().and_then(|s| HeaderValue::from_str(s).ok())
  {
    response.headers_mut().insert("X-Manta-Site", value);
  }
  response
}

/// Map a `GET /lookup/group/{label}` reply.
fn interpret_group_reply(
  status: u16,
  body: &str,
  label: &str,
) -> Result<String, ResolveError> {
  match status {
    200 => serde_json::from_str::<GroupReply>(body)
      .map(|reply| reply.site)
      .map_err(|e| undecodable(&e)),
    404 => Err(ResolveError::UnknownGroup(label.to_owned())),
    401 | 403 => Err(ResolveError::Rejected(status)),
    other => Err(unexpected_status(other)),
  }
}

/// Map a `GET /lookup/nodes` reply. A split or partially-unknown list
/// is a definitive answer, not an outage.
fn interpret_nodes_reply(
  status: u16,
  body: &str,
) -> Result<String, ResolveError> {
  match status {
    200 => match serde_json::from_str::<NodesReply>(body) {
      Ok(NodesReply {
        site: Some(site), ..
      }) => Ok(site),
      Ok(reply) => Err(ResolveError::NotSingleSite {
        resolutions: reply.resolutions,
        unknown: reply.unknown,
      }),
      Err(e) => Err(undecodable(&e)),
    },
    401 | 403 => Err(ResolveError::Rejected(status)),
    other => Err(unexpected_status(other)),
  }
}

fn undecodable(e: &serde_json::Error) -> ResolveError {
  ResolveError::Unavailable(format!("the cache reply was undecodable ({e})"))
}

fn unexpected_status(status: u16) -> ResolveError {
  ResolveError::Unavailable(format!("the cache returned HTTP {status}"))
}

/// Normalise `url` into `<scheme>://host[:port]/api/v1`; a missing
/// scheme defaults to `http://`.
fn normalize_base(url: &str) -> Result<reqwest::Url, String> {
  let with_scheme = if url.starts_with("http://") || url.starts_with("https://")
  {
    url.to_owned()
  } else {
    format!("http://{url}")
  };
  reqwest::Url::parse(&format!("{}/api/v1", with_scheme.trim_end_matches('/')))
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn group_target_ignores_blank_labels() {
    assert_eq!(
      Target::group(Some("zinal")),
      Some(Target::Group("zinal".to_owned()))
    );
    assert_eq!(Target::group(Some("  ")), None);
    assert_eq!(Target::group(None), None);
  }

  #[test]
  fn nodes_target_accepts_plain_xnames_only() {
    assert_eq!(
      Target::nodes(&["x1000c0s0b0n0, x1000c0s0b0n1"]),
      Some(Target::Nodes(vec![
        "x1000c0s0b0n0".to_owned(),
        "x1000c0s0b0n1".to_owned()
      ]))
    );
    assert_eq!(
      Target::nodes(&["x1000c0s0b0n0".to_owned(), "x1000c0s0b0n1".to_owned()]),
      Some(Target::Nodes(vec![
        "x1000c0s0b0n0".to_owned(),
        "x1000c0s0b0n1".to_owned()
      ]))
    );
    // Hostlist expressions need a site to expand; NIDs are not indexed.
    assert_eq!(Target::nodes(&["x1000c0s[0-3]b0n0"]), None);
    assert_eq!(Target::nodes(&["nid001234"]), None);
    assert_eq!(Target::nodes(&["x1000c0s0b0n0,nid001234"]), None);
    assert_eq!(Target::nodes::<&str>(&[]), None);
    assert_eq!(Target::nodes(&[""]), None);
  }

  #[test]
  fn group_reply_resolves_or_names_the_failure() {
    assert_eq!(
      interpret_group_reply(200, r#"{"site":"alps"}"#, "g").unwrap(),
      "alps"
    );
    let err = interpret_group_reply(404, "{}", "g").unwrap_err();
    assert_eq!(err.status(), StatusCode::NOT_FOUND);
    assert!(err.to_string().contains("knows no group 'g'"), "{err}");

    let err = interpret_group_reply(401, "", "g").unwrap_err();
    assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(err.to_string().contains("api_token_file"), "{err}");

    let err = interpret_group_reply(500, "boom", "g").unwrap_err();
    assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(err.to_string().contains("HTTP 500"), "{err}");

    let err = interpret_group_reply(200, "not json", "g").unwrap_err();
    assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);
  }

  #[test]
  fn nodes_reply_split_is_a_conflict_naming_resolutions() {
    let body = r#"{"site":null,
      "resolutions":{"x1":"alps","x2":"daint"},"unknown":["x9"]}"#;
    let err = interpret_nodes_reply(200, body).unwrap_err();
    assert_eq!(err.status(), StatusCode::CONFLICT);
    let msg = err.to_string();
    assert!(msg.contains("x1 → alps"), "{msg}");
    assert!(msg.contains("x2 → daint"), "{msg}");
    assert!(msg.contains("unknown: x9"), "{msg}");

    assert_eq!(
      interpret_nodes_reply(
        200,
        r#"{"site":"alps","resolutions":{"x1":"alps"},"unknown":[]}"#,
      )
      .unwrap(),
      "alps"
    );
  }

  #[test]
  fn normalize_base_defaults_scheme_and_appends_prefix() {
    assert_eq!(
      normalize_base("localhost:18081").unwrap().as_str(),
      "http://localhost:18081/api/v1"
    );
    assert_eq!(
      normalize_base("https://cache.example.ch:8444/")
        .unwrap()
        .as_str(),
      "https://cache.example.ch:8444/api/v1"
    );
  }

  fn resolver_for(server: &wiremock::MockServer) -> SiteResolver {
    SiteResolver::new(&SiteResolutionConfig {
      url: server.uri(),
      api_token_file: None,
      root_ca_cert_file: None,
    })
    .unwrap()
  }

  #[tokio::test]
  async fn resolve_percent_encodes_the_group_label() {
    use wiremock::matchers::{method, path};
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(method("GET"))
      .and(path("/api/v1/lookup/group/a%2Fb"))
      .respond_with(
        wiremock::ResponseTemplate::new(200)
          .set_body_json(serde_json::json!({ "site": "alps" })),
      )
      .expect(1)
      .mount(&server)
      .await;
    let site = resolver_for(&server)
      .resolve(&Target::Group("a/b".to_owned()))
      .await
      .unwrap();
    assert_eq!(site, "alps");
  }

  #[tokio::test]
  async fn resolve_sends_the_xname_list_as_one_query_param() {
    use wiremock::matchers::{method, path, query_param};
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(method("GET"))
      .and(path("/api/v1/lookup/nodes"))
      .and(query_param("xnames", "x1000c0s0b0n0,x1000c0s0b0n1"))
      .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(
        serde_json::json!({ "site": "daint", "resolutions": {}, "unknown": [] }),
      ))
      .expect(1)
      .mount(&server)
      .await;
    let target = Target::nodes(&["x1000c0s0b0n0,x1000c0s0b0n1"]).unwrap();
    assert_eq!(
      resolver_for(&server).resolve(&target).await.unwrap(),
      "daint"
    );
  }

  #[tokio::test]
  async fn resolve_sends_the_token_file_as_bearer() {
    use wiremock::matchers::{header, method};
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(method("GET"))
      .and(header("Authorization", "Bearer s3cret"))
      .respond_with(
        wiremock::ResponseTemplate::new(200)
          .set_body_json(serde_json::json!({ "site": "alps" })),
      )
      .expect(1)
      .mount(&server)
      .await;
    let dir = tempfile::tempdir().unwrap();
    let token_path = dir.path().join("token");
    // Trailing newline, as `echo` writes it: trimmed on read.
    std::fs::write(&token_path, "s3cret\n").unwrap();
    let resolver = SiteResolver::new(&SiteResolutionConfig {
      url: server.uri(),
      api_token_file: Some(token_path.display().to_string()),
      root_ca_cert_file: None,
    })
    .unwrap();
    let site = resolver
      .resolve(&Target::Group("g".to_owned()))
      .await
      .unwrap();
    assert_eq!(site, "alps");
  }

  #[test]
  fn new_rejects_a_missing_or_empty_token_file() {
    let config = |path: String| SiteResolutionConfig {
      url: "http://localhost:8081".to_owned(),
      api_token_file: Some(path),
      root_ca_cert_file: None,
    };
    let err =
      SiteResolver::new(&config("/nonexistent/token".to_owned())).unwrap_err();
    assert!(err.contains("api_token_file"), "{err}");

    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("empty");
    std::fs::write(&empty, "\n").unwrap();
    let err =
      SiteResolver::new(&config(empty.display().to_string())).unwrap_err();
    assert!(err.contains("is empty"), "{err}");
  }

  #[tokio::test]
  async fn resolve_reports_an_unreachable_cache_as_unavailable() {
    let resolver = SiteResolver::new(&SiteResolutionConfig {
      // Port 9 (discard) on loopback: nothing listens there.
      url: "http://127.0.0.1:9".to_owned(),
      api_token_file: None,
      root_ca_cert_file: None,
    })
    .unwrap();
    let err = resolver
      .resolve(&Target::Group("g".to_owned()))
      .await
      .unwrap_err();
    assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);
  }
}
