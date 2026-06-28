//! Implements the `manta apply hardware group` command.
//!
//! Pins (or unpins, with `--unpin-nodes`) hardware components matching
//! `--pattern` to a target HSM cluster, moving them from `--parent-group`.
//! Forwards to `POST /api/v1/hardware-clusters/{target}` with the
//! request's `dry_run` flag, `create_target_hsm_group`, and
//! `delete_empty_parent_hsm_group` flags honoured server-side. See
//! [`super::super::add::hardware`] for the variant that *adds*
//! components without flipping the pin mode.

use anyhow::{Context, Error};
use clap::ArgMatches;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::{
  ApplyHwConfigurationRequest, HwClusterMode,
};
use crate::output::action_result;

/// Apply a hardware cluster configuration (pin or unpin).
///
/// # Errors
///
/// Returns an error when `--pattern` is missing, when neither the CLI
/// nor `cli.toml` supplies a target or parent group, when the HTTP
/// client cannot be built, or when the `apply_hw_configuration` call
/// fails.
pub async fn exec(
  cli_apply_hw_group: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let settings_hsm_group_name_opt = ctx.settings_group_name_opt;

  let target_hsm_group_name_arg_opt =
    cli_apply_hw_group.opt_str("target-group");
  let parent_hsm_group_name_arg_opt =
    cli_apply_hw_group.opt_str("parent-group");
  let dryrun = cli_apply_hw_group.get_flag("dry-run");
  let create_target_hsm_group = *cli_apply_hw_group
    .get_one::<bool>("create-target-group")
    .unwrap_or(&true);
  let delete_empty_parent_hsm_group = *cli_apply_hw_group
    .get_one::<bool>("delete-empty-parent-group")
    .unwrap_or(&true);
  let is_unpin = cli_apply_hw_group
    .get_one::<bool>("unpin-nodes")
    .unwrap_or(&false);
  let mode = if *is_unpin {
    HwClusterMode::Unpin
  } else {
    HwClusterMode::Pin
  };
  let pattern = cli_apply_hw_group.req_str("pattern")?;

  let target = target_hsm_group_name_arg_opt
    .or(settings_hsm_group_name_opt)
    .context("No target HSM group specified")?;
  let parent = parent_hsm_group_name_arg_opt
    .or(settings_hsm_group_name_opt)
    .context("No parent HSM group specified")?;
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .apply_hw_configuration(
      target,
      client.site_name(),
      &ApplyHwConfigurationRequest {
        parent_cluster: parent.to_string(),
        pattern: pattern.to_string(),
        mode: Some(mode),
        dry_run: Some(dryrun),
        create_target_hsm_group: Some(create_target_hsm_group),
        delete_empty_parent_hsm_group: Some(delete_empty_parent_hsm_group),
      },
    )
    .await
    .into_anyhow()?;
  let output_opt = cli_apply_hw_group.opt_str("output");
  let message = if dryrun {
    "Dry run enabled, not modifying the HSM groups on the system."
  } else {
    "Hardware configuration applied."
  };
  action_result::print_with_data(message, &result, output_opt)?;
  Ok(())
}
