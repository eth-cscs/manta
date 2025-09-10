use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use dialoguer::{Input, Password};
use directories::ProjectDirs;
use manta_backend_dispatcher::{
  error::Error, interfaces::authentication::AuthenticationTrait,
};
use std::{
  fs::{create_dir_all, File},
  io::{self, IsTerminal, Read, Write},
  path::PathBuf,
};
use termion::color;

pub async fn get_api_token(
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<String, Error> {
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
        log::info!("Running in interactive mode. Falling back to next authentication method");
      }
    }
  }

  // Get authentication token from API interactively
  log::info!("Getting CSM authentication token interactively");
  let shasta_token = get_token_interactively(backend).await?;

  store_token_in_local_file(site_name, &shasta_token)?;
  return Ok(shasta_token);
}

pub async fn get_token_from_env(
  backend: &StaticBackendDispatcher,
) -> Result<String, Error> {
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

    /* match backend.validate_api_token(&shasta_token).await {
      Ok(_) => {
        log::info!("Authentication token in env var is valid");
        return Ok(shasta_token);
      }
      Err(e) => log::warn!(
        "Authentication token in env var is not valid. Reason: {:#?}",
        e
      ),
    } */
  } else {
    return Err(Error::AuthenticationTokenNotFound(
      auth_token_env_name.to_string(),
    ));
  }

  // Err(Error::Message("Authentication unsucessful".to_string()))
}

pub async fn get_token_from_local_file(
  site_name: &str,
  backend: &StaticBackendDispatcher,
) -> Result<String, Error> {
  // Look for authentication token in fielsystem
  let project_dirs = ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  );

  let mut path = PathBuf::from(project_dirs.unwrap().cache_dir());

  path.push(site_name.to_string() + "_auth"); // ~/.cache/manta/<site name>_http is the file containing the Shasta authentication

  log::info!(
    "Looking for authentication token in filesystem file '{}'",
    path.display()
  );

  let mut shasta_token = String::new();
  File::open(&path)
    .map_err(|_| {
      Error::AuthenticationTokenNotFound(path.display().to_string())
    })?
    .read_to_string(&mut shasta_token)?;

  log::info!(
    "Authentication token found in filesystem. Check if it is still valid",
  );

  backend.validate_api_token(&shasta_token).await?;

  Ok(shasta_token)
}

pub fn store_token_in_local_file(
  site_name: &str,
  shasta_token: &str,
) -> Result<(), Error> {
  // Look for authentication token in fielsystem
  log::info!("Store authentication token in filesystem file");

  let project_dirs = ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  );

  let mut path = PathBuf::from(project_dirs.unwrap().cache_dir());

  create_dir_all(&path)?;

  path.push(site_name.to_string() + "_auth"); // ~/.cache/manta/<site name>_http is the file containing the Shasta authentication

  log::info!("Cache file: {:?}", path);

  let mut file: File = File::create(&path)?;
  file.write_all(shasta_token.as_bytes())?;

  Ok(())
}

pub async fn get_token_interactively(
  backend: &StaticBackendDispatcher,
) -> Result<String, Error> {
  println!(
    "Please type your {}Keycloak credentials{}",
    color::Fg(color::Green),
    color::Fg(color::Reset)
  );

  let username: String =
    Input::new().with_prompt("username").interact_text()?;

  let password = Password::new().with_prompt("password").interact()?;

  let mut shasta_token_rslt = backend.get_api_token(&username, &password).await;

  let mut attempts = 0;

  while shasta_token_rslt.is_err() && attempts < 3 {
    let err = shasta_token_rslt.as_ref().err().unwrap();
    log::info!(
      "Authentication attempt {} failed. Reason: {}",
      attempts + 1,
      err
    );

    println!(
      "Please type your {}Keycloak credentials{}",
      color::Fg(color::Green),
      color::Fg(color::Reset)
    );
    let username: String = Input::new()
      .with_prompt("username")
      .interact_text()
      .unwrap();
    let password = Password::new().with_prompt("password").interact()?;

    shasta_token_rslt = backend.get_api_token(&username, &password).await;

    attempts += 1;
  }

  shasta_token_rslt
}
