//! Refresh plumbing: turn `[sites.*]` config into `SiteDescriptor`s
//! and run the full and per-site refreshes against the shared
//! [`AppState`], with a new service-account token for every request
//! ([`crate::auth`]).
//!
//! The state keeps the last-good [`SiteSnapshot`] per site so a
//! [`refresh_site`] can rebuild the index from one fresh snapshot plus
//! the stored siblings, without re-fetching every site.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use manta_cache::{Index, SiteDescriptor, SiteSnapshot};

use crate::config::SiteConfig;
use crate::routes::{AppState, SiteStatus};

/// The [`SiteDescriptor`] for one configured site.
fn descriptor_for(name: &str, site: &SiteConfig) -> SiteDescriptor {
  SiteDescriptor {
    name: name.to_owned(),
    manta_server_url: site.manta_server_url.clone(),
  }
}

/// Rebuild the index from a snapshot store. `BTreeMap` iteration is
/// name-sorted, so the Index's documented "snapshot order" collision
/// rules are deterministic across runs.
fn rebuild(snapshots: &BTreeMap<String, SiteSnapshot>) -> Index {
  Index::from_snapshots(snapshots.values().cloned().collect())
}

/// Fetch every configured site and **replace** the whole cache state
/// with the outcome — a site that fails drops out of the index (and
/// snapshot store) until a refresh reaches it again, with its error in
/// its status. Returns the per-site failure messages; a site whose
/// token could not be obtained fails like any other site.
///
/// # Errors
///
/// Errs only when the refresh cannot start at all: the HTTP client
/// cannot be built. The previous state then keeps serving.
pub async fn refresh_all(state: &AppState) -> Result<Vec<String>, String> {
  // Deterministic fetch order (HashMap iteration is not).
  let mut descriptors: Vec<SiteDescriptor> = state
    .config
    .sites
    .iter()
    .map(|(name, site)| descriptor_for(name, site))
    .collect();
  descriptors.sort_by(|a, b| a.name.cmp(&b.name));

  let outcome = manta_cache::fetch_snapshots(&descriptors, &state.tokens)
    .await
    .map_err(|e| format!("refresh could not start: {e}"))?;

  // A snapshot with no groups is a failure wearing a 200: see
  // `empty_snapshot_error`. Splitting them out here means they flow
  // through exactly the same status and reporting path as a site that
  // never answered.
  let (usable, empty): (Vec<SiteSnapshot>, Vec<SiteSnapshot>) = outcome
    .snapshots
    .into_iter()
    .partition(|snapshot| !is_empty(snapshot));

  let mut failures: Vec<String> =
    outcome.failures.iter().map(ToString::to_string).collect();
  failures.extend(empty.iter().map(empty_snapshot_error));

  // Stamped after the fan-out: the fetch can take up to the per-request
  // timeout (300s), and `refreshed_at` documents when the held snapshot
  // was *fetched*.
  let now = chrono::Utc::now();

  // A full refresh replaces the store wholesale, so a failed site keeps
  // no data and therefore no `refreshed_at` — unlike `refresh_site`,
  // which leaves the previous snapshot (and its timestamp) serving.
  let mut status: BTreeMap<String, SiteStatus> = usable
    .iter()
    .map(|s| {
      (
        s.site.clone(),
        SiteStatus {
          refreshed_at: Some(now),
          last_error: None,
        },
      )
    })
    .collect();
  for failure in &outcome.failures {
    // `ClientBuild` blames no site, but it errors the call above rather
    // than reaching the outcome, so `site()` is always `Some` here.
    if let Some(site) = failure.site() {
      status.insert(
        site.to_owned(),
        SiteStatus {
          refreshed_at: None,
          last_error: Some(failure.to_string()),
        },
      );
    }
  }
  for snapshot in &empty {
    let message = empty_snapshot_error(snapshot);
    tracing::warn!(site = %snapshot.site, "{message}");
    status.insert(
      snapshot.site.clone(),
      SiteStatus {
        refreshed_at: None,
        last_error: Some(message),
      },
    );
  }

  let snapshots: BTreeMap<String, SiteSnapshot> =
    usable.into_iter().map(|s| (s.site.clone(), s)).collect();

  let mut cache = state.cache.write().await;
  cache.index = rebuild(&snapshots);
  cache.snapshots = snapshots;
  cache.status = status;
  tracing::info!(
    sites = cache.index.sites().count(),
    failures = failures.len(),
    "cross-site refresh finished"
  );
  Ok(failures)
}

/// Does this snapshot carry no group labels?
///
/// The labels are the groups of `GET /groups`, which the site scopes
/// by the roles of the exchanged token — so an empty list is precisely
/// the shape a service account with no HSM roles at that site produces,
/// which is the condition worth catching. A snapshot without labels
/// has no nodes either: they come from the groups' member lists.
fn is_empty(snapshot: &SiteSnapshot) -> bool {
  snapshot.labels.is_empty()
}

/// Explain a snapshot that arrived empty.
///
/// A site answering `200 []` used to count as a successful refresh,
/// leaving `GET /dump` showing a healthy site — fresh timestamp, no
/// error — that resolved nothing at all. *Partial* visibility is
/// expected (it is just the service account's role set), but empty is
/// not: no one configures a site in order for it to contribute nothing.
/// It means the service account resolved to no HSM roles at that site:
/// the site's Keycloak maps it to none in the token exchange, or the
/// roles were removed.
fn empty_snapshot_error(snapshot: &SiteSnapshot) -> String {
  format!(
    "site '{}' returned no groups (labels=0 nodes={}): the service \
     account resolves to no HSM roles there — check which roles the \
     site's Keycloak gives it in the token exchange",
    snapshot.site,
    snapshot.nodes.len()
  )
}

/// Why a [`refresh_site`] call failed.
pub enum SiteRefreshError {
  /// The name does not match any `[sites.<name>]` entry.
  UnknownSite,
  /// The site is configured but its snapshot could not be fetched; the
  /// message names the cause. The **index and snapshots** are left
  /// untouched, so the site keeps serving what it had; the failure is
  /// recorded in its [`SiteStatus`].
  Fetch(String),
}

/// Re-fetch one site's snapshot and rebuild the index from it plus the
/// stored snapshots of every other site. On failure the previous state
/// (including this site's last-good snapshot, if any) keeps serving.
pub async fn refresh_site(
  state: &AppState,
  name: &str,
) -> Result<(), SiteRefreshError> {
  let site = state
    .config
    .sites
    .get(name)
    .ok_or(SiteRefreshError::UnknownSite)?;

  // Read before the fetch so the failure path can tell whether the
  // status it is about to stamp is still the one it observed.
  let previous_refreshed_at = refreshed_at_of(state, name).await;

  let snapshot = match fetch_one(state, name, site).await {
    Ok(snapshot) => snapshot,
    Err(message) => {
      // Record why, keeping the previous snapshot and its timestamp:
      // the site still serves, it is just no fresher than it was.
      record_failure(state, name, &message, previous_refreshed_at).await;
      return Err(SiteRefreshError::Fetch(message));
    }
  };

  let mut cache = state.cache.write().await;
  cache.snapshots.insert(name.to_owned(), snapshot);
  cache.index = rebuild(&cache.snapshots);
  cache.status.insert(
    name.to_owned(),
    SiteStatus {
      // Stamped after the fetch, for the reason given in `refresh_all`.
      refreshed_at: Some(chrono::Utc::now()),
      last_error: None,
    },
  );
  tracing::info!(site = name, "single-site refresh finished");
  Ok(())
}

/// Fetch exactly one site's snapshot, flattening every failure mode
/// (no token, fan-out failure, per-site error, empty outcome, empty
/// snapshot) into one message.
async fn fetch_one(
  state: &AppState,
  name: &str,
  site: &SiteConfig,
) -> Result<SiteSnapshot, String> {
  let descriptor = descriptor_for(name, site);
  let outcome = manta_cache::fetch_snapshots(
    std::slice::from_ref(&descriptor),
    &state.tokens,
  )
  .await
  .map_err(|e| e.to_string())?;
  if let Some(failure) = outcome.failures.first() {
    return Err(failure.to_string());
  }
  let snapshot = outcome
    .snapshots
    .into_iter()
    .next()
    .ok_or_else(|| "no snapshot returned".to_string())?;
  if is_empty(&snapshot) {
    return Err(empty_snapshot_error(&snapshot));
  }
  Ok(snapshot)
}

/// This site's current `refreshed_at`, for use as the guard value of a
/// later [`record_failure`].
async fn refreshed_at_of(
  state: &AppState,
  name: &str,
) -> Option<chrono::DateTime<chrono::Utc>> {
  let cache = state.cache.read().await;
  cache.status.get(name).and_then(|s| s.refreshed_at)
}

/// Note a failed refresh without disturbing the data that site is still
/// serving.
///
/// `expected` is the `refreshed_at` observed before the fetch started.
/// The fetch runs outside the lock, so a concurrent [`refresh_all`] may
/// have succeeded for this site in the meantime; stamping the error
/// anyway would flag a freshly-refreshed site as failing and send an
/// operator chasing an outage that is already over. A changed
/// `refreshed_at` means exactly that happened, so the error is dropped —
/// it describes a state that is no longer current.
///
/// The bias is deliberate: **drop a possibly-stale error rather than
/// risk a false alarm.** It is not free — when the failure is the more
/// recent truth (a concurrent refresh succeeded, *then* the site went
/// down and this fetch failed) the error is dropped too, and the dump
/// shows the site clean until the next tick. That is the acceptable
/// direction to be wrong in: the caller still gets the failure in its
/// own `502`, and a transient gap in a diagnostic field costs less than
/// a `last_error` that sends someone investigating a healthy site.
/// Comparing timestamps instead of equality would not help —
/// `refreshed_at` only ever moves forward on success, or to `None` on a
/// wholesale failure that records its own error.
async fn record_failure(
  state: &AppState,
  name: &str,
  message: &str,
  expected: Option<chrono::DateTime<chrono::Utc>>,
) {
  let mut cache = state.cache.write().await;
  let entry = cache.status.entry(name.to_owned()).or_default();
  if entry.refreshed_at != expected {
    tracing::debug!(
      site = name,
      "dropping stale refresh failure: a concurrent refresh succeeded"
    );
    return;
  }
  entry.last_error = Some(message.to_owned());
}

/// Spawn the periodic refresh loop: a full [`refresh_all`] every
/// `interval`. Failures keep the previous state serving until the next
/// tick.
pub fn spawn_periodic(state: Arc<AppState>, interval: Duration) {
  tokio::spawn(async move {
    let mut ticker = tokio::time::interval(interval);
    // The immediate first tick would duplicate the startup refresh.
    ticker.tick().await;
    loop {
      ticker.tick().await;
      if let Err(e) = refresh_all(&state).await {
        // Keep serving the previous index; try again next tick.
        tracing::error!("periodic refresh failed, keeping old index: {e}");
      }
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::CacheServerConfiguration;
  use crate::routes::CacheState;

  /// An `AppState` holding just the given per-site status.
  fn state_with(status: BTreeMap<String, SiteStatus>) -> AppState {
    let config: CacheServerConfiguration = toml::from_str(
      "[sites.alps]\nmanta_server_url = \"http://127.0.0.1:9\"\n",
    )
    .unwrap();
    AppState {
      cache: tokio::sync::RwLock::new(CacheState {
        status,
        ..Default::default()
      }),
      config: Arc::new(config),
      tokens: crate::auth::test_support::unused_tokens(),
    }
  }

  fn dated(at: chrono::DateTime<chrono::Utc>) -> BTreeMap<String, SiteStatus> {
    BTreeMap::from([(
      "alps".to_owned(),
      SiteStatus {
        refreshed_at: Some(at),
        ..Default::default()
      },
    )])
  }

  #[tokio::test]
  async fn record_failure_notes_the_error_and_keeps_the_timestamp() {
    let at = chrono::Utc::now();
    let state = state_with(dated(at));

    record_failure(&state, "alps", "boom", Some(at)).await;

    let cache = state.cache.read().await;
    assert_eq!(cache.status["alps"].last_error.as_deref(), Some("boom"));
    // The data fetched at `at` is still what serves, so its timestamp
    // must survive the failure.
    assert_eq!(cache.status["alps"].refreshed_at, Some(at));
  }

  #[tokio::test]
  async fn record_failure_records_for_a_site_that_never_refreshed() {
    // The operationally common case: the site has been down since
    // startup, so it holds no data and no timestamp, and the operator
    // retries POST /refresh/{site}. `None == None` must record — a
    // stricter guard would swallow this forever, leaving the dump with
    // no stated cause for a site that has never once worked.
    let state = state_with(BTreeMap::new());

    record_failure(&state, "alps", "boom", None).await;

    let cache = state.cache.read().await;
    assert_eq!(cache.status["alps"].last_error.as_deref(), Some("boom"));
    assert_eq!(cache.status["alps"].refreshed_at, None);
  }

  #[tokio::test]
  async fn record_failure_drops_an_error_a_concurrent_refresh_outran() {
    let at = chrono::Utc::now();
    let state = state_with(dated(at));

    // Guard value `None`: this failure belongs to a refresh that started
    // when alps held no data. By the time it failed, a concurrent
    // refresh_all had succeeded and stamped `at` — so the error
    // describes a state that no longer exists and must be dropped,
    // rather than flagging a healthy site as failing.
    record_failure(&state, "alps", "boom", None).await;

    let cache = state.cache.read().await;
    assert!(cache.status["alps"].last_error.is_none());
    assert_eq!(cache.status["alps"].refreshed_at, Some(at));
  }
}
