use anyhow::{Context, Error};
use clap::ArgMatches;

use crate::{
  cli::commands::hw_cluster_common::command::{self, HwClusterMode},
  common::{
    app_context::AppContext, authentication::get_api_token,
    authorization::get_groups_names_available,
  },
};

/// Apply a hardware cluster configuration (pin or unpin).
pub async fn exec(
  cli_apply_hw_cluster: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;

  let shasta_token = get_api_token(backend, site_name).await?;

  let target_hsm_group_name_arg_opt: Option<&str> = cli_apply_hw_cluster
    .get_one::<String>("target-cluster")
    .map(String::as_str);
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    target_hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  let parent_hsm_group_name_arg_opt: Option<&str> = cli_apply_hw_cluster
    .get_one::<String>("parent-cluster")
    .map(String::as_str);
  let parent_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    parent_hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

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

  command::exec(
    mode,
    ctx,
    &shasta_token,
    target_hsm_group_vec
      .first()
      .context("No target HSM group found")?,
    parent_hsm_group_vec
      .first()
      .context("No parent HSM group found")?,
    cli_apply_hw_cluster
      .get_one::<String>("pattern")
      .context("pattern argument is required")?,
    dryrun,
    create_target_hsm_group,
    delete_empty_parent_hsm_group,
  )
  .await?;

  Ok(())
}
