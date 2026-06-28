//! Token acquisition: env var -> cached file -> interactive Keycloak login.
//!
//! Every path checks the candidate token against the configured
//! `manta-server` (via `MantaClient`), which in turn validates it
//! against the CSM/OCHAMI backend. The CLI never reaches a backend
//! directly.
//!
//! ## Resolution order
//!
//! [`get_api_token`] walks the candidates in this order and returns
//! the first one that the server's `/api/v1/auth/validate` accepts:
//!
//! 1. `MANTA_CSM_TOKEN` environment variable.
//! 2. Cached file at `<cache_dir>/<site>_auth` (0600-permissions). The
//!    cache directory comes from
//!    [`manta_shared::common::config::get_default_cache_path`].
//! 3. Interactive Keycloak username + password prompt
//!    (`dialoguer`-based), retried up to [`MAX_LOGIN_ATTEMPTS`] times
//!    against `/api/v1/auth/token`. A successful interactive login is
//!    written back to the cache file.
//!
//! ## Short-circuit on unreachable server
//!
//! If any step fails because the manta server itself is unreachable
//! (DNS / TCP / TLS) — surfaced by the
//! [`crate::http_client::AuthServerUnreachable`] typed context — the
//! cascade aborts immediately. Trying the next path would hit the
//! same dead endpoint, and re-prompting would only confuse the
//! operator.
//!
//! Non-interactive callers (`stdin` is not a TTY) also stop after the
//! cached-token attempt rather than blocking on a prompt that can
//! never be answered.

use crate::common::app_context::AppContext;
use crate::http_client::{AuthServerUnreachable, MantaClient};
use crate::openapi_client::types::{AuthTokenRequest, ValidateTokenRequest};
use anyhow::{Result, anyhow};
use crossterm::style::Stylize;
use dialoguer::{Input, Password};
use manta_shared::common::config::get_default_cache_path;
use std::{
  fs::{File, create_dir_all},
  io::{self, IsTerminal, Read, Write},
  os::unix::fs::OpenOptionsExt,
};

/// `true` if `err`'s typed-context chain contains
/// [`AuthServerUnreachable`] — the marker we attach in
/// [`map_auth_error`] whenever the auth-bearing send fails at the
/// TCP / timeout layer. Used by every stage of `get_api_token`
/// (env var, cached file, interactive prompt) to bail out instead
/// of falling through or re-prompting.
///
/// Uses `downcast_ref` rather than `chain().any(is::<...>())`
/// because anyhow stores contexts behind an internal wrapper type;
/// `downcast_ref` knows how to look through it, but the chain
/// iterator yields the wrapper's concrete type.
fn is_auth_server_unreachable(err: &anyhow::Error) -> bool {
  err.downcast_ref::<AuthServerUnreachable>().is_some()
}

/// Environment variable name for the API authentication token.
const AUTH_TOKEN_ENV_VAR: &str = "MANTA_CSM_TOKEN";

/// Suffix appended to the site name to form the auth cache filename.
const AUTH_CACHE_FILE_SUFFIX: &str = "_auth";

/// Maximum number of interactive login attempts before giving up.
const MAX_LOGIN_ATTEMPTS: u32 = 3;

/// Wrap a progenitor `Error<E>` from an `/auth/*` call into an
/// `anyhow::Error`, attaching the typed [`AuthServerUnreachable`]
/// marker whenever the failure was at the TCP / timeout layer.
fn map_auth_error<E: std::fmt::Debug>(
  err: progenitor_client::Error<E>,
  url: &str,
) -> anyhow::Error
where
  progenitor_client::Error<E>: std::fmt::Display,
{
  let unreachable = match &err {
    progenitor_client::Error::CommunicationError(e) => {
      e.is_connect() || e.is_timeout()
    }
    _ => false,
  };
  let message = format!("{err}");
  if unreachable {
    anyhow!(message).context(AuthServerUnreachable {
      url: url.to_string(),
    })
  } else {
    anyhow!(message)
  }
}

/// Obtain a valid API token, trying in order: env var
/// `MANTA_CSM_TOKEN`, cached file, interactive login. Every candidate
/// is validated through `manta-server`.
///
/// On a successful interactive login the token is written back to
/// `<cache_dir>/<site>_auth` with `0600` permissions so subsequent
/// invocations re-use it.
///
/// # Errors
///
/// - No site is set (`ctx.require_site()` fails).
/// - The manta server is unreachable at any point — the cascade
///   aborts and surfaces an
///   [`crate::http_client::AuthServerUnreachable`]-wrapped error.
/// - All three candidates failed (no env var, no cached file or
///   stale cached token, and either the interactive retries hit
///   [`MAX_LOGIN_ATTEMPTS`] or stdin isn't a terminal).
/// - File I/O for the cache write fails after a successful login.
#[tracing::instrument(skip_all, fields(site = ctx.site_name.unwrap_or("<unset>")))]
pub async fn get_api_token(ctx: &AppContext<'_>) -> Result<String> {
  // Auth endpoints are the ones that *obtain* or *check* the token,
  // so we pass `None` as the bearer here; no default `Authorization`
  // header gets attached.
  let site_name = ctx.require_site()?;
  let client = MantaClient::new(ctx.manta_server_url, site_name)?;

  tracing::info!(
    server = %ctx.manta_server_url,
    "Beginning authentication"
  );

  match get_token_from_env(&client).await {
    Ok(token) => {
      tracing::info!("Authentication successful using env var");
      return Ok(token);
    }
    Err(err) => {
      // Short-circuit when the server is unreachable: validating the
      // file token or prompting interactively would just hit the
      // same dead endpoint. The user needs to fix the server or
      // their config before any auth path can succeed.
      if is_auth_server_unreachable(&err) {
        return Err(err);
      }
      tracing::warn!(
        error = %err,
        "env-var auth failed, trying cached token file"
      );
    }
  }

  match get_token_from_local_file(site_name, &client).await {
    Ok(token) => {
      tracing::info!("Authentication successful using local file");
      return Ok(token);
    }
    Err(err) => {
      if is_auth_server_unreachable(&err) {
        return Err(err);
      }
      let stdin = io::stdin();
      if !stdin.is_terminal() {
        tracing::warn!(
          error = %err,
          "cached token rejected and stdin is not a terminal; giving up"
        );
        return Err(err);
      }
      tracing::warn!(
        error = %err,
        "cached token rejected, prompting for credentials interactively"
      );
    }
  }

  tracing::info!("Getting CSM authentication token interactively");
  let shasta_token = get_token_interactively(&client).await?;

  store_token_in_local_file(site_name, &shasta_token)?;
  tracing::info!("Authentication successful using interactive login");
  Ok(shasta_token)
}

async fn get_token_from_env(client: &MantaClient) -> Result<String> {
  let auth_token_env_name = AUTH_TOKEN_ENV_VAR;

  tracing::info!(
    "Looking for authentication token in env var '{}'",
    auth_token_env_name
  );

  let shasta_token = std::env::var(auth_token_env_name).map_err(|_| {
    anyhow!("authentication token not found in env var '{auth_token_env_name}'")
  })?;

  tracing::info!(
    "Authentication token found in env var '{}'. Check if it is valid",
    auth_token_env_name
  );

  validate_token(client, &shasta_token).await?;
  Ok(shasta_token)
}

/// `POST /api/v1/auth/validate` — check whether the backend still
/// accepts `token`. Wraps the progenitor call so callers see a
/// clean `anyhow::Result<()>` plus the `AuthServerUnreachable`
/// marker on connect-level failures.
async fn validate_token(
  client: &MantaClient,
  token: &str,
) -> anyhow::Result<()> {
  let url = client.base_url().trim_end_matches("/api/v1").to_string();
  client
    .openapi
    .auth_validate(
      client.site_name(),
      &ValidateTokenRequest {
        token: token.to_owned(),
      },
    )
    .await
    .map(|_| ())
    .map_err(|e| map_auth_error(e, &url))
}

/// `POST /api/v1/auth/token` — exchange Keycloak credentials for a CSM
/// bearer token.
async fn get_token(
  client: &MantaClient,
  username: &str,
  password: &str,
) -> anyhow::Result<String> {
  let url = client.base_url().trim_end_matches("/api/v1").to_string();
  let resp = client
    .openapi
    .auth_token(
      client.site_name(),
      &AuthTokenRequest {
        username: username.to_owned(),
        password: password.to_owned(),
      },
    )
    .await
    .map_err(|e| map_auth_error(e, &url))?;
  Ok(resp.into_inner().token)
}

async fn get_token_from_local_file(
  site_name: &str,
  client: &MantaClient,
) -> Result<String> {
  let mut path = get_default_cache_path()?;

  path.push(site_name.to_string() + AUTH_CACHE_FILE_SUFFIX);

  tracing::info!(
    "Looking for authentication token in filesystem file '{}'",
    path.display()
  );

  let mut shasta_token = String::new();
  File::open(&path)
    .inspect_err(|e| {
      tracing::debug!("Could not open token file '{}': {}", path.display(), e);
    })
    .map_err(|_| {
      anyhow!("authentication token not found at '{}'", path.display())
    })?
    .read_to_string(&mut shasta_token)?;

  tracing::info!(
    "Authentication token found in filesystem. Check if it is still valid",
  );

  validate_token(client, &shasta_token).await?;
  Ok(shasta_token)
}

fn store_token_in_local_file(
  site_name: &str,
  shasta_token: &str,
) -> Result<()> {
  tracing::info!("Store authentication token in filesystem file");

  let mut path = get_default_cache_path()?;

  create_dir_all(&path)?;

  path.push(site_name.to_string() + AUTH_CACHE_FILE_SUFFIX);

  tracing::info!("Cache file: {:?}", path);

  let mut file: File = File::options()
    .write(true)
    .create(true)
    .truncate(true)
    .mode(0o600)
    .open(&path)?;
  file.write_all(shasta_token.as_bytes())?;

  tracing::info!(path = %path.display(), "Authentication token cached on disk");
  Ok(())
}

async fn get_token_interactively(client: &MantaClient) -> Result<String> {
  println!("Please type your {}", "Keycloak credentials".green());

  let username: String =
    Input::new().with_prompt("username").interact_text()?;

  let password = Password::new().with_prompt("password").interact()?;

  let mut shasta_token_rslt = get_token(client, &username, &password).await;

  let mut attempts = 0;

  while shasta_token_rslt.is_err() && attempts < MAX_LOGIN_ATTEMPTS {
    if let Err(ref err) = shasta_token_rslt {
      // If the failure is "server unreachable" rather than "wrong
      // credentials", re-prompting is pointless — the next attempt
      // would hit the same dead endpoint. Bail out and let the
      // operator see the meaningful message immediately.
      if is_auth_server_unreachable(err) {
        tracing::warn!(
          error = %err,
          "auth server unreachable; aborting interactive retries"
        );
        return shasta_token_rslt;
      }
      tracing::warn!(
        attempt = attempts + 1,
        max_attempts = MAX_LOGIN_ATTEMPTS,
        error = %err,
        "Interactive authentication attempt failed"
      );
    }

    println!("Please type your {}", "Keycloak credentials".green());
    let username: String =
      Input::new().with_prompt("username").interact_text()?;
    let password = Password::new().with_prompt("password").interact()?;

    shasta_token_rslt = get_token(client, &username, &password).await;

    attempts += 1;
  }

  if shasta_token_rslt.is_ok() && attempts > 0 {
    tracing::info!(
      attempts = attempts + 1,
      "Interactive authentication succeeded after retries"
    );
  }

  shasta_token_rslt
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;

  #[test]
  fn store_and_read_token_from_local_file() {
    let tmp_dir = tempfile::tempdir().unwrap();

    let site_name = "test_site";
    let token = "my-secret-token-12345";

    let mut path = tmp_dir.path().to_path_buf();
    path.push(format!("{site_name}{AUTH_CACHE_FILE_SUFFIX}"));

    let mut file = File::options()
      .write(true)
      .create(true)
      .truncate(true)
      .mode(0o600)
      .open(&path)
      .unwrap();
    file.write_all(token.as_bytes()).unwrap();

    let mut content = String::new();
    File::open(&path)
      .unwrap()
      .read_to_string(&mut content)
      .unwrap();
    assert_eq!(content, token);

    let metadata = std::fs::metadata(&path).unwrap();
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "Token file should have 600 permissions");
  }

  #[test]
  fn store_token_overwrites_existing() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let mut path = tmp_dir.path().to_path_buf();
    path.push("overwrite_test_auth");

    let mut file = File::options()
      .write(true)
      .create(true)
      .truncate(true)
      .mode(0o600)
      .open(&path)
      .unwrap();
    file.write_all(b"old-token").unwrap();

    let mut file = File::options()
      .write(true)
      .create(true)
      .truncate(true)
      .mode(0o600)
      .open(&path)
      .unwrap();
    file.write_all(b"new-token").unwrap();

    let mut content = String::new();
    File::open(&path)
      .unwrap()
      .read_to_string(&mut content)
      .unwrap();
    assert_eq!(content, "new-token");
  }

  #[test]
  fn auth_token_env_var_name() {
    assert_eq!(AUTH_TOKEN_ENV_VAR, "MANTA_CSM_TOKEN");
  }

  #[test]
  fn auth_cache_file_suffix_value() {
    assert_eq!(AUTH_CACHE_FILE_SUFFIX, "_auth");
  }

  #[test]
  fn max_login_attempts_is_reasonable() {
    const { assert!(MAX_LOGIN_ATTEMPTS >= 1 && MAX_LOGIN_ATTEMPTS <= 10) };
  }
}
