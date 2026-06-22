//! Implements the `manta apply sat-file` command.
//!
//! Everything before any HTTP call runs client-side so the operator
//! can preview and abort: Jinja2 rendering, parsing into a
//! `serde_json::Value`, applying the `image_only` /
//! `session_template_only` filters (prune-by-reference over the parsed
//! Value), and building the in-memory execution plan
//! (configurations → images topologically sorted by `base.image_ref` →
//! session_templates, with up-front validation of cross-references).
//! See the sibling [`plan`] module for the plan builder.
//!
//! After the preview confirm and the optional create-BOS-session confirm, the
//! command hands the plan to [`dispatch::dispatch_plan`], which POSTs
//! one element per request to the per-element server endpoints
//! (`POST /sat-file/{configurations,images,session-templates}`) and
//! accumulates the `ref_name → image_id` map between calls so
//! downstream `image_ref` references resolve.
//!
//! The CLI never embeds the SAT schema: filtering and plan-building
//! navigate a small set of field names (`configurations`, `images`,
//! `session_templates`, `hardware`, `name`, `ref_name`, `configuration`,
//! `image`, `image_ref`, `ims`) on the `serde_json::Value`. The
//! canonical schema lives in csm-rs (which deserialises during apply).
//!
//! On success [`dispatch::dispatch_plan`] returns the same four-list
//! summary the legacy `POST /sat-file` endpoint used to produce
//! (`configurations`, `images`, `session_templates`, `bos_sessions`);
//! the command pipes it through
//! [`crate::output::action_result::print_with_data`] so the user
//! sees a status message plus pretty-printed JSON (or `{ "status":
//! "ok", "message": ..., "data": ... }` with `--output json`).

use anyhow::{Context, Error, bail};
use crossterm::style::Stylize;

use super::render::render_jinja2_sat_file_yaml;
use crate::common;
use crate::common::app_context::AppContext;
use crate::dispatch::apply::sat_file::{dispatch, plan};
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::PostSatValidateRequest;
use crate::output::action_result;

/// Options for applying a SAT file.
#[allow(clippy::struct_excessive_bools)]
pub struct SatApplyOptions<'a> {
  pub sat_file_content: &'a str,
  pub values_file_content_opt: Option<&'a str>,
  pub values_cli_opt: Option<&'a [String]>,
  pub ansible_verbosity_opt: Option<u8>,
  pub ansible_passthrough_opt: Option<&'a str>,
  /// After each BOS session template is created, immediately create a
  /// BOS session from it so its target nodes boot via the new template.
  /// This typically causes a reboot.
  pub create_bos_session: bool,
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
    crate::common::hooks::check_hook_perms(hook_opt)
      .with_context(|| format!("Hook script '{hook}'"))?;
    println!("{label}-hook script '{hook}' exists and is executable.");
  }
  Ok(())
}

/// Process and apply a SAT file to the system.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  opts: &SatApplyOptions<'_>,
) -> Result<(), Error> {
  validate_hook(opts.prehook_opt, "Pre")?;
  validate_hook(opts.posthook_opt, "Post")?;

  // 1. Render Jinja2 (text-in / text-out).
  tracing::info!("Render SAT template file");
  let rendered_yaml = render_jinja2_sat_file_yaml(
    opts.sat_file_content,
    opts.values_file_content_opt,
    opts.values_cli_opt,
  )
  .context("Failed to render SAT Jinja2 template")?;

  // 2. Parse into a structured value. The CLI carries the SAT file as
  //    `serde_json::Value` end-to-end — the server forwards it
  //    verbatim, and csm-rs transcodes to its preferred shape during
  //    apply. No SAT schema lives in the CLI.
  let mut sat_file: serde_json::Value = serde_yaml::from_str(&rendered_yaml)
    .context("Rendered SAT template is not valid YAML")?;

  // 3. Build the ordered execution plan from the parsed SAT file. This
  //    also applies the --image-only / --sessiontemplate-only filters
  //    in place (prunes `sat_file`), so the preview below shows the
  //    same surviving sections that the plan will execute.
  let plan = plan::build_plan(
    &mut sat_file,
    opts.image_only,
    opts.session_template_only,
  )?;

  // 4. Display the filtered SAT file as YAML and confirm.
  let preview = serde_yaml::to_string(&sat_file)
    .context("Failed to serialize filtered SAT value for preview")?;
  println!("{}\n{}", "#### SAT file content ####".blue(), &preview,);
  if !common::confirm::confirm(
    "Please review the rendered SAT file above and confirm to proceed.",
    opts.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  // 5. Extra create-BOS-session confirmation if session_templates are
  //    still present after filtering.
  if sat_file.get("session_templates").is_some()
    && opts.create_bos_session
    && !common::confirm::confirm(
      "This operation will create a BOS session for each new template, \
       which will boot affected nodes through it (typically a reboot). \
       Please confirm to proceed.",
      opts.assume_yes,
    )
  {
    bail!("Operation cancelled by user");
  }

  // 6. Log the plan shape for operator visibility before the hook +
  //    dispatch pair runs (step 3 already pruned `sat_file` to match).
  let (n_cfg, n_img, n_st) =
    plan.iter().fold((0_usize, 0, 0), |(c, i, s), e| match e {
      plan::SatElement::Configuration(_) => (c + 1, i, s),
      plan::SatElement::Image(_) => (c, i + 1, s),
      plan::SatElement::SessionTemplate(_) => (c, i, s + 1),
    });
  tracing::info!(
    "Built execution plan: {} configurations, {} images, {} session_templates",
    n_cfg,
    n_img,
    n_st,
  );

  // 6a. Pre-flight: server-side validation against live CSM state.
  //     Built first so the same client is reused for dispatch below.
  //     Failing here aborts before the pre-hook fires.
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .post_sat_validate(
      client.site_name(),
      &PostSatValidateRequest {
        sat_file: sat_file.clone(),
      },
    )
    .await
    .into_anyhow()
    .context("Server-side SAT validation failed")?;
  tracing::info!("SAT file validated server-side");

  // 7. Pre-hook -> server call -> post-hook.
  crate::common::hooks::run_hook_if_present(opts.prehook_opt, "pre")?;

  // 7a. Dispatch the plan element-by-element. The CLI accumulates
  //     `ref_name → image_id` across calls and builds the same
  //     four-list response the legacy endpoint used to return.
  let result = dispatch::dispatch_plan(ctx, &client, plan, opts).await?;

  crate::common::hooks::run_hook_if_present(opts.posthook_opt, "post")?;

  let message = if opts.dry_run {
    "Dry-run enabled. No changes persisted into the system."
  } else {
    "SAT file applied."
  };
  action_result::print_with_data(message, &result, opts.output_opt)?;

  Ok(())
}
