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
//! After the preview confirm and the optional reboot confirm, the
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

use crate::commands::apply_sat_file::{dispatch, plan};
use crate::common;
use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;
use crate::common::sat_file::render_jinja2_sat_file_yaml;

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
    crate::common::hooks::check_hook_perms(hook_opt)
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
    let code = crate::common::hooks::run_hook(hook_opt)?;
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

  // 1. Render Jinja2 (text-in / text-out).
  tracing::info!("Render SAT template file");
  let rendered_yaml = render_jinja2_sat_file_yaml(
    opts.sat_file_content,
    opts.values_file_content_opt,
    opts.values_cli_opt,
  )
  .map_err(|e| anyhow::anyhow!("{e}"))?;

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
  if !common::user_interaction::confirm(
    "Please review the rendered SAT file above and confirm to proceed.",
    opts.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  // 5. Extra reboot confirmation if session_templates are still present
  //    after filtering.
  if sat_file.get("session_templates").is_some()
    && opts.reboot
    && !common::user_interaction::confirm(
      "This operation will reboot nodes. Please confirm to proceed.",
      opts.assume_yes,
    )
  {
    bail!("Operation cancelled by user");
  }

  // 6. Plan was built in step 3 (build_plan also pruned `sat_file`
  //    according to the filter flags). Dispatch of the plan
  //    element-by-element is the next refactor; for now we log its
  //    shape and still ship the whole SAT to the server.
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

  // 7. Pre-hook -> server call -> post-hook.
  run_hook_if_present(opts.prehook_opt, "pre")?;

  // 7a. Dispatch the plan element-by-element. The CLI accumulates
  //     `ref_name → image_id` across calls and builds the same
  //     four-list response the legacy endpoint used to return.
  let client = MantaClient::new(server_url, ctx.site_name)?;
  let result = dispatch::dispatch_plan(&client, token, plan, opts).await?;

  run_hook_if_present(opts.posthook_opt, "post")?;

  let message = if opts.dry_run {
    "Dry-run enabled. No changes persisted into the system."
  } else {
    "SAT file applied."
  };
  action_result::print_with_data(message, &result, opts.output_opt)?;

  Ok(())
}
