//! Live population of the [`Index`] from manta-server group endpoints.
//!
//! One refresh is **two HTTP calls per site**, fanned out concurrently:
//!
//! - `GET /api/v1/groups/available` → the accessible group labels.
//! - `GET /api/v1/groups/nodes` (unfiltered) → every accessible node,
//!   each carrying its xname and comma-separated `hsm` membership.
//!
//! Both calls send `X-Manta-Site: <name>` and `Authorization: Bearer
//! <token>` from the [`SiteDescriptor`].

use std::time::Duration;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::CacheError;
use crate::index::{Index, NodeMembership, SiteSnapshot};
use crate::site::SiteDescriptor;

/// TCP/TLS connect timeout per site request. A site that is down or
/// unroutable should fail the refresh quickly, not stall it.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Whole-request timeout per site request. Matches the CLI's
/// `DEFAULT_API_TIMEOUT_SECS` (and manta-server's default global
/// request timeout), so a refresh gives up at roughly the same point
/// the server would have anyway.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Longest error-body snippet carried in [`CacheError::Status`].
const ERROR_BODY_SNIPPET_CHARS: usize = 256;

/// Subset of `GET /api/v1/groups/nodes`'s `NodeDetails` we consume.
///
/// Deliberately a minimal mirror, not a re-export of
/// `manta_shared::types::dto::NodeDetails`: depending on `manta-shared`
/// would drag `manta-backend-dispatcher` (+ config/utoipa/...) into this
/// otherwise standalone crate for the sake of two fields. serde ignores
/// the other ~10 fields of the response. See the crate README/ROADMAP.
#[derive(Debug, Deserialize)]
struct GroupNode {
  /// Physical location ID, e.g. `x3000c0s1b0n0`.
  xname: String,
  /// Comma-separated HSM group names this node belongs to.
  hsm: String,
}

/// Refresh the index by querying every site concurrently.
///
/// Fans out with [`futures::future::try_join_all`], so the call is
/// all-or-nothing: the first site that errors aborts the refresh and
/// returns its [`CacheError`]. (Partial-failure tolerance is a Stage-4
/// refinement — see ROADMAP.)
///
/// # Errors
///
/// Returns [`CacheError::ClientBuild`] if the shared HTTP client cannot
/// be created, or [`CacheError::Request`] / [`CacheError::Status`] for
/// the first site whose call fails.
pub async fn refresh(sites: &[SiteDescriptor]) -> Result<Index, CacheError> {
  tracing::debug!(site_count = sites.len(), "cache refresh starting");

  // One pooled client, reused across every site request. Both timeouts
  // are load-bearing: `try_join_all` makes the refresh as slow as its
  // slowest site, so a hung site must not stall it indefinitely.
  let client = reqwest::Client::builder()
    .connect_timeout(CONNECT_TIMEOUT)
    .timeout(REQUEST_TIMEOUT)
    .build()
    .map_err(CacheError::ClientBuild)?;

  let snapshots = futures::future::try_join_all(
    sites.iter().map(|site| fetch_site(&client, site)),
  )
  .await?;

  let index = Index::build(snapshots);
  tracing::debug!(sites = index.sites().count(), "cache refresh complete");
  Ok(index)
}

/// Gather one site's snapshot from its two group endpoints.
async fn fetch_site(
  client: &reqwest::Client,
  site: &SiteDescriptor,
) -> Result<SiteSnapshot, CacheError> {
  let base = normalize_base(&site.manta_server_url);
  tracing::debug!(site = %site.name, %base, "refreshing site");

  let labels: Vec<String> =
    get_json(client, site, &format!("{base}/groups/available")).await?;

  let nodes: Vec<GroupNode> =
    get_json(client, site, &format!("{base}/groups/nodes")).await?;

  let nodes = nodes
    .into_iter()
    .map(|n| NodeMembership {
      xname: n.xname,
      groups: parse_hsm(&n.hsm),
    })
    .collect();

  Ok(SiteSnapshot {
    site: site.name.clone(),
    labels,
    nodes,
  })
}

/// GET `url` with the site's `X-Manta-Site` + bearer headers and decode
/// the JSON body, mapping failures to the right [`CacheError`] variant.
async fn get_json<T: DeserializeOwned>(
  client: &reqwest::Client,
  site: &SiteDescriptor,
  url: &str,
) -> Result<T, CacheError> {
  let resp = client
    .get(url)
    .header("X-Manta-Site", &site.name)
    .bearer_auth(&site.token)
    .send()
    .await
    .map_err(|source| CacheError::Request {
      site: site.name.clone(),
      source,
    })?;

  let status = resp.status();
  if !status.is_success() {
    // Carry a bounded snippet of the body: manta-server error bodies
    // are small JSON objects whose message is the diagnosis.
    let body = resp.text().await.unwrap_or_default();
    return Err(CacheError::Status {
      site: site.name.clone(),
      status: status.as_u16(),
      body: body_snippet(&body),
    });
  }

  resp
    .json::<T>()
    .await
    .map_err(|source| CacheError::Request {
      site: site.name.clone(),
      source,
    })
}

/// Bound an error body to [`ERROR_BODY_SNIPPET_CHARS`] characters for
/// inclusion in [`CacheError::Status`], substituting a marker when the
/// body is empty.
fn body_snippet(body: &str) -> String {
  let trimmed = body.trim();
  if trimmed.is_empty() {
    return "<empty body>".to_owned();
  }
  trimmed.chars().take(ERROR_BODY_SNIPPET_CHARS).collect()
}

/// Parse a node's comma-separated `hsm` field into trimmed,
/// non-empty group labels.
fn parse_hsm(hsm: &str) -> Vec<String> {
  hsm
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(str::to_owned)
    .collect()
}

/// Normalise a manta-server base URL into `<scheme>://host[:port]/api/v1`.
///
/// Mirrors the CLI's `MantaClient` (`http_client/client.rs`): a missing
/// scheme defaults to `http://` for localhost-dev convenience. Production
/// callers should pass a full `https://…` URL.
fn normalize_base(url: &str) -> String {
  let with_scheme = if url.starts_with("http://") || url.starts_with("https://")
  {
    url.to_owned()
  } else {
    format!("http://{url}")
  };
  format!("{}/api/v1", with_scheme.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalize_base_adds_default_scheme_and_prefix() {
    assert_eq!(
      normalize_base("localhost:8443"),
      "http://localhost:8443/api/v1"
    );
  }

  #[test]
  fn normalize_base_preserves_https_and_trims_slash() {
    assert_eq!(
      normalize_base("https://manta.example.ch:8443/"),
      "https://manta.example.ch:8443/api/v1"
    );
  }

  #[test]
  fn body_snippet_bounds_length_and_marks_empty() {
    assert_eq!(body_snippet("  \n"), "<empty body>");
    assert_eq!(body_snippet(r#"{"error":"boom"}"#), r#"{"error":"boom"}"#);
    let long = "e".repeat(4 * ERROR_BODY_SNIPPET_CHARS);
    assert_eq!(
      body_snippet(&long).chars().count(),
      ERROR_BODY_SNIPPET_CHARS
    );
  }

  #[test]
  fn parse_hsm_splits_trims_and_drops_empties() {
    assert_eq!(
      parse_hsm("compute, gpu ,, "),
      vec!["compute".to_owned(), "gpu".to_owned()]
    );
    assert!(parse_hsm("").is_empty());
  }
}
