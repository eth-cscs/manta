//! Implements the `manta restore vcluster` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::MigrateRestoreRequest;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub bos_file: Option<&'a str>,
  pub cfs_file: Option<&'a str>,
  pub hsm_file: Option<&'a str>,
  pub ims_file: Option<&'a str>,
  pub image_dir: Option<&'a str>,
  pub prehook: Option<&'a str>,
  pub posthook: Option<&'a str>,
  pub overwrite: bool,
  pub output: Option<&'a str>,
}

/// Restore cluster configuration from a backup bundle.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let bos_file = p.bos_file;
  let cfs_file = p.cfs_file;
  let hsm_file = p.hsm_file;
  let ims_file = p.ims_file;
  let image_dir = p.image_dir;
  let prehook = p.prehook;
  let posthook = p.posthook;
  let overwrite = p.overwrite;
  let output_opt = p.output;
  let bos_file_value = bos_file.context("BOS file is required")?;
  let cfs_file_value = cfs_file.context("CFS file is required")?;
  let ims_file_value = ims_file.context("IMS file is required")?;
  let hsm_file_value = hsm_file.context("HSM file is required")?;

  action_result::print(
    &format!(
      "Migrate_restore\n Prehook: {}\n Posthook: {}\n BOS_file: {}\n CFS_file: {}\n IMS_file: {}\n Group_file: {}",
      prehook.unwrap_or("none"),
      posthook.unwrap_or("none"),
      bos_file_value,
      cfs_file_value,
      ims_file_value,
      hsm_file_value
    ),
    output_opt,
  )?;

  if let Some(prehook_path) = prehook {
    match crate::common::hooks::check_hook_perms(Some(prehook_path)) {
      Ok(()) => {
        tracing::debug!("Pre-hook script exists and is executable.");
      }
      Err(e) => {
        bail!("{e}. File: {prehook_path}");
      }
    }
  }
  if let Some(posthook_path) = posthook {
    match crate::common::hooks::check_hook_perms(Some(posthook_path)) {
      Ok(()) => {
        tracing::debug!("Post-hook script exists and is executable.");
      }
      Err(e) => {
        bail!("{e}. File: {posthook_path}");
      }
    }
  }

  println!();
  crate::common::hooks::run_hook_if_present(prehook, "pre")?;

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .migrate_restore(
      client.site_name(),
      &MigrateRestoreRequest {
        bos_file: bos_file.map(str::to_string),
        cfs_file: cfs_file.map(str::to_string),
        hsm_file: hsm_file.map(str::to_string),
        ims_file: ims_file.map(str::to_string),
        image_dir: image_dir.map(str::to_string),
        overwrite: Some(overwrite),
      },
    )
    .await
    .into_anyhow()?;

  crate::common::hooks::run_hook_if_present(posthook, "post")?;

  action_result::print(
    "Done, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored.",
    output_opt,
  )?;

  Ok(())
}
