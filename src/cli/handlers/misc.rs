use crate::cli::commands::{
  add_nodes_to_hsm_groups, remove_nodes_from_hsm_groups, validate_local_repo,
};
use crate::common::kafka::Kafka;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;

pub async fn handle_misc(
  cli_root: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_root_cert: &[u8],
  vault_base_url: Option<&String>,
  gitea_base_url: &str,
  _settings_hsm_group_name_opt: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  if let Some(cli_validate_local_repo) =
    cli_root.subcommand_matches("validate-local-repo")
  {
    let repo_path = cli_validate_local_repo
      .get_one::<String>("repo-path")
      .unwrap();
    validate_local_repo::exec(
      backend,
      site_name,
      shasta_root_cert,
      vault_base_url,
      gitea_base_url,
      repo_path,
    )
    .await?;
  } else if let Some(cli_add_nodes) =
    cli_root.subcommand_matches("add-nodes-to-groups")
  {
    let dryrun = cli_add_nodes.get_flag("dry-run");
    let hosts_expression = cli_add_nodes.get_one::<String>("nodes").unwrap();
    let target_hsm_name: &String = cli_add_nodes
      .get_one::<String>("group")
      .expect("Error - target cluster is mandatory");
    add_nodes_to_hsm_groups::exec(
      backend,
      site_name,
      target_hsm_name,
      hosts_expression,
      dryrun,
      kafka_audit_opt,
    )
    .await?;
  } else if let Some(cli_remove_nodes) =
    cli_root.subcommand_matches("remove-nodes-from-groups")
  {
    let dryrun = cli_remove_nodes.get_flag("dry-run");
    let nodes = cli_remove_nodes.get_one::<String>("nodes").unwrap();
    let target_hsm_name: &String = cli_remove_nodes
      .get_one::<String>("group")
      .expect("Error - target cluster is mandatory");
    remove_nodes_from_hsm_groups::exec(
      backend,
      site_name,
      target_hsm_name,
      nodes,
      dryrun,
      kafka_audit_opt,
    )
    .await?;
  } else if let Some(_) = cli_root.subcommand_matches("download-boot-image") {
    println!("Download boot image");
  } else if let Some(_) = cli_root.subcommand_matches("upload-boot-image") {
    println!("Upload boot image");
  }
  Ok(())
}
