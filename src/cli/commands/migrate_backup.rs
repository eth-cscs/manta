use anyhow::Error;
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  bos: Option<&str>,
  destination: Option<&str>,
  prehook: Option<&str>,
  posthook: Option<&str>,
) -> Result<(), Error> {
  println!(
    "Migrate backup \n BOS Template: {}\n Destination folder: {}\n Pre-hook: {}\n Post-hook: {}\n",
    bos.unwrap(),
    destination.unwrap(),
    &prehook.unwrap_or(&"none".to_string()),
    &posthook.unwrap_or(&"none".to_string()),
  );
  if prehook.is_some() {
    match crate::common::hooks::check_hook_perms(prehook).await {
      Ok(_r) => log::debug!("Pre-hook script exists and is executable."),
      Err(e) => {
        return Err(Error::msg(format!("{}. File: {}", e, &prehook.unwrap())));
      }
    };
  }
  if posthook.is_some() {
    match crate::common::hooks::check_hook_perms(posthook).await {
      Ok(_) => log::debug!("Post-hook script exists and is executable."),
      Err(e) => {
        return Err(Error::msg(format!("{}. File: {}", e, &posthook.unwrap())));
      }
    };
  }

  println!("Running the pre-hook {}", &prehook.unwrap());
  match crate::common::hooks::run_hook(prehook).await {
    Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
    Err(_error) => {
      return Err(Error::msg(format!(
        "Pre-hook script failed. Error: {}",
        _error
      )));
    }
  };

  let migrate_backup_rslt = backend
    .migrate_backup(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos,
      destination,
    )
    .await;

  match migrate_backup_rslt {
    Ok(_) => {
      log::debug!("Migrate backup completed successfully.");
    }
    Err(e) => {
      return Err(Error::msg(format!("Migrate backup failed. Error: {}", e)));
    }
  }

  if posthook.is_some() {
    println!("Running the post-hook {}", &posthook.unwrap());
    match crate::common::hooks::run_hook(posthook).await {
      Ok(_code) => {
        log::debug!("Post-hook script completed ok. RT={}", _code);
      }
      Err(_error) => {
        return Err(Error::msg(format!(
          "Post-hook script failed. Error: {}",
          _error
        )));
      }
    };
  }

  Ok(())
}
