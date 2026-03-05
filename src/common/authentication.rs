use crate::{
  common::config::get_default_cache_path,
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use anyhow::Context;
use crossterm::style::Stylize;
use dialoguer::{Input, Password};
use manta_backend_dispatcher::interfaces::authentication::AuthenticationTrait;
use std::{
  fs::{File, create_dir_all},
  io::{self, IsTerminal, Read, Write},
  os::unix::fs::OpenOptionsExt,
};

/// Obtain a valid API token, trying in order: env var
/// `MANTA_CSM_TOKEN`, cached file, interactive login.
pub async fn get_api_token(
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<String, anyhow::Error> {
  let auth_token_rslt = get_token_from_env(backend).await;

  match auth_token_rslt {
    Ok(token) => {
      log::info!("Authentication successful using env var");
      return Ok(token);
    }
    Err(err) => {
      log::warn!(
        "{:#?}. Falling back to next authentication method",
        err.to_string()
      );
    }
  }

  let auth_token_rslt = get_token_from_local_file(site_name, backend).await;

  match auth_token_rslt {
    Ok(token) => {
      log::info!("Authentication successful using local file");
      return Ok(token);
    }
    Err(err) => {
      log::warn!("{:#?}", err.to_string());
      // Stop execution if not running in a terminal or fallback to next method
      let stdin = io::stdin();
      if !stdin.is_terminal() {
        log::info!(
          "Running in non-interactive method. Give up authentication."
        );
        return Err(err);
      } else {
        log::info!(
          "Running in interactive mode. Falling back to next authentication method"
        );
      }
    }
  }

  // Get authentication token from API interactively
  log::info!("Getting CSM authentication token interactively");
  let shasta_token = get_token_interactively(backend).await?;

  store_token_in_local_file(site_name, &shasta_token)?;
  Ok(shasta_token)
}

async fn get_token_from_env(
  backend: &StaticBackendDispatcher,
) -> Result<String, anyhow::Error> {
  let auth_token_env_name = "MANTA_CSM_TOKEN";

  // Look for authentication token in env vars
  log::info!(
    "Looking for authentication token in env var '{}'",
    auth_token_env_name
  );

  let shasta_token_rslt = std::env::var(auth_token_env_name);

  if let Ok(shasta_token) = shasta_token_rslt {
    log::info!(
      "Authentication token found in env var '{}'. Check if it is valid",
      auth_token_env_name
    );

    backend.validate_api_token(&shasta_token).await?;

    Ok(shasta_token)
  } else {
    Err(anyhow::anyhow!(
      "Authentication token not found in env var '{}'",
      auth_token_env_name
    ))
  }
}

async fn get_token_from_local_file(
  site_name: &str,
  backend: &StaticBackendDispatcher,
) -> Result<String, anyhow::Error> {
  // Look for authentication token in filesystem
  let mut path = get_default_cache_path()?;

  path.push(site_name.to_string() + "_auth"); // ~/.cache/manta/<site name>_http is the file containing the Shasta authentication

  log::info!(
    "Looking for authentication token in filesystem file '{}'",
    path.display()
  );

  let mut shasta_token = String::new();
  File::open(&path)
    .inspect_err(|e| {
      log::debug!("Could not open token file '{}': {}", path.display(), e);
    })
    .with_context(|| {
      format!("Authentication token not found in '{}'", path.display())
    })?
    .read_to_string(&mut shasta_token)?;

  log::info!(
    "Authentication token found in filesystem. Check if it is still valid",
  );

  backend.validate_api_token(&shasta_token).await?;

  Ok(shasta_token)
}

fn store_token_in_local_file(
  site_name: &str,
  shasta_token: &str,
) -> Result<(), anyhow::Error> {
  // Store authentication token in filesystem
  log::info!("Store authentication token in filesystem file");

  let mut path = get_default_cache_path()?;

  create_dir_all(&path)?;

  path.push(site_name.to_string() + "_auth"); // ~/.cache/manta/<site name>_http is the file containing the Shasta authentication

  log::info!("Cache file: {:?}", path);

  let mut file: File = File::options()
    .write(true)
    .create(true)
    .truncate(true)
    .mode(0o600)
    .open(&path)?;
  file.write_all(shasta_token.as_bytes())?;

  Ok(())
}

async fn get_token_interactively(
  backend: &StaticBackendDispatcher,
) -> Result<String, anyhow::Error> {
  println!("Please type your {}", "Keycloak credentials".green());

  let username: String = Input::new()
    .with_prompt("username")
    .interact_text()
    .context("Failed to read username")?;

  let password = Password::new()
    .with_prompt("password")
    .interact()
    .context("Failed to read password")?;

  let mut shasta_token_rslt = backend.get_api_token(&username, &password).await;

  let mut attempts = 0;

  while shasta_token_rslt.is_err() && attempts < 3 {
    if let Err(ref err) = shasta_token_rslt {
      log::info!(
        "Authentication attempt {} failed. Reason: {}",
        attempts + 1,
        err
      );
    }

    println!("Please type your {}", "Keycloak credentials".green());
    let username: String = Input::new()
      .with_prompt("username")
      .interact_text()
      .context("Failed to read username")?;
    let password = Password::new()
      .with_prompt("password")
      .interact()
      .context("Failed to read password")?;

    shasta_token_rslt = backend.get_api_token(&username, &password).await;

    attempts += 1;
  }

  Ok(shasta_token_rslt?)
}
