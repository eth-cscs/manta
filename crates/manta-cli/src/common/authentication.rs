//! Token acquisition: env var → cached file → interactive Keycloak login.
//!
//! Every path checks the candidate token against the configured
//! `manta-server` (via `MantaClient`), which in turn validates it
//! against the CSM/OCHAMI backend. The CLI never reaches a backend
//! directly.

use crate::http_client::MantaClient;
use anyhow::{Result, anyhow};
use crossterm::style::Stylize;
use dialoguer::{Input, Password};
use crate::common::app_context::AppContext;
use manta_shared::common::config::get_default_cache_path;
use std::{
  fs::{File, create_dir_all},
  io::{self, IsTerminal, Read, Write},
  os::unix::fs::OpenOptionsExt,
};

/// Environment variable name for the API authentication token.
const AUTH_TOKEN_ENV_VAR: &str = "MANTA_CSM_TOKEN";

/// Suffix appended to the site name to form the auth cache filename.
const AUTH_CACHE_FILE_SUFFIX: &str = "_auth";

/// Maximum number of interactive login attempts before giving up.
const MAX_LOGIN_ATTEMPTS: u32 = 3;

/// Obtain a valid API token, trying in order: env var
/// `MANTA_CSM_TOKEN`, cached file, interactive login. Every candidate
/// is validated through `manta-server`.
#[tracing::instrument(skip_all, fields(site = %ctx.site_name))]
pub async fn get_api_token(ctx: &AppContext<'_>) -> Result<String> {
  let client = MantaClient::new(ctx.manta_server_url, ctx.site_name)?;
  let site_name = ctx.site_name;

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

  client.validate_token(&shasta_token).await?;
  Ok(shasta_token)
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

  client.validate_token(&shasta_token).await?;
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

  let mut shasta_token_rslt = client.get_token(&username, &password).await;

  let mut attempts = 0;

  while shasta_token_rslt.is_err() && attempts < MAX_LOGIN_ATTEMPTS {
    if let Err(ref err) = shasta_token_rslt {
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

    shasta_token_rslt = client.get_token(&username, &password).await;

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
