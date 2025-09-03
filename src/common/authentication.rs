use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use dialoguer::{Input, Password};
use directories::ProjectDirs;
use manta_backend_dispatcher::{
  error::Error, interfaces::authentication::AuthenticationTrait,
};
use std::{
  fs::{create_dir_all, File},
  io::{Read, Write},
  path::PathBuf,
};
use termion::color;

pub async fn get_api_token(
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<String, Error> {
  let shasta_token_rslt = get_token_from_env(backend).await;

  if shasta_token_rslt.is_ok() {
    log::info!("Authentication token found in env var");
    return shasta_token_rslt;
  }

  log::info!("Authentication token not found in env var");

  let shasta_token_rslt = get_token_from_local_file(site_name, backend).await;

  if shasta_token_rslt.is_ok() {
    log::info!("Authentication token found in filesystem");
    return shasta_token_rslt;
  }

  log::info!("Authentication token not found in filesystem");

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
      "Authentication token found in env var 'MANTA_CSM_TOKEN'. Check if it is still valid"
    );
    if backend.validate_api_token(&shasta_token).await.is_ok() {
      return Ok(shasta_token);
    }
  }

  return Err(Error::Message("Authentication unsucessful".to_string()));
}

pub async fn get_token_from_local_file(
  site_name: &str,
  backend: &StaticBackendDispatcher,
) -> Result<String, Error> {
  // Look for authentication token in fielsystem
  log::info!("Looking for authentication token in filesystem file");

  let project_dirs = ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  );

  let mut path = PathBuf::from(project_dirs.unwrap().cache_dir());

  create_dir_all(&path)?;

  path.push(site_name.to_string() + "_auth"); // ~/.cache/manta/<site name>_http is the file containing the Shasta authentication

  log::info!("Cache file: {:?}", path);

  let mut shasta_token = String::new();
  File::open(path)?.read_to_string(&mut shasta_token)?;

  if backend.validate_api_token(&shasta_token).await.is_ok() {
    return Ok(shasta_token);
  }

  return Err(Error::Message("Authentication unsucessful".to_string()));
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
