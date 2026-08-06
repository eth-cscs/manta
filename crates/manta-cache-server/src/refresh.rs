//! Refresh plumbing: turn `[sites.*]` config into `SiteDescriptor`s
//! (resolving `token` / `token_file`) and run the full and per-site
//! refreshes against the shared [`AppState`].
//!
//! `token_file` is re-read on **every** refresh, so a secret manager
//! (Vault Agent, kubelet-projected secret) can rotate the credential
//! without restarting the service.
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

/// Build the [`SiteDescriptor`] for one configured site, reading its
/// `token_file` fresh. Fails if the token file is unreadable —
/// refreshing with a stale or empty credential would just produce a
/// confusing 401 from the site's manta-server.
fn descriptor_for(
  name: &str,
  site: &SiteConfig,
) -> Result<SiteDescriptor, String> {
  let token = match (&site.token, &site.token_file) {
    (Some(token), None) => token.clone(),
    (None, Some(path)) => std::fs::read_to_string(path)
      .map_err(|e| {
        format!(
          "[sites.{name}] token_file '{}' could not be read: {e}",
          path.display()
        )
      })?
      .trim()
      .to_owned(),
    // validate() rejects the other combinations at startup.
    _ => unreachable!("config validation enforces exactly one token form"),
  };
  Ok(SiteDescriptor {
    name: name.to_owned(),
    manta_server_url: site.manta_server_url.clone(),
    token,
  })
}

/// Rebuild the index from a snapshot store. `BTreeMap` iteration is
/// name-sorted, so the Index's documented "snapshot order" collision
/// rules are deterministic across runs.
fn rebuild(snapshots: &BTreeMap<String, SiteSnapshot>) -> Index {
  Index::from_snapshots(snapshots.values().cloned().collect())
}

/// Fetch every configured site and **replace** the whole cache state
/// with the outcome — a site that fails drops out of the index (and
/// snapshot store) until a refresh reaches it again. Returns the
/// per-site failure messages.
///
/// # Errors
///
/// Errs only when the refresh cannot start at all: an unreadable
/// `token_file` or HTTP-client build failure. Every unreadable
/// `token_file` is recorded against its own site before returning, so
/// `GET /dump` can explain why nothing refreshed — otherwise the
/// periodic loop, which logs and swallows this error, would leave every
/// site with a growing `age_seconds` and no stated cause.
///
/// The two failure modes leave **different** state behind, so a dump
/// reads differently after each:
///
/// - A per-site *fetch* failure reaches the wholesale replace below, so
///   that site loses its snapshot and its timestamp: `in_index: false`,
///   `refreshed_at: null`, `last_error` set.
/// - An unreadable `token_file` returns from here, before anything is
///   replaced, so every site keeps its snapshot, its index entry and its
///   `refreshed_at`, with the error recorded alongside — the same shape
///   [`refresh_site`] leaves behind when it fails. A site carrying both
///   a `last_error` and an old-but-present `refreshed_at` is therefore
///   expected, not a contradiction.
pub async fn refresh_all(state: &AppState) -> Result<Vec<String>, String> {
  // Collect *every* descriptor error rather than returning on the first.
  // `config.sites` is a HashMap, so returning early would blame an
  // arbitrary one of several unreadable token files — and a different
  // one on the next tick — while the rest showed no cause at all.
  let mut descriptors = Vec::with_capacity(state.config.sites.len());
  let mut unusable = Vec::new();
  for (name, site) in &state.config.sites {
    match descriptor_for(name, site) {
      Ok(descriptor) => descriptors.push(descriptor),
      Err(message) => {
        let previous = refreshed_at_of(state, name).await;
        record_failure(state, name, &message, previous).await;
        unusable.push((name.as_str(), message));
      }
    }
  }
  if !unusable.is_empty() {
    // Sorted so the message is stable across ticks despite the HashMap.
    unusable.sort_by_key(|(name, _)| *name);
    return Err(
      unusable
        .into_iter()
        .map(|(_, message)| message)
        .collect::<Vec<_>>()
        .join("; "),
    );
  }
  // Deterministic fetch order (HashMap iteration is not).
  descriptors.sort_by(|a, b| a.name.cmp(&b.name));

  let outcome = manta_cache::fetch_snapshots(&descriptors)
    .await
    .map_err(|e| format!("refresh could not start: {e}"))?;
  let failures: Vec<String> =
    outcome.failures.iter().map(ToString::to_string).collect();

  // A full refresh replaces the store wholesale, so a failed site keeps
  // no data and therefore no `refreshed_at` — unlike `refresh_site`,
  // which leaves the previous snapshot (and its timestamp) serving.
  let now = chrono::Utc::now();
  let mut status: BTreeMap<String, SiteStatus> = outcome
    .snapshots
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

  let snapshots: BTreeMap<String, SiteSnapshot> = outcome
    .snapshots
    .into_iter()
    .map(|s| (s.site.clone(), s))
    .collect();

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

/// Why a [`refresh_site`] call failed.
pub enum SiteRefreshError {
  /// The name does not match any `[sites.<name>]` entry.
  UnknownSite,
  /// The site is configured but its snapshot could not be fetched;
  /// the message names the cause. The cache state is left untouched.
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
  let previous = refreshed_at_of(state, name).await;

  let snapshot = match fetch_one(name, site).await {
    Ok(snapshot) => snapshot,
    Err(message) => {
      // Record why, keeping the previous snapshot and its timestamp:
      // the site still serves, it is just no fresher than it was.
      record_failure(state, name, &message, previous).await;
      return Err(SiteRefreshError::Fetch(message));
    }
  };

  let mut cache = state.cache.write().await;
  cache.snapshots.insert(name.to_owned(), snapshot);
  cache.index = rebuild(&cache.snapshots);
  cache.status.insert(
    name.to_owned(),
    SiteStatus {
      refreshed_at: Some(chrono::Utc::now()),
      last_error: None,
    },
  );
  tracing::info!(site = name, "single-site refresh finished");
  Ok(())
}

/// Fetch exactly one site's snapshot, flattening every failure mode
/// (unreadable token file, fan-out failure, per-site error, empty
/// outcome) into one message.
async fn fetch_one(
  name: &str,
  site: &SiteConfig,
) -> Result<SiteSnapshot, String> {
  let descriptor = descriptor_for(name, site)?;
  let outcome = manta_cache::fetch_snapshots(std::slice::from_ref(&descriptor))
    .await
    .map_err(|e| e.to_string())?;
  if let Some(failure) = outcome.failures.first() {
    return Err(failure.to_string());
  }
  outcome
    .snapshots
    .into_iter()
    .next()
    .ok_or_else(|| "no snapshot returned".to_string())
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
      "[sites.alps]\nmanta_server_url = \"http://127.0.0.1:9\"\n\
       token = \"t\"\n",
    )
    .unwrap();
    AppState {
      cache: tokio::sync::RwLock::new(CacheState {
        status,
        ..Default::default()
      }),
      config: Arc::new(config),
    }
  }

  fn dated(at: chrono::DateTime<chrono::Utc>) -> BTreeMap<String, SiteStatus> {
    BTreeMap::from([(
      "alps".to_owned(),
      SiteStatus {
        refreshed_at: Some(at),
        last_error: None,
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
