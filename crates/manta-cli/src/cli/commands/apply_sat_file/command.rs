//! Implements the `manta apply sat-file` command.
//!
//! Jinja2 rendering, `SatFile` parsing, and the `image_only` /
//! `session_template_only` filters all run client-side so the operator
//! can preview the final YAML before any backend mutation. The slim
//! `POST /sat-file` endpoint then forwards the post-processed YAML to
//! the backend together with the Vault/K8s context.

use anyhow::{Context, Error, bail};
use crossterm::style::Stylize;

use crate::cli::common;
use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::sat_file::{SatFile, render_jinja2_sat_file_yaml};

/// Options for applying a SAT file.
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
  pub output_opt: Option<&'a str>,
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

  // 1. Render Jinja2 (shared helper, MantaError -> anyhow)
  tracing::info!("Render SAT template file");
  let rendered_yaml = render_jinja2_sat_file_yaml(
    opts.sat_file_content,
    opts.values_file_content_opt,
    opts.values_cli_opt,
  )
  .map_err(|e| anyhow::anyhow!("{e}"))?;

  // 2. Parse + filter the SAT file (image_only / session_template_only).
  //
  // Round-trip through a YAML string: `serde_yaml::from_value` is brittle
  // around untagged enums (see SatFile's `BaseOrIms` / `ImageIms`), while
  // `from_str` handles them reliably.
  let rendered_str = serde_yaml::to_string(&rendered_yaml)
    .context("Failed to serialize rendered SAT template to YAML")?;
  let mut sat_file: SatFile = serde_yaml::from_str(&rendered_str)
    .context("Could not parse rendered SAT template into SatFile")?;
  sat_file
    .filter(opts.image_only, opts.session_template_only)
    .map_err(|e| anyhow::anyhow!("{e}"))?;

  let sat_yaml_string = serde_yaml::to_string(&sat_file)
    .context("Failed to serialize filtered SAT file to YAML")?;

  // 3. Display + confirm.
  println!(
    "{}\n{}",
    "#### SAT file content ####".blue(),
    &sat_yaml_string,
  );
  if !common::user_interaction::confirm(
    "Please review the rendered SAT file above and confirm to proceed.",
    opts.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  // 4. Extra reboot confirmation if session templates are present.
  if sat_file.session_templates.is_some()
    && opts.reboot
    && !common::user_interaction::confirm(
      "This operation will reboot nodes. Please confirm to proceed.",
      opts.assume_yes,
    )
  {
    bail!("Operation cancelled by user");
  }

  // 5. Pre-hook -> server call -> post-hook.
  run_hook_if_present(opts.prehook_opt, "pre")?;

  let result = MantaClient::new(server_url, ctx.site_name)?
    .apply_sat_file(
      token,
      &sat_yaml_string,
      opts.ansible_verbosity_opt,
      opts.ansible_passthrough_opt,
      opts.reboot,
      opts.watch_logs,
      opts.timestamps,
      opts.overwrite,
      opts.dry_run,
    )
    .await?;

  run_hook_if_present(opts.posthook_opt, "post")?;

  let message = if opts.dry_run {
    "Dry-run enabled. No changes persisted into the system."
  } else {
    "SAT file applied."
  };
  action_result::print_with_data(message, &result, opts.output_opt)?;

  Ok(())
}
