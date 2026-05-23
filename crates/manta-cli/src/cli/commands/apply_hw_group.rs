//! Implements the `manta apply hardware cluster` command.

use anyhow::{Context, Error};
use clap::ArgMatches;

use crate::cli::common::clap_ext::ArgMatchesExt;
use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::params::hw_cluster::HwClusterMode;

/// Apply a hardware cluster configuration (pin or unpin).
pub async fn exec(
  cli_apply_hw_group: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;

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

  let server_url = ctx.manta_server_url;
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
  let result = MantaClient::new(server_url, ctx.site_name)?
    .apply_hw_configuration(
      token,
      target,
      parent,
      pattern,
      mode_str,
      dryrun,
      create_target_hsm_group,
      delete_empty_parent_hsm_group,
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
