use crate::cli::commands::{
  add_nodes_to_hsm_groups, remove_nodes_from_hsm_groups, validate_local_repo,
};
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch top-level misc commands (validate-local-repo,
/// add-nodes-to-groups, remove-nodes-from-groups).
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
      ctx.infra.backend,
      ctx.infra.site_name,
      ctx.infra.shasta_root_cert,
      ctx.infra.vault_base_url,
      ctx.infra.gitea_base_url,
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
    let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;
    add_nodes_to_hsm_groups::exec(
      ctx,
      &token,
      target_hsm_name,
      hosts_expression,
      dryrun,
      ctx.cli.kafka_audit_opt,
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
      ctx.infra.backend,
      ctx.infra.site_name,
      target_hsm_name,
      nodes,
      dryrun,
      ctx.cli.kafka_audit_opt,
    )
    .await?;
  } else if cli_root.subcommand_matches("download-boot-image").is_some() {
    println!("Download boot image");
  } else if cli_root.subcommand_matches("upload-boot-image").is_some() {
    println!("Upload boot image");
  } else {
    bail!("Unknown command");
  }
  Ok(())
}
