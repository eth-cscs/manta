//! Token acquisition: env var -> cached token -> interactive login.
//!
//! The bearer token is issued by the CSCS Keycloak and obtained by the
//! CLI directly (see [`crate::common::oidc`]); `manta-server` takes no
//! part in authentication. One token serves every site, so nothing
//! here depends on the selected site.
//!
//! ## Resolution order
//!
//! [`get_api_token`] tries each source in turn and returns the first
//! token it gets:
//!
//! 1. `MANTA_TOKEN` environment variable, used as-is (for scripts).
//! 2. Cached token at `<cache_dir>/token.json` (`0600`). The cache
//!    directory comes from
//!    [`manta_shared::common::config::get_default_cache_path`]. An
//!    expired access token is refreshed with the cached refresh token
//!    and written back.
//! 3. Interactive OIDC login ([`oidc::login`]: browser, or device code
//!    when headless). The result is written to the cache.
//!
//! Tokens are not checked against the server up front. A token the
//! site rejects surfaces as a `401` on the first real request (see
//! [`crate::http_client::OpenApiResultExt`]).
//!
//! Non-interactive callers (`stdin` is not a TTY) stop after the cache
//! instead of starting a login nobody can complete.

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use manta_shared::common::config::get_default_cache_path;
use manta_shared::common::jwt_ops;

use crate::common::app_context::AppContext;
use crate::common::oidc::{self, OidcSettings, TokenStore};

/// Environment variable holding a ready-made bearer token.
pub const AUTH_TOKEN_ENV_VAR: &str = "MANTA_TOKEN";

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

/// Obtain a bearer token, trying in order: env var `MANTA_TOKEN`,
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

  if let Some(token) = token_from_env(std::env::var(AUTH_TOKEN_ENV_VAR).ok()) {
    tracing::info!("Using authentication token from env var");
    return Ok(token);
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
       {AUTH_TOKEN_ENV_VAR}."
    );
  }

  tracing::info!(issuer = %ctx.oidc.issuer_url, "Logging in interactively");
  let store = oidc::login(&ctx.oidc).await?;
  store.save(&cache_path)?;
  tracing::info!(path = %cache_path.display(), "Authentication token cached on disk");
  Ok(store.access_token)
}

/// Source 1: the token in `MANTA_TOKEN` (passed in as `value`). Used
/// as-is; an expired one only draws a warning, since the server will
/// reject it with a clear error anyway.
fn token_from_env(value: Option<String>) -> Option<String> {
  let token = value.filter(|t| !t.trim().is_empty())?;
  if let Ok(Some(exp)) = jwt_ops::get_expiration(&token)
    && exp <= oidc::now_secs()
  {
    tracing::warn!("the token in {AUTH_TOKEN_ENV_VAR} has expired");
  }
  Some(token)
}

/// Source 2: the cached token at `path`. Returns the access token when
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
  match oidc::refresh(settings, refresh_token).await {
    Ok(refreshed) => {
      refreshed.save(path)?;
      tracing::info!("Access token refreshed");
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
  fn auth_token_env_var_name() {
    assert_eq!(AUTH_TOKEN_ENV_VAR, "MANTA_TOKEN");
  }

  #[test]
  fn env_token_is_used_when_set() {
    assert_eq!(
      token_from_env(Some("abc".to_string())).as_deref(),
      Some("abc")
    );
  }

  #[test]
  fn env_token_absent_or_blank_is_skipped() {
    assert!(token_from_env(None).is_none());
    assert!(token_from_env(Some("  ".to_string())).is_none());
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
