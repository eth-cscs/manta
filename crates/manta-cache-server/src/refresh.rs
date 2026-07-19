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
use crate::routes::AppState;

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
/// `token_file` or HTTP-client build failure.
pub async fn refresh_all(state: &AppState) -> Result<Vec<String>, String> {
  let mut descriptors = Vec::with_capacity(state.config.sites.len());
  for (name, site) in &state.config.sites {
    descriptors.push(descriptor_for(name, site)?);
  }
  // Deterministic fetch order (HashMap iteration is not).
  descriptors.sort_by(|a, b| a.name.cmp(&b.name));

  let outcome = manta_cache::fetch_snapshots(&descriptors)
    .await
    .map_err(|e| format!("refresh could not start: {e}"))?;
  let failures: Vec<String> =
    outcome.failures.iter().map(ToString::to_string).collect();
  let snapshots: BTreeMap<String, SiteSnapshot> = outcome
    .snapshots
    .into_iter()
    .map(|s| (s.site.clone(), s))
    .collect();

  let mut cache = state.cache.write().await;
  cache.index = rebuild(&snapshots);
  cache.snapshots = snapshots;
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
  let descriptor =
    descriptor_for(name, site).map_err(SiteRefreshError::Fetch)?;

  let outcome = manta_cache::fetch_snapshots(std::slice::from_ref(&descriptor))
    .await
    .map_err(|e| SiteRefreshError::Fetch(e.to_string()))?;
  if let Some(failure) = outcome.failures.first() {
    return Err(SiteRefreshError::Fetch(failure.to_string()));
  }
  let snapshot = outcome.snapshots.into_iter().next().ok_or_else(|| {
    SiteRefreshError::Fetch("no snapshot returned".to_string())
  })?;

  let mut cache = state.cache.write().await;
  cache.snapshots.insert(name.to_owned(), snapshot);
  cache.index = rebuild(&cache.snapshots);
  tracing::info!(site = name, "single-site refresh finished");
  Ok(())
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
