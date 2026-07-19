//! Refresh plumbing: turn `[sites.*]` config into `SiteDescriptor`s
//! (resolving `token` / `token_file`) and run the initial + periodic
//! cross-site refreshes.
//!
//! `token_file` is re-read on **every** refresh, so a secret manager
//! (Vault Agent, kubelet-projected secret) can rotate the credential
//! without restarting the service.

use std::sync::Arc;
use std::time::Duration;

use manta_cache::{RefreshOutcome, SiteDescriptor};

use crate::config::CacheServerConfiguration;
use crate::routes::AppState;

/// Build one [`SiteDescriptor`] per configured site, reading each
/// `token_file` fresh. Fails if any token file is unreadable —
/// refreshing a site with a stale or empty credential would just
/// produce a confusing 401 from its manta-server.
pub fn build_descriptors(
  config: &CacheServerConfiguration,
) -> Result<Vec<SiteDescriptor>, String> {
  let mut descriptors = Vec::with_capacity(config.sites.len());
  for (name, site) in &config.sites {
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
    descriptors.push(SiteDescriptor {
      name: name.clone(),
      manta_server_url: site.manta_server_url.clone(),
      token,
    });
  }
  // Deterministic refresh order (HashMap iteration is not), so the
  // Index's documented "snapshot order" collision rules are stable
  // across runs.
  descriptors.sort_by(|a, b| a.name.cmp(&b.name));
  Ok(descriptors)
}

/// Run one cross-site refresh and log its outcome. Per-site failures
/// are already logged at `warn` by `manta_cache::refresh`; this adds
/// the summary line.
pub async fn refresh_once(
  config: &CacheServerConfiguration,
) -> Result<RefreshOutcome, String> {
  let descriptors = build_descriptors(config)?;
  let outcome = manta_cache::refresh(&descriptors)
    .await
    .map_err(|e| format!("refresh could not start: {e}"))?;
  tracing::info!(
    sites = outcome.index.sites().count(),
    failures = outcome.failures.len(),
    "cross-site refresh finished"
  );
  Ok(outcome)
}

/// Spawn the periodic refresh loop. Each tick rebuilds descriptors
/// (picking up rotated `token_file` contents) and **replaces** the
/// whole index with the outcome — a site that fails a tick drops out
/// of the index until a later tick reaches it again, exactly like at
/// startup. Keeping a failed site's stale entries alive is a Stage-4
/// (persistence/merge) concern.
pub fn spawn_periodic(
  state: Arc<AppState>,
  config: Arc<CacheServerConfiguration>,
  interval: Duration,
) {
  tokio::spawn(async move {
    let mut ticker = tokio::time::interval(interval);
    // The immediate first tick would duplicate the startup refresh.
    ticker.tick().await;
    loop {
      ticker.tick().await;
      match refresh_once(&config).await {
        Ok(outcome) => {
          *state.index.write().await = outcome.index;
        }
        Err(e) => {
          // Keep serving the previous index; try again next tick.
          tracing::error!("periodic refresh failed, keeping old index: {e}");
        }
      }
    }
  });
}
