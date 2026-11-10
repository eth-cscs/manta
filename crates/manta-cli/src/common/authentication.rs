//! The bearer token of every request to manta-server.
//!
//! The token is issued by the CSCS Keycloak and obtained by the CLI
//! directly (see [`crate::common::oidc`]); `manta-server` takes no part
//! in the login (it exchanges the token per request for one the site
//! accepts). One token serves every site, so nothing here depends on
//! the selected site.
//!
//! [`TokenProvider::token`] is the only way to a token. The generated
//! client's pre-request hook ([`crate::http_client::set_current_token`])
//! and the raw log-stream and console connections call it for every
//! request. Nothing is resolved before a command runs: the command's
//! first request (the session's `GET /groups/available`) is where a
//! login happens when one is needed.
//!
//! ## Where each token comes from
//!
//! 1. Service-account credentials in `MANTA_SA_CLIENT_ID` and
//!    `MANTA_SA_CLIENT_SECRET` (for CI): the client credentials grant.
//!    Nothing is cached.
//! 2. Otherwise the refresh token in `<cache_dir>/token.json` (`0600`,
//!    the directory from
//!    [`manta_shared::common::config::get_default_cache_path`]): the
//!    refresh grant. The new tokens are written back.
//! 3. When there is no usable cache, or the Keycloak refuses the cached
//!    refresh token (`invalid_grant`: the session ended or was revoked),
//!    an interactive login ([`oidc::login`]: browser, or device code
//!    when headless), written to the cache. Only when stderr is a
//!    terminal: both flows talk to the user there (the browser flow's
//!    fallback URL, the device flow's code), and neither reads stdin, so
//!    `… | manta` can still log in. Otherwise the request fails and says
//!    how to log in.
//!
//! Any other failure (the Keycloak unreachable or answering 5xx) fails
//! the request without a login, which would not help.
//!
//! ## A new token for every request
//!
//! The server exchanges every token it receives, and an OpenCHAMI site's
//! Hydra accepts each token only once (it rejects a reused `jti`). So
//! even a valid cached access token is never sent: every request gets a
//! token no request has carried before. That also keeps a command that
//! polls for hours supplied with a live token, though the CSCS access
//! token lives for minutes, and it is why there is no way to pass a
//! ready-made token: it could be sent only once.
//!
//! ## Concurrency
//!
//! Getting a token from the cache holds an exclusive lock on
//! `<cache>.lock`, through a login too. Every request opens the lock
//! file anew, so the lock orders parallel requests of one run as well as
//! separate `manta` runs. So each refresh uses the refresh token the
//! previous one saved, which a Keycloak that rotates refresh tokens
//! requires, and a login one request starts is reused by the others,
//! not repeated. Service-account tokens need no ordering.

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError};

use anyhow::{Context, Result, bail};
use manta_shared::common::config::get_default_cache_path;

use crate::common::app_context::AppContext;
use crate::common::oidc::{self, OidcSettings, TokenStore};

/// The ready-made token earlier releases read. No longer read: a token
/// could be sent only once (see the module docs). Its presence only
/// draws a warning. (Not `MANTA_TOKEN`: API.md's curl examples use that
/// name for a shell variable.)
const REMOVED_TOKEN_ENV_VAR: &str = "MANTA_CSM_TOKEN";

/// Environment variable holding a service account's client id.
pub const SA_CLIENT_ID_ENV_VAR: &str = "MANTA_SA_CLIENT_ID";

/// Environment variable holding a service account's client secret.
pub const SA_CLIENT_SECRET_ENV_VAR: &str = "MANTA_SA_CLIENT_SECRET";

/// How long to wait for the token cache lock before saying so. A refresh
/// holds it well under this; a login another command started holds it
/// for minutes.
const LOCK_WAIT_NOTICE_AFTER: std::time::Duration =
  std::time::Duration::from_secs(2);

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

/// Supplies the token of every request of a command: always a new one
/// (see the module docs).
#[derive(Debug)]
pub struct TokenProvider {
  issuer_url: String,
  client_id: String,
  headless: bool,
  /// Whether a login may be started: someone is at the terminal.
  interactive: bool,
  source: TokenSource,
  /// The token handed out last, for reading its claims.
  last_issued: std::sync::Mutex<Option<String>>,
}

/// Where [`TokenProvider`] gets each new token.
#[derive(Debug)]
enum TokenSource {
  /// The token cache at this path: its refresh token, or a login.
  Cache(PathBuf),
  /// The client credentials grant.
  ServiceAccount(ServiceAccount),
}

impl TokenProvider {
  /// Tokens from the token cache at `cache_path`, logging in when
  /// `interactive` and the cache cannot supply one.
  pub(crate) fn from_cache(
    settings: &OidcSettings<'_>,
    cache_path: PathBuf,
    interactive: bool,
  ) -> Self {
    Self::with_source(settings, TokenSource::Cache(cache_path), interactive)
  }

  /// Tokens from `account`'s credentials. Never logs in.
  pub(crate) fn for_service_account(
    settings: &OidcSettings<'_>,
    account: ServiceAccount,
  ) -> Self {
    Self::with_source(settings, TokenSource::ServiceAccount(account), false)
  }

  fn with_source(
    settings: &OidcSettings<'_>,
    source: TokenSource,
    interactive: bool,
  ) -> Self {
    Self {
      issuer_url: settings.issuer_url.to_string(),
      client_id: settings.client_id.to_string(),
      headless: settings.headless,
      interactive,
      source,
      last_issued: std::sync::Mutex::new(None),
    }
  }

  /// A new token to send now.
  ///
  /// # Errors
  ///
  /// - The service account's credentials are refused, or the Keycloak
  ///   cannot be reached.
  /// - The cached refresh token cannot be used for a reason a login
  ///   would not fix (the Keycloak unreachable or failing).
  /// - A login is needed but stderr is not a terminal, or the login
  ///   fails.
  pub async fn token(&self) -> Result<String> {
    let token = match &self.source {
      TokenSource::Cache(cache_path) => {
        self.token_from_cache(cache_path).await?
      }
      TokenSource::ServiceAccount(account) => oidc::client_credentials(
        &self.issuer_url,
        &account.client_id,
        &account.client_secret,
      )
      .await
      .context("could not get a token for the service account")?,
    };
    *self
      .last_issued
      .lock()
      .unwrap_or_else(PoisonError::into_inner) = Some(token.clone());
    Ok(token)
  }

  /// The token [`Self::token`] handed out last, if any.
  pub fn last_issued(&self) -> Option<String> {
    self
      .last_issued
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .clone()
  }

  fn settings(&self) -> OidcSettings<'_> {
    OidcSettings {
      issuer_url: &self.issuer_url,
      client_id: &self.client_id,
      headless: self.headless,
    }
  }

  /// Refresh with the cached refresh token, or log in when there is
  /// none or the Keycloak refuses it. Under the cache lock throughout.
  async fn token_from_cache(&self, path: &Path) -> Result<String> {
    let settings = self.settings();
    let _lock = lock_cache(path).await;

    let why_login = match cached_refresh_token(path, &settings) {
      Ok(refresh_token) => {
        match oidc::refresh(&settings, &refresh_token).await {
          Ok(refreshed) => {
            save_or_warn(&refreshed, path);
            tracing::debug!("Access token refreshed for this request");
            return Ok(refreshed.access_token);
          }
          Err(err) if err.is::<oidc::RefreshRefused>() => err.to_string(),
          Err(err) => {
            return Err(err.context("could not refresh the token"));
          }
        }
      }
      Err(why) => why,
    };

    if !self.interactive {
      bail!(
        "{why_login}, and stderr is not a terminal, so cannot log in \
         interactively. Run manta interactively once to log in, or set \
         {SA_CLIENT_ID_ENV_VAR} and {SA_CLIENT_SECRET_ENV_VAR} for a \
         service account."
      );
    }
    tracing::info!(issuer = %self.issuer_url, reason = %why_login, "Logging in interactively");
    let store = oidc::login(&settings).await?;
    save_or_warn(&store, path);
    Ok(store.access_token)
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

/// The token provider for a command: the service account's when one is
/// set, else the token cache's.
///
/// # Errors
///
/// Only one service-account variable is set, or the cache directory
/// cannot be resolved.
pub fn token_provider(ctx: &AppContext<'_>) -> Result<Arc<TokenProvider>> {
  if is_set(std::env::var(REMOVED_TOKEN_ENV_VAR).ok().as_deref()) {
    tracing::warn!(
      "{REMOVED_TOKEN_ENV_VAR} is no longer supported and is ignored; use \
       {SA_CLIENT_ID_ENV_VAR} and {SA_CLIENT_SECRET_ENV_VAR} for a service \
       account"
    );
  }

  if let Some(account) = service_account_from_env()? {
    tracing::info!(client_id = %account.client_id, "Using service account");
    return Ok(Arc::new(TokenProvider::for_service_account(
      &ctx.oidc, account,
    )));
  }
  Ok(Arc::new(TokenProvider::from_cache(
    &ctx.oidc,
    token_cache_path()?,
    io::stderr().is_terminal(),
  )))
}

/// Whether an environment variable holds something other than blanks.
fn is_set(value: Option<&str>) -> bool {
  value.is_some_and(|t| !t.trim().is_empty())
}

/// The refresh token in the cache at `path`, if it is one `settings`'
/// issuer and client gave out. Otherwise why there is none, which a
/// login fixes: no cache, no refresh token in it, a cache that cannot be
/// read (e.g. written by an older manta), or one from another issuer or
/// client, whose refresh token must not go to this one.
fn cached_refresh_token(
  path: &Path,
  settings: &OidcSettings<'_>,
) -> Result<String, String> {
  if !path.exists() {
    return Err("not logged in".to_string());
  }
  let store = TokenStore::load(path)
    .map_err(|err| format!("the token cache is unusable ({err:#})"))?;
  if !store.is_from(settings) {
    return Err(format!(
      "logged in at {} as client {}, not at the configured {} as {}",
      store.issuer_url,
      store.client_id,
      settings.issuer_url,
      settings.client_id
    ));
  }
  store
    .refresh_token
    .ok_or_else(|| "not logged in (no refresh token cached)".to_string())
}

/// Save `store` at `path`. A failure only warns: the token in hand is
/// still good for this request, and the next one refreshes or logs in.
fn save_or_warn(store: &TokenStore, path: &Path) {
  if let Err(err) = store.save(path) {
    tracing::warn!(error = %format!("{err:#}"), "could not save the token cache");
  }
}

/// Hold `<cache>.lock` exclusively until the returned file is dropped, so
/// no two requests, of one `manta` run or of several, refresh at once.
/// Each call opens the file anew: `flock` locks taken through separate
/// opens exclude each other within one process too. `None`, with a
/// warning, when the lock cannot be taken; the refresh then goes ahead
/// without it.
///
/// Waits as long as the holder needs, which during a login is minutes.
/// After [`LOCK_WAIT_NOTICE_AFTER`] it says so on stderr, so the wait
/// does not look like a hang.
async fn lock_cache(path: &Path) -> Option<std::fs::File> {
  let lock_path = path.with_extension("lock");
  let shown_path = lock_path.display().to_string();
  let mut locking = tokio::task::spawn_blocking(move || {
    // Before the first login the cache directory may not exist yet;
    // create it as `TokenStore::save` would.
    if let Some(dir) = lock_path.parent() {
      use std::os::unix::fs::DirBuilderExt;
      std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)?;
    }
    let file = std::fs::OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(false)
      .open(&lock_path)?;
    file.lock()?;
    Ok::<_, std::io::Error>(file)
  });
  let locked =
    match tokio::time::timeout(LOCK_WAIT_NOTICE_AFTER, &mut locking).await {
      Ok(locked) => locked,
      Err(_) => {
        eprintln!(
          "Waiting for another manta command (or request) that is logging \
           in or refreshing the token (lock: {shown_path})..."
        );
        locking.await
      }
    };
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

#[cfg(test)]
mod tests {
  use super::*;

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

  fn settings(issuer_url: &str) -> OidcSettings<'_> {
    OidcSettings {
      issuer_url,
      client_id: "manta-cli",
      headless: true,
    }
  }

  fn cache_from(path: &Path, issuer_url: &str) {
    TokenStore {
      access_token: "access".to_string(),
      refresh_token: Some("refresh".to_string()),
      id_token: None,
      expires_at: None,
      issuer_url: issuer_url.to_string(),
      client_id: "manta-cli".to_string(),
    }
    .save(path)
    .unwrap();
  }

  #[test]
  fn the_configured_issuers_cache_gives_its_refresh_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    cache_from(&path, "https://prod");
    assert_eq!(
      cached_refresh_token(&path, &settings("https://prod")),
      Ok("refresh".to_string())
    );
  }

  #[test]
  fn no_cache_means_no_refresh_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    assert_eq!(
      cached_refresh_token(&path, &settings("https://prod")),
      Err("not logged in".to_string())
    );
  }

  #[test]
  fn a_corrupt_cache_means_no_refresh_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    std::fs::write(&path, "not json").unwrap();
    let why =
      cached_refresh_token(&path, &settings("https://prod")).unwrap_err();
    assert!(why.starts_with("the token cache is unusable"), "got: {why}");
  }

  /// Switching `oidc_issuer_url` (prod ↔ TDS) must not send prod's refresh
  /// token to TDS.
  #[test]
  fn another_issuers_cache_gives_no_refresh_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(TOKEN_CACHE_FILE);
    cache_from(&path, "https://prod");
    let why =
      cached_refresh_token(&path, &settings("https://tds")).unwrap_err();
    assert!(why.contains("https://prod"), "got: {why}");
    assert!(why.contains("https://tds"), "got: {why}");
  }
}
