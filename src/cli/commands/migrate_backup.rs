use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::service::migrate;

/// Back up cluster configuration to a local bundle.
pub async fn exec(
    ctx: &AppContext<'_>,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
    prehook: Option<&str>,
    posthook: Option<&str>,
) -> Result<(), Error> {
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
                tracing::debug!("Pre-hook script exists and is executable.")
            }
            Err(e) => {
                bail!("{}. File: {}", e, prehook_path);
            }
        };
    }
    if let Some(posthook_path) = posthook {
        match crate::common::hooks::check_hook_perms(Some(posthook_path)) {
            Ok(_) => {
                tracing::debug!("Post-hook script exists and is executable.")
            }
            Err(e) => {
                bail!("{}. File: {}", e, posthook_path);
            }
        };
    }

    if let Some(prehook_path) = prehook {
        println!("Running the pre-hook {}", prehook_path);
        match crate::common::hooks::run_hook(Some(prehook_path)) {
            Ok(_code) => {
                tracing::debug!("Pre-hook script completed ok. RT={}", _code)
            }
            Err(_error) => {
                bail!("Pre-hook script failed. Error: {}", _error);
            }
        };
    }

    migrate::migrate_backup(&ctx.infra, token, bos, destination).await?;
    tracing::debug!("Migrate backup completed successfully.");

    if let Some(posthook_path) = posthook {
        println!("Running the post-hook {}", posthook_path);
        match crate::common::hooks::run_hook(posthook) {
            Ok(_code) => {
                tracing::debug!("Post-hook script completed ok. RT={}", _code);
            }
            Err(_error) => {
                bail!("Post-hook script failed. Error: {}", _error);
            }
        };
    }

    Ok(())
}
