use crate::cli::commands::{
  add_nodes_to_hsm_groups, remove_nodes_from_hsm_groups, validate_local_repo,
};
use crate::common::app_context::AppContext;
use anyhow::{Context, Error};
use clap::ArgMatches;

pub async fn handle_misc(
  cli_root: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(cli_validate_local_repo) =
    cli_root.subcommand_matches("validate-local-repo")
  {
    let repo_path = cli_validate_local_repo
      .get_one::<String>("repo-path")
      .context("The 'repo-path' argument must have a value")?;
    validate_local_repo::exec(
      ctx.backend,
      ctx.site_name,
      ctx.shasta_root_cert,
      ctx.vault_base_url,
      ctx.gitea_base_url,
      repo_path,
    )
    .await?;
  } else if let Some(cli_add_nodes) =
    cli_root.subcommand_matches("add-nodes-to-groups")
  {
    let dryrun = cli_add_nodes.get_flag("dry-run");
    let hosts_expression = cli_add_nodes
      .get_one::<String>("nodes")
      .context("The 'nodes' argument must have a value")?;
    let target_hsm_name: &str = cli_add_nodes
      .get_one::<String>("group")
      .map(String::as_str)
      .context("The 'group' argument is mandatory")?;
    add_nodes_to_hsm_groups::exec(
      ctx.backend,
      ctx.site_name,
      target_hsm_name,
      hosts_expression,
      dryrun,
      ctx.kafka_audit_opt,
    )
    .await?;
  } else if let Some(cli_remove_nodes) =
    cli_root.subcommand_matches("remove-nodes-from-groups")
  {
    let dryrun = cli_remove_nodes.get_flag("dry-run");
    let nodes = cli_remove_nodes
      .get_one::<String>("nodes")
      .context("The 'nodes' argument must have a value")?;
    let target_hsm_name: &str = cli_remove_nodes
      .get_one::<String>("group")
      .map(String::as_str)
      .context("The 'group' argument is mandatory")?;
    remove_nodes_from_hsm_groups::exec(
      ctx.backend,
      ctx.site_name,
      target_hsm_name,
      nodes,
      dryrun,
      ctx.kafka_audit_opt,
    )
    .await?;
  } else if cli_root.subcommand_matches("download-boot-image").is_some() {
    println!("Download boot image");
  } else if cli_root.subcommand_matches("upload-boot-image").is_some() {
    println!("Upload boot image");
  }
  Ok(())
}
