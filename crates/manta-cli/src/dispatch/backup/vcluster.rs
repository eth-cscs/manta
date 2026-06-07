//! Implements the `manta backup vcluster` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub bos: Option<&'a str>,
  pub destination: Option<&'a str>,
  pub prehook: Option<&'a str>,
  pub posthook: Option<&'a str>,
  pub output: Option<&'a str>,
}

/// Back up cluster configuration to a local bundle.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let bos = p.bos;
  let destination = p.destination;
  let prehook = p.prehook;
  let posthook = p.posthook;
  let output_opt = p.output;
  let bos_value = bos.context("BOS template is required")?;
  let destination_value =
    destination.context("Destination folder is required")?;

  action_result::print(
    &format!(
      "Migrate backup\n BOS Template: {}\n Destination folder: {}\n Pre-hook: {}\n Post-hook: {}",
      bos_value,
      destination_value,
      prehook.unwrap_or("none"),
      posthook.unwrap_or("none"),
    ),
    output_opt,
  )?;

  if let Some(prehook_path) = prehook {
    match crate::common::hooks::check_hook_perms(Some(prehook_path)) {
      Ok(_r) => {
        tracing::debug!("Pre-hook script exists and is executable.")
      }
      Err(e) => {
        bail!("{e}. File: {prehook_path}");
      }
    }
  }
  if let Some(posthook_path) = posthook {
    match crate::common::hooks::check_hook_perms(Some(posthook_path)) {
      Ok(_) => {
        tracing::debug!("Post-hook script exists and is executable.")
      }
      Err(e) => {
        bail!("{e}. File: {posthook_path}");
      }
    }
  }

  crate::common::hooks::run_hook_if_present(prehook, "pre")?;

  MantaClient::from_app_ctx(ctx)?
    .backup_vcluster(token, bos, destination)
    .await?;
  tracing::debug!("Migrate backup completed successfully.");

  crate::common::hooks::run_hook_if_present(posthook, "post")?;

  action_result::print("Backup completed", output_opt)?;

  Ok(())
}
