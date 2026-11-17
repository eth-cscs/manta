//! Live population of the [`Index`] from manta-server group endpoints.
//!
//! One refresh is **one HTTP call per site**, fanned out concurrently:
//! `GET /v2/groups` (unfiltered), which returns every group the token
//! can access at the site with its member xnames. The labels become
//! [`SiteSnapshot::labels`] and the member lists are inverted into
//! [`SiteSnapshot::nodes`], so the index holds exactly the groups the
//! caller can access — no group a node merely belongs to.
//!
//! The call sends `X-Manta-Site: <name>` from the [`SiteDescriptor`]
//! and `Authorization: Bearer <token>` with a new token from the
//! [`TokenSource`] (see there for why it is never reused).

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::CacheError;
use crate::index::{Index, NodeMembership, SiteSnapshot};
use crate::site::SiteDescriptor;
use crate::token::TokenSource;

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

/// Subset of a `GET /v2/groups` group object we consume.
///
/// Deliberately a minimal mirror, not a re-export of
/// `manta_backend_dispatcher::types::Group`: depending on it would drag
/// the backend dispatcher (+ utoipa/...) into this otherwise standalone
/// crate for the sake of two fields. serde ignores `description`,
/// `tags` and `exclusiveGroup`. `members` and its `ids` are optional
/// there, so both may be absent here (an empty group).
#[derive(Debug, Deserialize)]
struct GroupEntry {
  /// Group label, e.g. `compute`.
  label: String,
  #[serde(default)]
  members: Option<GroupMembers>,
}

#[derive(Debug, Deserialize)]
struct GroupMembers {
  #[serde(default)]
  ids: Option<Vec<String>>,
}

/// What a [`refresh`] produced: the index over every site that
/// answered, plus the errors of those that did not.
///
/// The split leaves the availability policy with the caller: one
/// unreachable site does not have to cost the mapping of every other
/// site (e.g. at `manta-server` startup), but a caller that wants
/// all-or-nothing can reject any outcome that is not
/// [`complete`](RefreshOutcome::is_complete).
#[derive(Debug)]
pub struct RefreshOutcome {
  /// Index built from the sites that refreshed successfully. Failed
  /// sites contribute nothing — their groups and xnames are absent and
  /// they do not appear in [`Index::sites`].
  pub index: Index,
  /// One [`CacheError`] per failed site, in input order (each variant
  /// names the site). Empty on full success.
  pub failures: Vec<CacheError>,
}

impl RefreshOutcome {
  /// `true` when every site refreshed successfully.
  pub fn is_complete(&self) -> bool {
    self.failures.is_empty()
  }
}

/// What a [`fetch_snapshots`] produced: the raw per-site snapshots
/// (for callers that keep their own snapshot store, e.g. to rebuild
/// the index after re-fetching a single site) plus per-site errors.
#[derive(Debug)]
pub struct SnapshotOutcome {
  /// One snapshot per site that answered, in input order.
  pub snapshots: Vec<SiteSnapshot>,
  /// One [`CacheError`] per failed site, in input order (each variant
  /// names the site). Empty on full success.
  pub failures: Vec<CacheError>,
}

/// Fetch every site's snapshot concurrently, without folding them into
/// an [`Index`].
///
/// This is the lower-level half of [`refresh`], for callers that keep
/// the snapshots — e.g. a service that re-fetches one site on demand
/// and rebuilds via [`Index::from_snapshots`] together with the other
/// sites' stored snapshots. Per-site failures are logged at `warn` and
/// collected in [`SnapshotOutcome::failures`].
///
/// # Errors
///
/// Returns [`CacheError::ClientBuild`] if the shared HTTP client cannot
/// be created — the only failure that precedes the fan-out. Per-site
/// failures ([`CacheError::Token`] / [`CacheError::Request`] /
/// [`CacheError::Status`]) never error the call; they are collected in
/// the outcome.
pub async fn fetch_snapshots(
  sites: &[SiteDescriptor],
  tokens: &impl TokenSource,
) -> Result<SnapshotOutcome, CacheError> {
  tracing::debug!(site_count = sites.len(), "snapshot fetch starting");

  // One pooled client, reused across every site request. Both timeouts
  // are load-bearing: the fan-out completes only when the slowest site
  // has answered, so a hung site must not stall it indefinitely.
  let client = reqwest::Client::builder()
    .connect_timeout(CONNECT_TIMEOUT)
    .timeout(REQUEST_TIMEOUT)
    .build()
    .map_err(CacheError::ClientBuild)?;

  let results = futures::future::join_all(
    sites.iter().map(|site| fetch_site(&client, tokens, site)),
  )
  .await;

  let mut snapshots = Vec::with_capacity(results.len());
  let mut failures = Vec::new();
  for result in results {
    match result {
      Ok(snapshot) => snapshots.push(snapshot),
      Err(error) => {
        tracing::warn!(%error, "site refresh failed");
        failures.push(error);
      }
    }
  }
  Ok(SnapshotOutcome {
    snapshots,
    failures,
  })
}

/// Refresh the index by querying every site concurrently.
///
/// The fan-out tolerates per-site failure: sites that answer contribute
/// their snapshot to [`RefreshOutcome::index`], sites that fail
/// contribute a [`CacheError`] to [`RefreshOutcome::failures`] (and are
/// logged at `warn`). The caller picks the policy — reject an
/// incomplete refresh via [`RefreshOutcome::is_complete`], or serve the
/// partial index and retry the failed sites later. To keep the raw
/// snapshots instead of the folded index, use [`fetch_snapshots`].
///
/// # Errors
///
/// Same as [`fetch_snapshots`], which this wraps.
pub async fn refresh(
  sites: &[SiteDescriptor],
  tokens: &impl TokenSource,
) -> Result<RefreshOutcome, CacheError> {
  let outcome = fetch_snapshots(sites, tokens).await?;
  let index = Index::from_snapshots(outcome.snapshots);
  tracing::debug!(
    sites = index.sites().count(),
    failures = outcome.failures.len(),
    "cache refresh complete"
  );
  Ok(RefreshOutcome {
    index,
    failures: outcome.failures,
  })
}

/// Gather one site's snapshot from `GET /groups`.
async fn fetch_site(
  client: &reqwest::Client,
  tokens: &impl TokenSource,
  site: &SiteDescriptor,
) -> Result<SiteSnapshot, CacheError> {
  let base = normalize_base(&site.manta_server_url);
  tracing::debug!(site = %site.name, %base, "refreshing site");

  let groups: Vec<GroupEntry> =
    get_json(client, tokens, site, &format!("{base}/groups")).await?;
  Ok(snapshot_from_groups(&site.name, groups))
}

/// Fold a `GET /groups` answer into a [`SiteSnapshot`]: the labels in
/// response order, and each member xname with the groups listing it
/// (in response order, a group listing it twice counted once).
fn snapshot_from_groups(site: &str, groups: Vec<GroupEntry>) -> SiteSnapshot {
  let mut labels = Vec::with_capacity(groups.len());
  let mut memberships: BTreeMap<String, Vec<String>> = BTreeMap::new();
  for group in groups {
    let ids = group.members.and_then(|m| m.ids).unwrap_or_default();
    for xname in ids {
      let listed = memberships.entry(xname).or_default();
      if !listed.contains(&group.label) {
        listed.push(group.label.clone());
      }
    }
    labels.push(group.label);
  }
  SiteSnapshot {
    site: site.to_owned(),
    labels,
    nodes: memberships
      .into_iter()
      .map(|(xname, groups)| NodeMembership { xname, groups })
      .collect(),
  }
}

/// GET `url` with the site's `X-Manta-Site` header and a new bearer
/// token, and decode the JSON body, mapping failures to the right
/// [`CacheError`] variant.
///
/// The only place a token is taken: every request gets its own.
async fn get_json<T: DeserializeOwned>(
  client: &reqwest::Client,
  tokens: &impl TokenSource,
  site: &SiteDescriptor,
  url: &str,
) -> Result<T, CacheError> {
  let token = tokens.token().await.map_err(|source| CacheError::Token {
    site: site.name.clone(),
    source,
  })?;
  let resp = client
    .get(url)
    .header("X-Manta-Site", &site.name)
    .bearer_auth(token)
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

/// Normalise a manta-server base URL into `<scheme>://host[:port]/v2`.
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
  format!("{}/v2", with_scheme.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalize_base_adds_default_scheme_and_prefix() {
    assert_eq!(normalize_base("localhost:8443"), "http://localhost:8443/v2");
  }

  #[test]
  fn normalize_base_preserves_https_and_trims_slash() {
    assert_eq!(
      normalize_base("https://manta.example.ch:8443/"),
      "https://manta.example.ch:8443/v2"
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

  fn groups(json: serde_json::Value) -> Vec<GroupEntry> {
    serde_json::from_value(json).unwrap()
  }

  #[test]
  fn groups_fold_into_labels_and_per_node_membership() {
    let snapshot = snapshot_from_groups(
      "alps",
      groups(serde_json::json!([
        {"label": "compute", "members": {"ids": ["x1", "x2"]}},
        {"label": "gpu", "description": "d", "members": {"ids": ["x2"]}},
      ])),
    );
    assert_eq!(snapshot.site, "alps");
    assert_eq!(snapshot.labels, vec!["compute", "gpu"]);
    let nodes: Vec<(&str, &[String])> = snapshot
      .nodes
      .iter()
      .map(|n| (n.xname.as_str(), n.groups.as_slice()))
      .collect();
    assert_eq!(
      nodes,
      vec![
        ("x1", &["compute".to_owned()][..]),
        ("x2", &["compute".to_owned(), "gpu".to_owned()][..]),
      ]
    );
  }

  #[test]
  fn an_empty_group_keeps_its_label_however_it_is_spelled() {
    // `members` and `ids` are both optional in the backend's type.
    let snapshot = snapshot_from_groups(
      "alps",
      groups(serde_json::json!([
        {"label": "no_members"},
        {"label": "no_ids", "members": {}},
        {"label": "empty_ids", "members": {"ids": []}},
      ])),
    );
    assert_eq!(snapshot.labels, vec!["no_members", "no_ids", "empty_ids"]);
    assert!(snapshot.nodes.is_empty());
  }

  #[test]
  fn a_member_listed_twice_in_one_group_counts_once() {
    let snapshot = snapshot_from_groups(
      "alps",
      groups(serde_json::json!([
        {"label": "compute", "members": {"ids": ["x1", "x1"]}},
      ])),
    );
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(snapshot.nodes[0].groups, vec!["compute".to_owned()]);
  }
}
