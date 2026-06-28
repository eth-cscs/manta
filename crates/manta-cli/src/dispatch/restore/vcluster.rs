//! Implements the `manta restore vcluster` command.
//!
//! Drives `POST /api/v1/migrate/restore` to reload BOS / CFS / HSM /
//! IMS state from a previously-captured bundle. All four
//! `--*-file` arguments are required (an IMS-only or BOS-only restore
//! isn't supported by the server-side flow). Runs an optional
//! pre-hook before the POST and a post-hook after; hook perms are
//! validated up-front. `--dry-run` short-circuits client-side via
//! [`crate::output::action_result::preview_request`] (the endpoint
//! has no `dry_run` flag). Inverse of
//! [`super::super::backup::vcluster`].

use anyhow::{Context, Error};

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
  pub dry_run: bool,
}

/// Restore cluster configuration from a backup bundle.
///
/// # Errors
///
/// Returns an error when any of `--bos-file`, `--cfs-file`,
/// `--hsm-file`, `--ims-file` is missing, when a hook script fails its
/// perms check or its execution, when the HTTP client cannot be built,
/// or when the `migrate_restore` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let ExecParams {
    bos_file,
    cfs_file,
    hsm_file,
    ims_file,
    image_dir,
    prehook,
    posthook,
    overwrite,
    output: output_opt,
    dry_run,
  } = p;
  let bos_file_value = bos_file.context("BOS file is required")?;
  let cfs_file_value = cfs_file.context("CFS file is required")?;
  let ims_file_value = ims_file.context("IMS file is required")?;
  let hsm_file_value = hsm_file.context("HSM file is required")?;

  if let Some(prehook_path) = prehook {
    crate::common::hooks::check_hook_perms(Some(prehook_path))
      .map_err(|e| anyhow::anyhow!("{e}. File: {prehook_path}"))?;
    tracing::debug!("Pre-hook script exists and is executable.");
  }
  if let Some(posthook_path) = posthook {
    crate::common::hooks::check_hook_perms(Some(posthook_path))
      .map_err(|e| anyhow::anyhow!("{e}. File: {posthook_path}"))?;
    tracing::debug!("Post-hook script exists and is executable.");
  }

  let req = MigrateRestoreRequest {
    bos_file: bos_file.map(str::to_string),
    cfs_file: cfs_file.map(str::to_string),
    hsm_file: hsm_file.map(str::to_string),
    ims_file: ims_file.map(str::to_string),
    image_dir: image_dir.map(str::to_string),
    overwrite: Some(overwrite),
  };

  if dry_run {
    return action_result::preview_request(
      "POST",
      "/migrate/restore",
      &req,
      output_opt,
    );
  }

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
  println!();

  crate::common::hooks::run_hook_if_present(prehook, "pre")?;

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .migrate_restore(client.site_name(), &req)
    .await
    .into_anyhow()?;

  crate::common::hooks::run_hook_if_present(posthook, "post")?;

  action_result::print(
    "Done, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored.",
    output_opt,
  )?;

  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta restore vcluster` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "restore",
      "vcluster",
      "--bos-file",
      "/tmp/bos.yaml",
      "--cfs-file",
      "/tmp/cfs.yaml",
      "--hsm-file",
      "/tmp/hsm.yaml",
      "--ims-file",
      "/tmp/ims.yaml",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `restore vcluster`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "restore",
      "vcluster",
      "--bos-file",
      "/tmp/bos.yaml",
      "--cfs-file",
      "/tmp/cfs.yaml",
      "--hsm-file",
      "/tmp/hsm.yaml",
      "--ims-file",
      "/tmp/ims.yaml",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }
}
