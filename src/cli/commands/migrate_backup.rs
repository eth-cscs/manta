use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;

use crate::common::{app_context::AppContext, authentication::get_api_token};

/// Back up cluster configuration to a local bundle.
pub async fn exec(
  ctx: &AppContext<'_>,
  bos: Option<&str>,
  destination: Option<&str>,
  prehook: Option<&str>,
  posthook: Option<&str>,
) -> Result<(), Error> {
  let backend = ctx.infra.backend;
  let site_name = ctx.infra.site_name;
  let shasta_base_url = ctx.infra.shasta_base_url;
  let shasta_root_cert = ctx.infra.shasta_root_cert;

  let shasta_token = get_api_token(backend, site_name).await?;

  let bos_value = bos.context("BOS template is required")?;
  let destination_value =
    destination.context("Destination folder is required")?;

  println!(
    "Migrate backup \n BOS Template: {}\n Destination folder: {}\n Pre-hook: {}\n Post-hook: {}\n",
    bos_value,
    destination_value,
    prehook.unwrap_or("none"),
    posthook.unwrap_or("none"),
  );
  if let Some(prehook_path) = prehook {
    match crate::common::hooks::check_hook_perms(Some(prehook_path)) {
      Ok(_r) => {
        log::debug!("Pre-hook script exists and is executable.")
      }
      Err(e) => {
        bail!("{}. File: {}", e, prehook_path);
      }
    };
  }
  if let Some(posthook_path) = posthook {
    match crate::common::hooks::check_hook_perms(Some(posthook_path)) {
      Ok(_) => {
        log::debug!("Post-hook script exists and is executable.")
      }
      Err(e) => {
        bail!("{}. File: {}", e, posthook_path);
      }
    };
  }

  if let Some(prehook_path) = prehook {
    println!("Running the pre-hook {}", prehook_path);
    match crate::common::hooks::run_hook(Some(prehook_path)) {
      Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
      Err(_error) => {
        bail!("Pre-hook script failed. Error: {}", _error);
      }
    };
  }

  let migrate_backup_rslt = backend
    .migrate_backup(
      &shasta_token,
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
      bail!("Migrate backup failed. Error: {}", e);
    }
  }

  if let Some(posthook_path) = posthook {
    println!("Running the post-hook {}", posthook_path);
    match crate::common::hooks::run_hook(posthook) {
      Ok(_code) => {
        log::debug!("Post-hook script completed ok. RT={}", _code);
      }
      Err(_error) => {
        bail!("Post-hook script failed. Error: {}", _error);
      }
    };
  }

  Ok(())
}
