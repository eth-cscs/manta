use anyhow::Error;
use clap::ArgMatches;

use crate::{
  cli::commands::{apply_hw_cluster_pin, apply_hw_cluster_unpin},
  common::{
    authentication::get_api_token, authorization::get_groups_names_available,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn process_subcommand(
  cli_apply_hw_cluster: &ArgMatches,
  backend: StaticBackendDispatcher,
  site_name: &str,
  settings_hsm_group_name_opt: Option<&String>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  let target_hsm_group_name_arg_opt: Option<&String> =
    cli_apply_hw_cluster.get_one("target-cluster");
  let target_hsm_group_vec = get_groups_names_available(
    &backend,
    &shasta_token,
    target_hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  let parent_hsm_group_name_arg_opt: Option<&String> =
    cli_apply_hw_cluster.get_one("parent-cluster");
  let parent_hsm_group_vec = get_groups_names_available(
    &backend,
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

  if *is_unpin {
    apply_hw_cluster_unpin::command::exec(
      &backend,
      &shasta_token,
      target_hsm_group_vec.first().unwrap(),
      parent_hsm_group_vec.first().unwrap(),
      cli_apply_hw_cluster.get_one::<String>("pattern").unwrap(),
      dryrun,
      create_target_hsm_group,
      delete_empty_parent_hsm_group,
    )
    .await;
  } else {
    apply_hw_cluster_pin::command::exec(
      &backend,
      &shasta_token,
      target_hsm_group_vec.first().unwrap(),
      parent_hsm_group_vec.first().unwrap(),
      cli_apply_hw_cluster.get_one::<String>("pattern").unwrap(),
      dryrun,
      create_target_hsm_group,
      delete_empty_parent_hsm_group,
    )
    .await;
  }

  Ok(())
}
