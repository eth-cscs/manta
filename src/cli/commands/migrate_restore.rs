use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::service::migrate;

/// Restore cluster configuration from a backup bundle.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
    ctx: &AppContext<'_>,
    token: &str,
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    prehook: Option<&str>,
    posthook: Option<&str>,
    overwrite: bool,
) -> Result<(), Error> {
    let bos_file_value = bos_file.context("BOS file is required")?;
    let cfs_file_value = cfs_file.context("CFS file is required")?;
    let ims_file_value = ims_file.context("IMS file is required")?;
    let hsm_file_value = hsm_file.context("HSM file is required")?;

    println!(
        "Migrate_restore\n Prehook: {}\n Posthook: {}\n BOS_file: {}\n CFS_file: {}\n IMS_file: {}\n HSM_file: {}",
        prehook.unwrap_or("none"),
        posthook.unwrap_or("none"),
        bos_file_value,
        cfs_file_value,
        ims_file_value,
        hsm_file_value
    );

    if let Some(prehook_path) = prehook {
        match crate::common::hooks::check_hook_perms(Some(prehook_path)) {
            Ok(_) => {
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

    println!();
    if let Some(prehook_path) = prehook {
        println!("Running the pre-hook {}", prehook_path);
        match crate::common::hooks::run_hook(prehook) {
            Ok(_code) => {
                log::debug!("Pre-hook script completed ok. RT={}", _code)
            }
            Err(_error) => {
                bail!("Pre-hook script failed. Error: {}", _error);
            }
        };
    }

    migrate::migrate_restore(
        &ctx.infra,
        token,
        bos_file,
        cfs_file,
        hsm_file,
        ims_file,
        image_dir,
        overwrite,
    )
    .await?;

    if let Some(posthook_path) = posthook {
        println!("Running the post-hook {}", posthook_path);
        match crate::common::hooks::run_hook(posthook) {
            Ok(_code) => {
                log::debug!("Post-hook script completed ok. RT={}", _code)
            }
            Err(_error) => {
                bail!("Post-hook script failed. Error: {}", _error);
            }
        };
    }

    println!(
        "\nDone, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored."
    );

    Ok(())
}
