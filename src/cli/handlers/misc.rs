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
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  match cli_root.subcommand() {
    Some(("validate-local-repo", m)) => {
      let repo_path = m
        .get_one::<String>("repo-path")
        .context("The 'repo-path' argument must have a value")?;
      validate_local_repo::exec(
        ctx.infra.backend,
        ctx.infra.site_name,
        &token,
        ctx.infra.shasta_root_cert,
        ctx.infra.vault_base_url,
        ctx.infra.gitea_base_url,
        repo_path,
      )
      .await?;
    }
    Some(("add-nodes-to-groups", m)) => {
      let dryrun = m.get_flag("dry-run");
      let hosts_expression = m
        .get_one::<String>("nodes")
        .context("The 'nodes' argument must have a value")?;
      let target_hsm_name: &str = m
        .get_one::<String>("group")
        .map(String::as_str)
        .context("The 'group' argument is mandatory")?;
      add_nodes_to_hsm_groups::exec(
        ctx,
        &token,
        target_hsm_name,
        hosts_expression,
        dryrun,
        ctx.cli.kafka_audit_opt,
      )
      .await?;
    }
    Some(("remove-nodes-from-groups", m)) => {
      let dryrun = m.get_flag("dry-run");
      let nodes = m
        .get_one::<String>("nodes")
        .context("The 'nodes' argument must have a value")?;
      let target_hsm_name: &str = m
        .get_one::<String>("group")
        .map(String::as_str)
        .context("The 'group' argument is mandatory")?;
      remove_nodes_from_hsm_groups::exec(
        ctx.infra.backend,
        &token,
        target_hsm_name,
        nodes,
        dryrun,
        ctx.cli.kafka_audit_opt,
      )
      .await?;
    }
    Some(("download-boot-image", _)) => println!("Download boot image"),
    Some(("upload-boot-image", _)) => println!("Upload boot image"),
    Some((other, _)) => bail!("Unknown command: {other}"),
    None => bail!("No command provided"),
  }
  Ok(())
}
