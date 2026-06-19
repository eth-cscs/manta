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

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::CacheError;
use crate::index::{Index, NodeMembership, SiteSnapshot};
use crate::site::SiteDescriptor;

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

  // One pooled client, reused across every site request.
  let client = reqwest::Client::builder()
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
    return Err(CacheError::Status {
      site: site.name.clone(),
      status: status.as_u16(),
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
  fn parse_hsm_splits_trims_and_drops_empties() {
    assert_eq!(
      parse_hsm("compute, gpu ,, "),
      vec!["compute".to_owned(), "gpu".to_owned()]
    );
    assert!(parse_hsm("").is_empty());
  }
}
