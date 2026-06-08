//! Implements the `manta apply hardware group` command.

use anyhow::{Context, Error};
use clap::ArgMatches;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{ApplyHwConfigurationRequest, MantaClient};
use crate::output::action_result;
use manta_shared::types::params::hw_cluster::HwClusterMode;

/// Apply a hardware cluster configuration (pin or unpin).
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
  let mode_str = match mode {
    HwClusterMode::Pin => "pin",
    HwClusterMode::Unpin => "unpin",
  };
  let result = MantaClient::from_app_ctx(ctx)?
    .apply_hw_configuration(
      token,
      target,
      &ApplyHwConfigurationRequest {
        parent_cluster: parent,
        pattern,
        mode: mode_str,
        dry_run: dryrun,
        create_target_hsm_group,
        delete_empty_parent_hsm_group,
      },
    )
    .await?;
  let output_opt = cli_apply_hw_group.opt_str("output");
  let message = if dryrun {
    "Dry run enabled, not modifying the HSM groups on the system."
  } else {
    "Hardware configuration applied."
  };
  action_result::print_with_data(message, &result, output_opt)?;
  Ok(())
}
