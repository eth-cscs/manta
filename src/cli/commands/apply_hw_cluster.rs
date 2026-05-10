//! Implements the `manta apply hardware cluster` command.

use anyhow::{Context, Error};
use clap::ArgMatches;

use crate::{
  cli::http_client::MantaClient,
  common::app_context::AppContext,
  service::hw_cluster::HwClusterMode,
};

/// Apply a hardware cluster configuration (pin or unpin).
pub async fn exec(
  cli_apply_hw_cluster: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let settings_hsm_group_name_opt = ctx.cli.settings_hsm_group_name_opt;

  let target_hsm_group_name_arg_opt: Option<&str> = cli_apply_hw_cluster
    .get_one::<String>("target-cluster")
    .map(String::as_str);
  let parent_hsm_group_name_arg_opt: Option<&str> = cli_apply_hw_cluster
    .get_one::<String>("parent-cluster")
    .map(String::as_str);
  let dryrun = cli_apply_hw_cluster.get_flag("dry-run");
  let create_target_hsm_group = *cli_apply_hw_cluster
    .get_one::<bool>("create-target-hsm-group")
    .unwrap_or(&true);
  let delete_empty_parent_hsm_group = *cli_apply_hw_cluster
    .get_one::<bool>("delete-empty-parent-hsm-group")
    .unwrap_or(&true);
  let is_unpin = cli_apply_hw_cluster
    .get_one::<bool>("unpin-nodes")
    .unwrap_or(&false);
  let mode = if *is_unpin {
    HwClusterMode::Unpin
  } else {
    HwClusterMode::Pin
  };
  let pattern = cli_apply_hw_cluster
    .get_one::<String>("pattern")
    .context("pattern argument is required")?;

  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
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
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
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
  if dryrun {
    println!("Dry run enabled, not modifying the HSM groups on the system.");
  }
  println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
  Ok(())
}
