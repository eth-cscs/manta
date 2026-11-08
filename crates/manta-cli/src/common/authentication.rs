//! Token acquisition: service account -> cached token -> interactive login.
//!
//! The bearer token is issued by the CSCS Keycloak and obtained by the
//! CLI directly (see [`crate::common::oidc`]); `manta-server` takes no
//! part in the login (it exchanges the token per request for one the
//! site accepts). One token serves every site, so nothing here depends
//! on the selected site.
//!
//! ## Resolution order
//!
//! [`get_api_token`] tries each source in turn and returns the first
//! token it gets:
//!
//! 1. Service-account credentials in `MANTA_SA_CLIENT_ID` and
//!    `MANTA_SA_CLIENT_SECRET` (for CI): a token from the client
//!    credentials grant. Never cached.
//! 2. Cached token at `<cache_dir>/token.json` (`0600`). The cache
//!    directory comes from
//!    [`manta_shared::common::config::get_default_cache_path`]. An
//!    expired access token is refreshed with the cached refresh token
//!    and written back.
//! 3. Interactive OIDC login ([`oidc::login`]: browser, or device code
//!    when headless). The result is written to the cache.
//!
//! ## During a command
//!
//! [`TokenRefresher`] refreshes the token before every request, so each
//! request carries a token no request has carried before. The server
//! exchanges every token it receives, and an OpenCHAMI site's Hydra
//! accepts each token only once (it rejects a reused `jti`). Refreshing
//! every time also keeps a command that polls for hours supplied with a
//! live token, though the CSCS access token lives for minutes. That is
//! why there is no way to pass a ready-made token: it could be sent only
//! once.
//!
//! Tokens are not checked against the server up front. A token the
//! site rejects surfaces as a `401` on the first real request (see
//! [`crate::http_client::OpenApiResultExt`]).
//!
//! Non-interactive callers (`stdin` is not a TTY) stop after the cache
//! instead of starting a login nobody can complete.

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, bail};
use manta_shared::common::config::get_default_cache_path;

use crate::common::app_context::AppContext;
use crate::common::oidc::{self, OidcSettings, TokenStore};

/// No longer read: a ready-made token could be sent only once (see the
/// module docs). Its presence only draws a warning.
const REMOVED_TOKEN_ENV_VAR: &str = "MANTA_TOKEN";

/// Environment variable holding a service account's client id.
pub const SA_CLIENT_ID_ENV_VAR: &str = "MANTA_SA_CLIENT_ID";

/// Environment variable holding a service account's client secret.
pub const SA_CLIENT_SECRET_ENV_VAR: &str = "MANTA_SA_CLIENT_SECRET";

/// File name of the token cache inside the manta cache directory.
const TOKEN_CACHE_FILE: &str = "token.json";

/// Path of the token cache file (`<cache_dir>/token.json`).
///
/// # Errors
///
/// The platform cache directory cannot be resolved.
pub fn token_cache_path() -> Result<PathBuf> {
  Ok(get_default_cache_path()?.join(TOKEN_CACHE_FILE))
}

/// Obtain a bearer token, trying in order: service-account credentials,
/// cached token (refreshed when expired), interactive login.
///
/// A successful refresh or login is written back to the cache file so
/// later invocations — against any site — reuse it.
///
/// # Errors
///
/// - No site is set (`ctx.require_site()` fails).
/// - The cache directory cannot be resolved, or the cache cannot be
///   written after a login.
/// - No token is available and stdin is not a terminal.
/// - The interactive login fails.
#[tracing::instrument(skip_all)]
pub async fn get_api_token(ctx: &AppContext<'_>) -> Result<String> {
  // process_cli pre-resolves the token at the top of every
  // authenticated command and stashes it on AppContext. Handlers
  // continue to call this function unchanged; we just hand back the
  // already-resolved token instead of re-walking the sources.
  if let Some(t) = &ctx.token {
    return Ok(t.clone());
  }

  // The token itself is site-independent, but every caller goes on to
  // send it to a site — fail on a missing site before starting a login.
  ctx.require_site()?;

  if is_set(std::env::var(REMOVED_TOKEN_ENV_VAR).ok().as_deref()) {
    tracing::warn!(
      "{REMOVED_TOKEN_ENV_VAR} is no longer supported and is ignored; use \
       {SA_CLIENT_ID_ENV_VAR} and {SA_CLIENT_SECRET_ENV_VAR} for a service \
       account"
    );
  }

  if let Some(account) = service_account_from_env()? {
    tracing::info!(client_id = %account.client_id, "Using service account");
    return oidc::client_credentials(
      ctx.oidc.issuer_url,
      &account.client_id,
      &account.client_secret,
    )
    .await;
  }

  let cache_path = token_cache_path()?;
  match token_from_cache(&ctx.oidc, &cache_path).await {
    Ok(Some(token)) => {
      tracing::info!("Using cached authentication token");
      return Ok(token);
    }
    Ok(None) => {}
    Err(err) => {
      tracing::warn!(error = %err, "cached token unusable, logging in again");
    }
  }

  if !io::stdin().is_terminal() {
    bail!(
      "no valid cached token and stdin is not a terminal, so cannot log \
       in interactively. Run manta interactively once to log in, or set \
       {SA_CLIENT_ID_ENV_VAR} and {SA_CLIENT_SECRET_ENV_VAR} for a service \
       account."
    );
  }

  tracing::info!(issuer = %ctx.oidc.issuer_url, "Logging in interactively");
  let store = oidc::login(&ctx.oidc).await?;
  store.save(&cache_path)?;
  tracing::info!(path = %cache_path.display(), "Authentication token cached on disk");
  Ok(store.access_token)
}

/// Supplies the token for each request of a command: always a new one,
/// from the cached refresh token or the service account's credentials.
///
/// Never the cached access token as is: another request, or another
/// `manta` run, may have sent it already, and Hydra would refuse it. The
/// cache file is read every time, under a lock shared with other `manta`
/// runs, so none of them refreshes with a refresh token another has
/// already used (Keycloak may refuse a rotated one).
#[derive(Debug)]
pub struct TokenRefresher {
  issuer_url: String,
  client_id: String,
  headless: bool,
  source: TokenSource,
  /// Parallel requests wait for one refresh rather than starting their own.
  refreshing: tokio::sync::Mutex<()>,
}

/// Where [`TokenRefresher`] gets each new token.
#[derive(Debug)]
enum TokenSource {
  /// The refresh token in the token cache at this path.
  Cache(PathBuf),
  /// The client credentials grant.
  ServiceAccount(ServiceAccount),
}

impl TokenRefresher {
  /// Refresh from the token cache at `cache_path`.
  pub(crate) fn new(settings: &OidcSettings<'_>, cache_path: PathBuf) -> Self {
    Self::with_source(settings, TokenSource::Cache(cache_path))
  }

  /// Fetch each token with `account`'s credentials.
  pub(crate) fn for_service_account(
    settings: &OidcSettings<'_>,
    account: ServiceAccount,
  ) -> Self {
    Self::with_source(settings, TokenSource::ServiceAccount(account))
  }

  fn with_source(settings: &OidcSettings<'_>, source: TokenSource) -> Self {
    Self {
      issuer_url: settings.issuer_url.to_string(),
      client_id: settings.client_id.to_string(),
      headless: settings.headless,
      source,
      refreshing: tokio::sync::Mutex::new(()),
    }
  }

  /// A new token to send now. `None` when there is no cached refresh token
  /// or getting a token fails: the caller then keeps the token it has, and
  /// the server answers if it refuses that one. Never starts a login.
  pub async fn current_token(&self) -> Option<String> {
    let _guard = self.refreshing.lock().await;
    match &self.source {
      TokenSource::Cache(cache_path) => {
        let settings = OidcSettings {
          issuer_url: &self.issuer_url,
          client_id: &self.client_id,
          headless: self.headless,
        };
        fresh_token_from_cache(&settings, cache_path)
          .await
          .unwrap_or_else(|err| {
            tracing::warn!(error = %err, "cached token unusable");
            None
          })
      }
      TokenSource::ServiceAccount(account) => oidc::client_credentials(
        &self.issuer_url,
        &account.client_id,
        &account.client_secret,
      )
      .await
      .map_err(
        |err| tracing::warn!(error = %err, "service-account login failed"),
      )
      .ok(),
    }
  }
}

/// A service account's client credentials, from [`SA_CLIENT_ID_ENV_VAR`]
/// and [`SA_CLIENT_SECRET_ENV_VAR`].
pub struct ServiceAccount {
  client_id: String,
  client_secret: String,
}

impl std::fmt::Debug for ServiceAccount {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServiceAccount")
      .field("client_id", &self.client_id)
      .field("client_secret", &"<REDACTED>")
      .finish()
  }
}

#[cfg(test)]
impl ServiceAccount {
  pub(crate) fn new(client_id: &str, client_secret: &str) -> Self {
    Self {
      client_id: client_id.to_string(),
      client_secret: client_secret.to_string(),
    }
  }
}

/// The service account in the environment, if any.
///
/// # Errors
///
/// Only one of the two variables is set.
pub fn service_account_from_env() -> Result<Option<ServiceAccount>> {
  service_account_from(
    std::env::var(SA_CLIENT_ID_ENV_VAR).ok(),
    std::env::var(SA_CLIENT_SECRET_ENV_VAR).ok(),
  )
}

fn service_account_from(
  client_id: Option<String>,
  client_secret: Option<String>,
) -> Result<Option<ServiceAccount>> {
  let client_id = client_id.filter(|v| is_set(Some(v)));
  let client_secret = client_secret.filter(|v| is_set(Some(v)));
  match (client_id, client_secret) {
    (Some(client_id), Some(client_secret)) => Ok(Some(ServiceAccount {
      client_id,
      client_secret,
    })),
    (None, None) => Ok(None),
    _ => bail!(
      "set both {SA_CLIENT_ID_ENV_VAR} and {SA_CLIENT_SECRET_ENV_VAR} for a \
       service account, or neither"
    ),
  }
}

/// The refresher for a command: the service account's when one is set,
/// else the token cache's.
///
/// # Errors
///
/// Only one service-account variable is set, or the cache directory cannot
/// be resolved.
pub fn token_refresher(ctx: &AppContext<'_>) -> Result<Arc<TokenRefresher>> {
  if let Some(account) = service_account_from_env()? {
    return Ok(Arc::new(TokenRefresher::for_service_account(
      &ctx.oidc, account,
    )));
  }
  Ok(Arc::new(TokenRefresher::new(
    &ctx.oidc,
    token_cache_path()?,
  )))
}

/// Whether an environment variable holds something other than blanks.
fn is_set(value: Option<&str>) -> bool {
  value.is_some_and(|t| !t.trim().is_empty())
}

/// The cached token at `path`. Returns the access token when
/// still valid; otherwise refreshes it (saving the result) when a
/// refresh token is cached. `Ok(None)` when there is no cache or it
/// cannot be refreshed — the caller then logs in interactively.
async fn token_from_cache(
  settings: &OidcSettings<'_>,
  path: &Path,
) -> Result<Option<String>> {
  if !path.exists() {
    return Ok(None);
  }
  let store = TokenStore::load(path)?;
  if !store.is_expired() {
    return Ok(Some(store.access_token));
  }
  let Some(refresh_token) = &store.refresh_token else {
    return Ok(None);
  };
  let token = refresh_and_save(settings, refresh_token, path).await?;
  if token.is_some() {
    tracing::info!("Access token refreshed");
  }
  Ok(token)
}

/// A new access token from the cached refresh token, whatever the cached
/// access token's expiry. `Ok(None)` when there is no cache, no refresh
/// token, or the refresh fails.
async fn fresh_token_from_cache(
  settings: &OidcSettings<'_>,
  path: &Path,
) -> Result<Option<String>> {
  if !path.exists() {
    return Ok(None);
  }
  let _lock = lock_cache(path).await;
  let store = TokenStore::load(path)?;
  let Some(refresh_token) = &store.refresh_token else {
    return Ok(None);
  };
  let token = refresh_and_save(settings, refresh_token, path).await?;
  if token.is_some() {
    tracing::debug!("Access token refreshed for this request");
  }
  Ok(token)
}

/// Hold `<cache>.lock` exclusively until the returned file is dropped, so
/// two `manta` runs never refresh at once. `None`, with a warning, when the
/// lock cannot be taken; the refresh then goes ahead without it.
async fn lock_cache(path: &Path) -> Option<std::fs::File> {
  let lock_path = path.with_extension("lock");
  let locked = tokio::task::spawn_blocking(move || {
    let file = std::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(false)
      .open(&lock_path)?;
    file.lock()?;
    Ok::<_, std::io::Error>(file)
  })
  .await;
  match locked {
    Ok(Ok(file)) => Some(file),
    Ok(Err(err)) => {
      tracing::warn!(error = %err, "cannot lock the token cache");
      None
    }
    Err(err) => {
      tracing::warn!(error = %err, "cannot lock the token cache");
      None
    }
  }
}

/// Refresh with `refresh_token` and save the result at `path`. `Ok(None)`
/// when the Keycloak refuses (the refresh token expired or was revoked).
async fn refresh_and_save(
  settings: &OidcSettings<'_>,
  refresh_token: &str,
  path: &Path,
) -> Result<Option<String>> {
  match oidc::refresh(settings, refresh_token).await {
    Ok(refreshed) => {
      refreshed.save(path)?;
      Ok(Some(refreshed.access_token))
    }
    Err(err) => {
      tracing::warn!(error = %err, "token refresh failed");
      Ok(None)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn settings() -> OidcSettings<'static> {
    // Unroutable issuer: any attempt to reach the Keycloak fails fast,
    // so a test that expects no network call would surface one.
    OidcSettings {
      issuer_url: "http://127.0.0.1:9",
      client_id: "test-client",
      headless: true,
    }
  }

  fn store(expires_at: i64, refresh_token: Option<&str>) -> TokenStore {
    TokenStore {
      access_token: "cached-access".to_string(),
      refresh_token: refresh_token.map(str::to_string),
      id_token: None,
      expires_at: Some(expires_at),
    }
  }

  #[test]
  fn a_service_account_needs_both_variables() {
    let s = |v: &str| Some(v.to_string());
    assert!(
      service_account_from(s("id"), s("secret"))
        .unwrap()
        .is_some()
    );
    assert!(service_account_from(None, None).unwrap().is_none());
    assert!(service_account_from(s(" "), s(" ")).unwrap().is_none());
    assert!(service_account_from(s("id"), None).is_err());
    assert!(service_account_from(None, s("secret")).is_err());
  }

  #[test]
  fn service_account_debug_redacts_the_secret() {
    let account = ServiceAccount::new("ci-bot", "hunter2");
    let rendered = format!("{account:?}");
    assert!(rendered.contains("ci-bot"), "{rendered}");
    assert!(!rendered.contains("hunter2"), "{rendered}");
  }

  #[test]
  fn only_a_non_blank_value_counts_as_set() {
    assert!(is_set(Some("abc")));
    assert!(!is_set(Some("  ")));
    assert!(!is_set(None));
  }

  #[tokio::test]
  async fn missing_cache_yields_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    assert!(
      token_from_cache(&settings(), &path)
        .await
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn valid_cached_token_is_returned_without_refresh() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    store(oidc::now_secs() + 3600, Some("refresh"))
      .save(&path)
      .unwrap();
    assert_eq!(
      token_from_cache(&settings(), &path)
        .await
        .unwrap()
        .as_deref(),
      Some("cached-access")
    );
  }

  #[tokio::test]
  async fn expired_cached_token_without_refresh_token_yields_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    store(0, None).save(&path).unwrap();
    assert!(
      token_from_cache(&settings(), &path)
        .await
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn failed_refresh_yields_none_and_keeps_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    let expired = store(0, Some("refresh"));
    expired.save(&path).unwrap();
    assert!(
      token_from_cache(&settings(), &path)
        .await
        .unwrap()
        .is_none()
    );
    assert_eq!(TokenStore::load(&path).unwrap(), expired);
  }

  #[tokio::test]
  async fn corrupt_cache_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    std::fs::write(&path, "not json").unwrap();
    assert!(token_from_cache(&settings(), &path).await.is_err());
  }
}
