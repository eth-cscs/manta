use manta_backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;
use std::process::exit;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  bos_file: Option<&String>,
  cfs_file: Option<&String>,
  hsm_file: Option<&String>,
  ims_file: Option<&String>,
  image_dir: Option<&String>,
  prehook: Option<&String>,
  posthook: Option<&String>,
) {
  println!(
        "Migrate_restore\n Prehook: {}\n Posthook: {}\n BOS_file: {}\n CFS_file: {}\n IMS_file: {}\n HSM_file: {}",
        &prehook.unwrap_or(&"none".to_string()),
        &posthook.unwrap_or(&"none".to_string()),
        bos_file.unwrap(),
        cfs_file.unwrap(),
        ims_file.unwrap(),
        hsm_file.unwrap()
    );
  if prehook.is_some() {
    match crate::common::hooks::check_hook_perms(prehook).await {
      Ok(_) => log::debug!("Pre-hook script exists and is executable."),
      Err(e) => {
        log::error!("{}. File: {}", e, &prehook.unwrap());
        exit(2);
      }
    };
  }
  if posthook.is_some() {
    match crate::common::hooks::check_hook_perms(posthook).await {
      Ok(_) => log::debug!("Post-hook script exists and is executable."),
      Err(e) => {
        log::error!("{}. File: {}", e, &posthook.unwrap());
        exit(2);
      }
    };
  }

  println!();
  if prehook.is_some() {
    println!("Running the pre-hook {}", &prehook.unwrap());
    match crate::common::hooks::run_hook(prehook).await {
      Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
      Err(_error) => {
        log::error!("{}", _error);
        exit(2);
      }
    };
  }

  let migrate_restore_rslt = backend
    .migrate_restore(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos_file,
      cfs_file,
      hsm_file,
      ims_file,
      image_dir,
    )
    .await;

  if migrate_restore_rslt.is_err() {
    eprintln!("Error: {}", migrate_restore_rslt.err().unwrap());
    exit(2);
  }

  if posthook.is_some() {
    println!("Running the post-hook {}", &posthook.unwrap());
    match crate::common::hooks::run_hook(posthook).await {
      Ok(_code) => log::debug!("Post-hook script completed ok. RT={}", _code),
      Err(_error) => {
        log::error!("{}", _error);
        exit(2);
      }
    };
  }
  println!("\nDone, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored.");

  // ========================================================================================================
}
