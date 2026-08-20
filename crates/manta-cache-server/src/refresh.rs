//! Refresh plumbing: turn `[sites.*]` config into `SiteDescriptor`s
//! (reading each `token_file`) and run the full and per-site refreshes
//! against the shared [`AppState`].
//!
//! `token_file` is re-read on **every** refresh, so a secret manager
//! (Vault Agent, kubelet-projected secret) can rotate the credential
//! without restarting the service. Each read also yields a
//! [`CredentialInfo`], which is stored per site so `GET /dump` can say
//! who a site refreshes as and when that credential lapses.
//!
//! The state keeps the last-good [`SiteSnapshot`] per site so a
//! [`refresh_site`] can rebuild the index from one fresh snapshot plus
//! the stored siblings, without re-fetching every site.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use manta_cache::{Index, SiteDescriptor, SiteSnapshot};

use crate::config::SiteConfig;
use crate::credential::{self, CredentialInfo};
use crate::routes::{AppState, SiteStatus};

/// Build the [`SiteDescriptor`] for one configured site, reading its
/// `token_file` fresh, along with what the credential says about itself.
///
/// Fails if the token file is unreadable or blank — refreshing with a
/// missing credential would just produce a confusing 401 from the site's
/// manta-server, several layers away from the actual problem.
fn descriptor_for(
  name: &str,
  site: &SiteConfig,
) -> Result<(SiteDescriptor, CredentialInfo), String> {
  let token = std::fs::read_to_string(&site.token_file)
    .map_err(|e| {
      format!(
        "[sites.{name}] token_file '{}' could not be read: {e}",
        site.token_file.display()
      )
    })?
    .trim()
    .to_owned();
  if token.is_empty() {
    return Err(format!(
      "[sites.{name}] token_file '{}' is empty; write the site's \
       service-account token into it.",
      site.token_file.display()
    ));
  }
  let info = credential::inspect(&token);
  Ok((
    SiteDescriptor {
      name: name.to_owned(),
      manta_server_url: site.manta_server_url.clone(),
      token,
    },
    info,
  ))
}

/// Is there anything new to say about this site's credential?
///
/// Compares against the [`SiteStatus`] left by the previous refresh:
/// the credential it used, and the warnings that were **actually
/// logged** for it. `warnings` is the freshly-computed list for the
/// current instant.
///
/// Taking the previous warnings as *stored data* is the whole point. The
/// tempting formulation — `previous.credential.warnings(now)` versus
/// `current.warnings(now)` — re-derives both sides at the same instant,
/// so for an unchanged token file they are equal by construction. That
/// comparison is a tautology that suppresses every message after the
/// first, meaning a server booted eleven months before expiry would
/// never report the lapse this exists to warn about.
///
/// The credential is compared too, so rotating a site from one healthy
/// service account to another is announced even though neither carries a
/// warning.
fn credential_state_changed(
  previous: Option<&SiteStatus>,
  current: &CredentialInfo,
  warnings: &[String],
) -> bool {
  !previous.is_some_and(|p| {
    p.credential.as_ref() == Some(current) && p.credential_warnings == warnings
  })
}

/// Log a credential's state when [`credential_state_changed`] says there
/// is something new, and return the current warnings so the caller can
/// store them as the next comparison point.
fn log_credential_change(
  site: &str,
  previous: Option<&SiteStatus>,
  current: &CredentialInfo,
  now: chrono::DateTime<chrono::Utc>,
) -> Vec<String> {
  let warnings = current.warnings(now);
  if !credential_state_changed(previous, current, &warnings) {
    return warnings;
  }

  if warnings.is_empty() {
    tracing::info!(site, credential = %current.summary(now), "credential ok");
    return warnings;
  }
  // An expired credential is an outage, not a heads-up: every line about
  // it belongs at error level, including a second warning (a personal
  // token, say) that would otherwise be a mere advisory.
  for warning in &warnings {
    if current.is_expired(now) {
      tracing::error!(site, "{warning}");
    } else {
      tracing::warn!(site, "{warning}");
    }
  }
  warnings
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
  let started = chrono::Utc::now();

  // Snapshotted before anything is replaced: the wholesale swap below
  // drops the previous statuses, and without them every tick would
  // re-log warnings that have not changed since the last one.
  let previous_status = status_snapshot(state).await;

  // Collect *every* descriptor error rather than returning on the first.
  // `config.sites` is a HashMap, so returning early would blame an
  // arbitrary one of several unreadable token files — and a different
  // one on the next tick — while the rest showed no cause at all.
  let mut descriptors = Vec::with_capacity(state.config.sites.len());
  let mut credentials: BTreeMap<String, (CredentialInfo, Vec<String>)> =
    BTreeMap::new();
  let mut unusable = Vec::new();
  for (name, site) in &state.config.sites {
    match descriptor_for(name, site) {
      Ok((descriptor, info)) => {
        let warnings = log_credential_change(
          name,
          previous_status.get(name),
          &info,
          started,
        );
        credentials.insert(name.clone(), (info, warnings));
        descriptors.push(descriptor);
      }
      Err(message) => {
        let previous = refreshed_at_of(state, name).await;
        record_failure(state, name, &message, previous).await;
        unusable.push((name.as_str(), message));
      }
    }
  }
  // Persist what the loop just logged, **before** any exit can skip it.
  //
  // The dedup rests on one invariant: anything logged has been stored.
  // The loop visits every site before either bail-out below, so a
  // healthy site's credential is logged even when a sibling's
  // `token_file` is unreadable or the HTTP client fails to build.
  // Storing on each exit path separately would leave the next added `?`
  // to reintroduce the flood — every tick recomputing the same delta and
  // re-logging it, at ERROR once the credential lapses, for as long as
  // the persistent fault lasts. Keeping the store adjacent to the log
  // makes that unrepresentable rather than merely fixed. The wholesale
  // replace at the end folds the same data in again, which is
  // idempotent.
  store_credentials(state, &credentials).await;

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

  // Stamped after the fan-out, not from `started`: the fetch can take up
  // to the per-request timeout (300s), and `refreshed_at` documents when
  // the held snapshot was *fetched*. Dating it from the start of the
  // refresh would overstate `age_seconds` by however long the slowest
  // site took.
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
          ..Default::default()
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
          last_error: Some(failure.to_string()),
          ..Default::default()
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
        last_error: Some(message),
        ..Default::default()
      },
    );
  }
  // Attached last so it survives whichever branch above wrote the entry:
  // knowing *which* credential a failing site used is most of the
  // diagnosis.
  for (name, (info, warnings)) in credentials {
    let entry = status.entry(name).or_default();
    entry.credential = Some(info);
    entry.credential_warnings = warnings;
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
/// Keyed on the labels alone, because they come from
/// `/groups/available`, which csm-rs derives from the token's realm
/// roles — so an empty list is precisely the shape a credential with no
/// HSM roles produces, which is the condition worth catching.
///
/// Note this can be true of a snapshot that still holds nodes, and
/// [`refresh_all`] discards those too. In practice `/groups/nodes` is
/// scoped by the same roles and empties alongside the labels; the
/// deliberate choice is that a label-less site is reported as broken
/// rather than half-served, since resolving xnames for a site whose
/// groups have all vanished would mask the credential problem.
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
/// It means the credential resolved to no HSM roles — expired, wrong
/// account, roles removed, or aimed at the wrong site.
fn empty_snapshot_error(snapshot: &SiteSnapshot) -> String {
  format!(
    "site '{}' returned no groups (labels=0 nodes={}): the credential \
     resolves to no HSM roles — check the service account's Keycloak \
     roles and that its token has not expired",
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
  /// untouched, so the site keeps serving what it had; its
  /// [`SiteStatus`] is not — the failure is recorded there, along with
  /// the credential that produced it.
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
  // status it is about to stamp is still the one it observed, and so the
  // credential logging has its comparison point.
  let previous = status_snapshot(state).await;
  let previous_refreshed_at =
    previous.get(name).and_then(|status| status.refreshed_at);
  let started = chrono::Utc::now();

  let (snapshot, info, warnings) = match fetch_one(name, site).await {
    Ok((snapshot, info)) => {
      let warnings =
        log_credential_change(name, previous.get(name), &info, started);
      (snapshot, Some(info), warnings)
    }
    Err(FetchOneError {
      message,
      credential,
    }) => {
      // Record why, keeping the previous snapshot and its timestamp:
      // the site still serves, it is just no fresher than it was.
      record_failure(state, name, &message, previous_refreshed_at).await;
      if let Some(info) = credential {
        // Log the credential here too: a single-site refresh that fails
        // *because* the token expired must still say so, and this is the
        // only path that reaches it.
        let warnings =
          log_credential_change(name, previous.get(name), &info, started);
        let mut cache = state.cache.write().await;
        let entry = cache.status.entry(name.to_owned()).or_default();
        entry.credential = Some(info);
        entry.credential_warnings = warnings;
      }
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
      credential: info,
      credential_warnings: warnings,
    },
  );
  tracing::info!(site = name, "single-site refresh finished");
  Ok(())
}

/// Why a [`fetch_one`] call failed, plus whatever was learned about the
/// credential before it did.
///
/// The credential survives the failure on purpose: a site that fails to
/// refresh is exactly when an operator wants to know which account it
/// was trying to refresh as, and `GET /dump` can only show that if the
/// error path carries it.
struct FetchOneError {
  /// Operator-facing explanation, also returned to the management API.
  message: String,
  /// `None` when the failure happened before the token could be read.
  credential: Option<CredentialInfo>,
}

/// Fetch exactly one site's snapshot, flattening every failure mode
/// (unreadable token file, fan-out failure, per-site error, empty
/// outcome, empty snapshot) into one message.
async fn fetch_one(
  name: &str,
  site: &SiteConfig,
) -> Result<(SiteSnapshot, CredentialInfo), FetchOneError> {
  let (descriptor, info) =
    descriptor_for(name, site).map_err(|message| FetchOneError {
      message,
      credential: None,
    })?;
  let fail = |message: String| FetchOneError {
    message,
    credential: Some(info.clone()),
  };

  let outcome = manta_cache::fetch_snapshots(std::slice::from_ref(&descriptor))
    .await
    .map_err(|e| fail(e.to_string()))?;
  if let Some(failure) = outcome.failures.first() {
    return Err(fail(failure.to_string()));
  }
  let snapshot = outcome
    .snapshots
    .into_iter()
    .next()
    .ok_or_else(|| fail("no snapshot returned".to_string()))?;
  if is_empty(&snapshot) {
    return Err(fail(empty_snapshot_error(&snapshot)));
  }
  Ok((snapshot, info))
}

/// Copy of the current per-site status, taken before a refresh mutates
/// it. Carries both the previous credential and the warnings last logged
/// for it, which is what [`log_credential_change`] compares against.
async fn status_snapshot(state: &AppState) -> BTreeMap<String, SiteStatus> {
  state.cache.read().await.status.clone()
}

/// Record each site's credential and the warnings just logged for it,
/// leaving the rest of its status alone.
///
/// Called immediately after the logging loop so no exit between there
/// and the wholesale replace can skip it. Takes a reference because the
/// replace consumes the same map afterwards.
///
/// One write lock spans the whole loop deliberately: the dedup predicate
/// compares `credential` and `credential_warnings` **together**, so a
/// reader that caught one updated without the other would register a
/// spurious change and re-log.
async fn store_credentials(
  state: &AppState,
  credentials: &BTreeMap<String, (CredentialInfo, Vec<String>)>,
) {
  if credentials.is_empty() {
    return;
  }
  let mut cache = state.cache.write().await;
  for (name, (info, warnings)) in credentials {
    let entry = cache.status.entry(name.clone()).or_default();
    entry.credential = Some(info.clone());
    entry.credential_warnings = warnings.clone();
  }
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
       token_file = \"/run/secrets/alps\"\n",
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
        ..Default::default()
      },
    )])
  }

  /// A status entry as it stands after a refresh logged `warnings` for
  /// `info`.
  fn logged(info: &CredentialInfo, warnings: Vec<String>) -> SiteStatus {
    SiteStatus {
      credential: Some(info.clone()),
      credential_warnings: warnings,
      ..Default::default()
    }
  }

  #[test]
  fn an_expiry_warning_is_logged_when_it_first_becomes_true() {
    // The regression this guards: deciding by
    // `previous.credential.warnings(now) == current.warnings(now)`
    // re-derives both sides at the same instant, so an unchanged token
    // file always compares equal and every message after the first is
    // suppressed. A server booted eleven months before expiry would then
    // never report the lapse. Asserting on the *decision* is essential —
    // the warnings themselves are returned either way, so a test that
    // checks only the return value passes against the broken version.
    let info =
      credential::inspect(&credential::test_support::service_account_token(
        chrono::Utc::now() + chrono::Duration::days(365),
      ));

    // Eleven months in: quiet, and that silence is what gets stored.
    let early = chrono::Utc::now() + chrono::Duration::days(300);
    let early_warnings = info.warnings(early);
    assert!(early_warnings.is_empty());
    assert!(
      credential_state_changed(None, &info, &early_warnings),
      "a site's first credential is always worth announcing"
    );

    // Same credential, same file, now inside the 30-day window. The
    // stored warnings are empty and the current ones are not, so there
    // is something new to say.
    let late = chrono::Utc::now() + chrono::Duration::days(350);
    let late_warnings = info.warnings(late);
    assert_eq!(late_warnings.len(), 1, "{late_warnings:?}");
    let previous = logged(&info, early_warnings);
    assert!(
      credential_state_changed(Some(&previous), &info, &late_warnings),
      "the expiry warning must be logged when it first becomes true"
    );

    // Next tick, nothing changed: stays out of the log.
    let previous = logged(&info, late_warnings.clone());
    assert!(
      !credential_state_changed(Some(&previous), &info, &late_warnings),
      "an unchanged warning must not repeat on every tick"
    );
  }

  #[test]
  fn a_rotated_credential_is_noticed_even_when_both_are_healthy() {
    // Swapping service account A for B would go unlogged if the change
    // key were the warnings alone — both are quiet. The principal
    // changing is exactly what an operator wants to see.
    let long = chrono::Utc::now() + chrono::Duration::days(365);
    let a = credential::inspect(
      &credential::test_support::service_account_token(long),
    );
    let b =
      credential::inspect(&credential::test_support::jwt(serde_json::json!({
        "azp": "manta-cache-rotated",
        "preferred_username": "service-account-manta-cache-rotated",
        "exp": long.timestamp(),
      })));
    let warnings = b.warnings(chrono::Utc::now());
    assert!(warnings.is_empty(), "both accounts are healthy");

    let previous = logged(&a, Vec::new());
    assert!(credential_state_changed(Some(&previous), &b, &warnings));
    // ...and re-reading the same file next tick says nothing.
    let previous = logged(&b, Vec::new());
    assert!(!credential_state_changed(Some(&previous), &b, &warnings));
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
