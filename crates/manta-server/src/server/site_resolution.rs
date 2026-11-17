//! Server-side site resolution through `manta-cache-server`.
//!
//! A request that carries no `X-Manta-Site` header has its site looked
//! up here from the groups and xnames it names (see
//! [`super::handlers::SiteResolvingCtx`]). **Every** group and xname
//! must resolve, and all must agree on one site. The cache is an
//! accelerator, but a wrong guess could aim a destructive command at
//! the wrong cluster, so every answer short of a single site fails the
//! request with a message that tells the caller to pass `--site`:
//!
//! - nothing to resolve from, or nodes the cache cannot resolve
//!   (hostlist bracket expressions, NIDs) → 400;
//! - group the cache does not resolve → 404;
//! - group at several sites, xname list that is split across sites,
//!   ambiguous or partly unresolved, or groups and nodes that resolve
//!   to different sites → 409, naming the sites;
//! - cache down, slow or garbled → 503;
//! - cache rejects this server's token → 500 (an operator problem, not
//!   the caller's).

use std::collections::{BTreeMap, BTreeSet};
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

/// What a request targets: every group and node it names. Built with
/// [`Target::new`] and the chained [`Target::group`] / [`Target::groups`]
/// / [`Target::nodes`]; blank values are ignored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Target {
  /// Group labels, de-duplicated.
  groups: BTreeSet<String>,
  /// Plain xnames, de-duplicated.
  xnames: BTreeSet<String>,
  /// Node expressions the cache cannot resolve (hostlist expressions,
  /// NIDs), as given.
  unresolvable: Vec<String>,
}

impl Target {
  /// An empty target — names nothing.
  pub fn new() -> Self {
    Self::default()
  }

  /// Add a group label; an absent or blank label adds nothing.
  #[must_use]
  pub fn group(self, label: Option<&str>) -> Self {
    self.groups(label)
  }

  /// Add several group labels; blank ones are skipped.
  #[must_use]
  pub fn groups<I, S>(mut self, labels: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    self.groups.extend(
      labels
        .into_iter()
        .map(|l| l.as_ref().trim().to_owned())
        .filter(|l| !l.is_empty()),
    );
    self
  }

  /// Add node expressions, each a comma-separated list. An expression
  /// of plain xnames is looked up; any other (hostlist brackets, NIDs)
  /// makes the target unresolvable, since the cache indexes xnames
  /// only.
  #[must_use]
  pub fn nodes<S: AsRef<str>>(mut self, expressions: &[S]) -> Self {
    for expression in expressions.iter().map(|e| e.as_ref().trim()) {
      let tokens: Vec<&str> = expression
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
      if tokens.iter().all(|t| looks_like_xname(t)) {
        self.xnames.extend(tokens.into_iter().map(str::to_owned));
      } else {
        self.unresolvable.push(expression.to_owned());
      }
    }
    self
  }

  /// Does it name nothing at all?
  pub fn is_empty(&self) -> bool {
    self.groups.is_empty()
      && self.xnames.is_empty()
      && self.unresolvable.is_empty()
  }

  /// One-line description for log lines and audit details.
  pub fn describe(&self) -> String {
    let mut parts: Vec<String> = self
      .groups
      .iter()
      .map(|label| format!("group '{label}'"))
      .collect();
    let nodes: Vec<&str> = self
      .xnames
      .iter()
      .map(String::as_str)
      .chain(self.unresolvable.iter().map(String::as_str))
      .collect();
    if !nodes.is_empty() {
      parts.push(format!("nodes {}", nodes.join(",")));
    }
    parts.join(", ")
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
  /// No site header and the request names no group or node.
  NoTarget,
  /// No site header and the request names nodes the cache cannot
  /// resolve (hostlist expressions, NIDs).
  Unresolvable(Vec<String>),
  /// No site header and this server has no `[site_resolution]`.
  NotConfigured,
  /// The cache does not resolve the group (it may not exist, or the
  /// cache's service account may not see it).
  UnknownGroup(String),
  /// The group exists at several sites.
  AmbiguousGroup {
    /// The group label.
    label: String,
    /// Every site that has it, sorted.
    sites: Vec<String>,
  },
  /// The xnames do not resolve to one site; carries the per-xname
  /// resolutions, the ambiguous xnames and the unresolved ones.
  NotSingleSite {
    /// Xnames known at one site, and that site.
    resolutions: BTreeMap<String, String>,
    /// Xnames known at several sites, and every site.
    ambiguous: BTreeMap<String, Vec<String>>,
    /// Xnames the cache does not resolve.
    unknown: Vec<String>,
  },
  /// Each part of the request resolved, but not to the same site;
  /// carries what resolved where.
  SitesDisagree(BTreeMap<String, String>),
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
         or xname to resolve the site from. Pass --site <name>."
      ),
      Self::Unresolvable(tokens) => write!(
        f,
        "Missing X-Manta-Site header, and the site cannot be resolved \
         from {} (only group names and plain xnames are looked up, not \
         hostlist expressions or NIDs). Pass --site <name>.",
        tokens.join(", ")
      ),
      Self::NotConfigured => write!(
        f,
        "Missing X-Manta-Site header, and site resolution is not \
         configured on this server. Pass --site <name>."
      ),
      Self::UnknownGroup(label) => write!(
        f,
        "Group '{label}' could not be resolved to a site. Pass \
         --site <name>."
      ),
      Self::AmbiguousGroup { label, sites } => write!(
        f,
        "Group '{label}' exists at several sites ({}). Pass \
         --site <name>.",
        sites.join(", ")
      ),
      Self::NotSingleSite {
        resolutions,
        ambiguous,
        unknown,
      } => {
        let mut detail: Vec<String> = resolutions
          .iter()
          .map(|(xname, site)| format!("{xname} → {site}"))
          .chain(
            ambiguous
              .iter()
              .map(|(xname, sites)| format!("{xname} → {}", sites.join(" | "))),
          )
          .collect();
        if !unknown.is_empty() {
          detail.push(format!("not resolved: {}", unknown.join(", ")));
        }
        write!(
          f,
          "The xnames do not resolve to a single site ({}). Split the \
           command per site or pass --site <name> explicitly.",
          detail.join("; ")
        )
      }
      Self::SitesDisagree(found) => {
        let detail: Vec<String> = found
          .iter()
          .map(|(what, site)| format!("{what} → {site}"))
          .collect();
        write!(
          f,
          "The request's groups and nodes resolve to different sites \
           ({}). Pass --site <name>.",
          detail.join("; ")
        )
      }
      Self::Rejected(status) => write!(
        f,
        "The site-resolution cache rejected manta-server's token (HTTP \
         {status}); check [site_resolution] api_token (or api_token_file) \
         against the cache's [server] api_token. Pass --site <name> \
         meanwhile."
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
      Self::NoTarget | Self::Unresolvable(_) | Self::NotConfigured => {
        StatusCode::BAD_REQUEST
      }
      Self::UnknownGroup(_) => StatusCode::NOT_FOUND,
      Self::AmbiguousGroup { .. }
      | Self::NotSingleSite { .. }
      | Self::SitesDisagree(_) => StatusCode::CONFLICT,
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

/// `409` reply body of `GET /lookup/group/{label}`: the label exists
/// at several sites.
#[derive(Deserialize)]
struct AmbiguousGroupReply {
  sites: Vec<String>,
}

/// `GET /lookup/nodes` reply body.
#[derive(Deserialize)]
struct NodesReply {
  site: Option<String>,
  #[serde(default)]
  resolutions: BTreeMap<String, String>,
  #[serde(default)]
  ambiguous: BTreeMap<String, Vec<String>>,
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
  /// Build the resolver: normalise the URL, take the token (inline or
  /// from its file) and read the optional CA certificate.
  ///
  /// # Errors
  ///
  /// A human-readable message naming the offending
  /// `[site_resolution]` key.
  pub fn new(config: &SiteResolutionConfig) -> Result<Self, String> {
    let base = normalize_base(&config.url).map_err(|e| {
      format!("[site_resolution] url '{}' is not a URL: {e}", config.url)
    })?;
    if config.api_token.is_some() && config.api_token_file.is_some() {
      return Err(
        "[site_resolution] sets both `api_token` and `api_token_file`; \
         set at most one."
          .to_owned(),
      );
    }
    let inline = config
      .api_token
      .as_deref()
      .map(|token| {
        let token = token.trim();
        if token.is_empty() {
          return Err("[site_resolution] api_token is empty".to_owned());
        }
        Ok(token.to_owned())
      })
      .transpose()?;
    let from_file = config
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
    let api_token = inline.or(from_file);
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

  /// Ask the cache for the one site of `target`: every group and the
  /// xname list are looked up, and all must name the same site.
  ///
  /// # Errors
  ///
  /// Every [`ResolveError`] variant except `NotConfigured`, which the
  /// caller decides before getting here.
  pub async fn resolve(&self, target: &Target) -> Result<String, ResolveError> {
    if !target.unresolvable.is_empty() {
      return Err(ResolveError::Unresolvable(target.unresolvable.clone()));
    }
    if target.is_empty() {
      return Err(ResolveError::NoTarget);
    }
    // What resolved where, keyed by a description of the part.
    let mut found = BTreeMap::new();
    for label in &target.groups {
      let site = self.lookup_group(label).await?;
      found.insert(format!("group '{label}'"), site);
    }
    if !target.xnames.is_empty() {
      let xnames: Vec<&str> =
        target.xnames.iter().map(String::as_str).collect();
      let site = self.lookup_nodes(&xnames).await?;
      found.insert(format!("nodes {}", xnames.join(",")), site);
    }
    let sites: BTreeSet<&String> = found.values().collect();
    let site = match sites.into_iter().collect::<Vec<_>>().as_slice() {
      [site] => (*site).clone(),
      _ => return Err(ResolveError::SitesDisagree(found)),
    };
    tracing::info!(site, target = %target.describe(), "site resolved via manta-cache");
    Ok(site)
  }

  /// `GET /lookup/group/{label}`.
  async fn lookup_group(&self, label: &str) -> Result<String, ResolveError> {
    let mut url = self.base.clone();
    url
      .path_segments_mut()
      .expect("base URL normalised to http(s), so it has a path")
      .extend(["lookup", "group", label]);
    let (status, body) = self.get(url).await?;
    interpret_group_reply(status, &body, label)
  }

  /// `GET /lookup/nodes?xnames=…`.
  async fn lookup_nodes(
    &self,
    xnames: &[&str],
  ) -> Result<String, ResolveError> {
    let mut url = self.base.clone();
    url
      .path_segments_mut()
      .expect("base URL normalised to http(s), so it has a path")
      .extend(["lookup", "nodes"]);
    url
      .query_pairs_mut()
      .append_pair("xnames", &xnames.join(","));
    let (status, body) = self.get(url).await?;
    interpret_nodes_reply(status, &body)
  }

  /// Send one lookup; the status and body, or `Unavailable`.
  async fn get(
    &self,
    url: reqwest::Url,
  ) -> Result<(u16, String), ResolveError> {
    let mut request = self.client.get(url);
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
    Ok((status, body))
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
    409 => serde_json::from_str::<AmbiguousGroupReply>(body)
      .map_err(|e| undecodable(&e))
      .and_then(|reply| {
        Err(ResolveError::AmbiguousGroup {
          label: label.to_owned(),
          sites: reply.sites,
        })
      }),
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
        ambiguous: reply.ambiguous,
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
  fn target_collects_groups_and_xnames_and_skips_blanks() {
    let target = Target::new()
      .group(Some("zinal"))
      .group(Some("  "))
      .group(None)
      .groups(["zinal", "", "fora"])
      .nodes(&["x1000c0s0b0n0, x1000c0s0b0n1", ""])
      .nodes::<&str>(&[]);
    assert_eq!(target.groups.iter().collect::<Vec<_>>(), ["fora", "zinal"]);
    assert_eq!(
      target.xnames.iter().collect::<Vec<_>>(),
      ["x1000c0s0b0n0", "x1000c0s0b0n1"]
    );
    assert!(target.unresolvable.is_empty());
    assert_eq!(
      target.describe(),
      "group 'fora', group 'zinal', nodes x1000c0s0b0n0,x1000c0s0b0n1"
    );
    assert!(Target::new().group(Some(" ")).nodes(&[""]).is_empty());
  }

  #[test]
  fn hostlists_and_nids_make_the_target_unresolvable() {
    // Hostlist expressions need a site to expand; NIDs are not indexed.
    // The whole expression is kept, so the error shows what was typed.
    let target = Target::new().nodes(&[
      "x1000c0s[0,1]b0n0",
      "nid001234",
      "x1000c0s0b0n0,nid000001",
    ]);
    assert_eq!(
      target.unresolvable,
      ["x1000c0s[0,1]b0n0", "nid001234", "x1000c0s0b0n0,nid000001"]
    );
    assert!(target.xnames.is_empty());
  }

  #[test]
  fn group_reply_resolves_or_names_the_failure() {
    assert_eq!(
      interpret_group_reply(200, r#"{"site":"alps"}"#, "g").unwrap(),
      "alps"
    );
    let err = interpret_group_reply(404, "{}", "g").unwrap_err();
    assert_eq!(err.status(), StatusCode::NOT_FOUND);
    assert!(
      err.to_string().contains("'g' could not be resolved"),
      "{err}"
    );

    let err = interpret_group_reply(
      409,
      r#"{"error":"…","sites":["alps","daint"]}"#,
      "g",
    )
    .unwrap_err();
    assert_eq!(err.status(), StatusCode::CONFLICT);
    assert!(
      err.to_string().contains("several sites (alps, daint)"),
      "{err}"
    );

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
      "resolutions":{"x1":"alps","x2":"daint"},
      "ambiguous":{"x3":["alps","daint"]},"unknown":["x9"]}"#;
    let err = interpret_nodes_reply(200, body).unwrap_err();
    assert_eq!(err.status(), StatusCode::CONFLICT);
    let msg = err.to_string();
    assert!(msg.contains("x1 → alps"), "{msg}");
    assert!(msg.contains("x2 → daint"), "{msg}");
    assert!(msg.contains("x3 → alps | daint"), "{msg}");
    assert!(msg.contains("not resolved: x9"), "{msg}");

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
      api_token: None,
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
      .resolve(&Target::new().group(Some("a/b")))
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
    let target = Target::new().nodes(&["x1000c0s0b0n0,x1000c0s0b0n1"]);
    assert_eq!(
      resolver_for(&server).resolve(&target).await.unwrap(),
      "daint"
    );
  }

  /// Mock cache: `zinal` and `fora` at alps, `pilatus` at daint, and
  /// any xname list at alps.
  async fn two_site_cache() -> wiremock::MockServer {
    use wiremock::matchers::{method, path};
    let server = wiremock::MockServer::start().await;
    for (label, site) in
      [("zinal", "alps"), ("fora", "alps"), ("pilatus", "daint")]
    {
      wiremock::Mock::given(method("GET"))
        .and(path(format!("/api/v1/lookup/group/{label}")))
        .respond_with(
          wiremock::ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({ "site": site })),
        )
        .mount(&server)
        .await;
    }
    wiremock::Mock::given(method("GET"))
      .and(path("/api/v1/lookup/nodes"))
      .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(
        serde_json::json!({ "site": "alps", "resolutions": {}, "unknown": [] }),
      ))
      .mount(&server)
      .await;
    server
  }

  #[tokio::test]
  async fn resolve_accepts_groups_and_nodes_that_agree() {
    let server = two_site_cache().await;
    let target = Target::new()
      .groups(["zinal", "fora"])
      .nodes(&["x1000c0s0b0n0"]);
    assert_eq!(
      resolver_for(&server).resolve(&target).await.unwrap(),
      "alps"
    );
  }

  #[tokio::test]
  async fn resolve_rejects_parts_that_resolve_to_different_sites() {
    let server = two_site_cache().await;
    let target = Target::new()
      .groups(["zinal", "pilatus"])
      .nodes(&["x1000c0s0b0n0"]);
    let err = resolver_for(&server).resolve(&target).await.unwrap_err();
    assert_eq!(err.status(), StatusCode::CONFLICT);
    let msg = err.to_string();
    assert!(msg.contains("group 'pilatus' → daint"), "{msg}");
    assert!(msg.contains("group 'zinal' → alps"), "{msg}");
    assert!(msg.contains("nodes x1000c0s0b0n0 → alps"), "{msg}");
  }

  #[tokio::test]
  async fn resolve_refuses_unresolvable_nodes_without_asking_the_cache() {
    // No mocks mounted: any request would 404 and fail differently.
    let server = wiremock::MockServer::start().await;
    let target = Target::new().group(Some("zinal")).nodes(&["nid001289"]);
    let err = resolver_for(&server).resolve(&target).await.unwrap_err();
    assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    assert!(err.to_string().contains("nid001289"), "{err}");
    assert!(server.received_requests().await.unwrap().is_empty());
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
      api_token: None,
      api_token_file: Some(token_path.display().to_string()),
      root_ca_cert_file: None,
    })
    .unwrap();
    let site = resolver
      .resolve(&Target::new().group(Some("g")))
      .await
      .unwrap();
    assert_eq!(site, "alps");
  }

  #[test]
  fn new_rejects_a_missing_or_empty_token_file() {
    let config = |path: String| SiteResolutionConfig {
      url: "http://localhost:8081".to_owned(),
      api_token: None,
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
  async fn resolve_sends_the_inline_token_as_bearer() {
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
    let resolver = SiteResolver::new(&SiteResolutionConfig {
      url: server.uri(),
      api_token: Some("s3cret".to_owned()),
      api_token_file: None,
      root_ca_cert_file: None,
    })
    .unwrap();
    let site = resolver
      .resolve(&Target::new().group(Some("g")))
      .await
      .unwrap();
    assert_eq!(site, "alps");
  }

  #[test]
  fn new_rejects_both_token_forms_and_an_empty_inline_token() {
    let config =
      |api_token: &str, api_token_file: Option<&str>| SiteResolutionConfig {
        url: "http://localhost:8081".to_owned(),
        api_token: Some(api_token.to_owned()),
        api_token_file: api_token_file.map(str::to_owned),
        root_ca_cert_file: None,
      };
    let err =
      SiteResolver::new(&config("s3cret", Some("/run/secrets/t"))).unwrap_err();
    assert!(err.contains("at most one"), "{err}");
    let err = SiteResolver::new(&config("  ", None)).unwrap_err();
    assert!(err.contains("api_token is empty"), "{err}");
  }

  #[tokio::test]
  async fn resolve_reports_an_unreachable_cache_as_unavailable() {
    let resolver = SiteResolver::new(&SiteResolutionConfig {
      // Port 9 (discard) on loopback: nothing listens there.
      url: "http://127.0.0.1:9".to_owned(),
      api_token: None,
      api_token_file: None,
      root_ca_cert_file: None,
    })
    .unwrap();
    let err = resolver
      .resolve(&Target::new().group(Some("g")))
      .await
      .unwrap_err();
    assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);
  }
}
