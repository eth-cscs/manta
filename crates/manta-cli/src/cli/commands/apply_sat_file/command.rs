//! Implements the `manta apply sat-file` command.

use anyhow::{Error, bail};

use crate::cli::common;
use manta_shared::common::app_context::AppContext;

use crate::cli::http_client::MantaClient;

/// Options for applying a SAT file.
///
/// Bundles the many parameters needed by [`exec`] into a
/// single struct, improving call-site readability.
//
// Mirrors `ApplySatFileParams` on the shared/server side — see the
// note there for why `struct_excessive_bools` is silenced.
#[allow(clippy::struct_excessive_bools)]
pub struct SatApplyOptions<'a> {
  pub sat_file_content: &'a str,
  pub values_file_content_opt: Option<&'a str>,
  pub values_cli_opt: Option<&'a [String]>,
  pub ansible_verbosity_opt: Option<u8>,
  pub ansible_passthrough_opt: Option<&'a str>,
  pub reboot: bool,
  pub watch_logs: bool,
  pub timestamps: bool,
  pub prehook_opt: Option<&'a str>,
  pub posthook_opt: Option<&'a str>,
  pub image_only: bool,
  pub session_template_only: bool,
  pub overwrite: bool,
  pub dry_run: bool,
  pub assume_yes: bool,
}

/// Validate that a hook script exists and is executable.
fn validate_hook(hook_opt: Option<&str>, label: &str) -> Result<(), Error> {
  if let Some(hook) = hook_opt {
    crate::cli::common::hooks::check_hook_perms(hook_opt)
      .map_err(|e| anyhow::anyhow!("{e}. File: {hook}"))?;
    println!("{label}-hook script '{hook}' exists and is executable.");
  }
  Ok(())
}

/// Run a hook script if one was provided.
fn run_hook_if_present(
  hook_opt: Option<&str>,
  label: &str,
) -> Result<(), Error> {
  if let Some(hook) = hook_opt {
    println!("Running the {label}-hook '{hook}'");
    let code = crate::cli::common::hooks::run_hook(hook_opt)?;
    tracing::debug!("{}-hook script completed ok. RT={}", label, code);
  }
  Ok(())
}

/// Process and apply a SAT file to the system.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  opts: &SatApplyOptions<'_>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  validate_hook(opts.prehook_opt, "Pre")?;
  validate_hook(opts.posthook_opt, "Post")?;

  if !common::user_interaction::confirm(
    "Apply SAT file to the system via manta server. Please confirm to proceed.",
    opts.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  run_hook_if_present(opts.prehook_opt, "pre")?;

  let values_json: Option<serde_json::Value> =
    opts.values_cli_opt.map(|vals| {
      serde_json::Value::Array(
        vals
          .iter()
          .map(|v| serde_json::Value::String(v.clone()))
          .collect(),
      )
    });

  MantaClient::new(server_url, ctx.site_name)?
    .apply_sat_file(
      token,
      opts.sat_file_content,
      values_json,
      opts.values_file_content_opt,
      opts.ansible_verbosity_opt,
      opts.ansible_passthrough_opt,
      opts.reboot,
      opts.watch_logs,
      opts.timestamps,
      opts.image_only,
      opts.session_template_only,
      opts.overwrite,
      opts.dry_run,
    )
    .await?;

  run_hook_if_present(opts.posthook_opt, "post")?;
  Ok(())
}
