use crate::cli::commands::{
  migrate_backup, migrate_nodes_between_hsm_groups, migrate_restore,
};
use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
  kafka::Kafka,
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;

pub async fn handle_migrate(
  cli_migrate: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  settings_hsm_group_name_opt: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  if let Some(cli_migrate_nodes) = cli_migrate.subcommand_matches("nodes") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let dry_run: bool = cli_migrate_nodes.get_flag("dry-run");
    let from_opt: Option<&String> = cli_migrate_nodes.get_one("from");
    let to: &String = cli_migrate_nodes
      .get_one("to")
      .expect("to value is mandatory");
    let xnames_string: &String = cli_migrate_nodes.get_one("XNAMES").unwrap();
    let from_rslt = get_groups_names_available(
      backend,
      &shasta_token,
      from_opt,
      settings_hsm_group_name_opt,
    )
    .await;
    let from = match from_rslt {
      Ok(from) => from,
      Err(e) => {
        return Err(Error::msg(e));
      }
    };
    let to_rslt = get_groups_names_available(
      backend,
      &shasta_token,
      Some(to),
      settings_hsm_group_name_opt,
    )
    .await;
    let to = match to_rslt {
      Ok(to) => to,
      Err(e) => {
        return Err(Error::msg(e));
      }
    };
    migrate_nodes_between_hsm_groups::exec(
      backend,
      site_name,
      &to,
      &from,
      xnames_string,
      !dry_run,
      false,
      kafka_audit_opt,
    )
    .await?;
  } else if let Some(_cli_migrate_vcluster) =
    cli_migrate.subcommand_matches("vCluster")
  {
    if let Some(cli_migrate_vcluster_backup) =
      cli_migrate.subcommand_matches("backup")
    {
      let bos: Option<&String> = cli_migrate_vcluster_backup.get_one("bos");
      let destination: Option<&String> =
        cli_migrate_vcluster_backup.get_one("destination");
      let prehook: Option<&String> =
        cli_migrate_vcluster_backup.get_one("pre-hook");
      let posthook: Option<&String> =
        cli_migrate_vcluster_backup.get_one("post-hook");
      migrate_backup::exec(
        backend,
        site_name,
        shasta_base_url,
        shasta_root_cert,
        bos.map(String::as_str),
        destination.map(String::as_str),
        prehook.map(String::as_str),
        posthook.map(String::as_str),
      )
      .await?;
    } else if let Some(cli_migrate_vcluster_restore) =
      cli_migrate.subcommand_matches("restore")
    {
      let bos_file: Option<&String> =
        cli_migrate_vcluster_restore.get_one("bos-file");
      let cfs_file: Option<&String> =
        cli_migrate_vcluster_restore.get_one("cfs-file");
      let hsm_file: Option<&String> =
        cli_migrate_vcluster_restore.get_one("hsm-file");
      let ims_file: Option<&String> =
        cli_migrate_vcluster_restore.get_one("ims-file");
      let image_dir: Option<&String> =
        cli_migrate_vcluster_restore.get_one("image-dir");
      let prehook: Option<&String> =
        cli_migrate_vcluster_restore.get_one("pre-hook");
      let posthook: Option<&String> =
        cli_migrate_vcluster_restore.get_one("post-hook");
      let overwrite: bool = cli_migrate_vcluster_restore.get_flag("overwrite");
      migrate_restore::exec(
        backend,
        site_name,
        shasta_base_url,
        shasta_root_cert,
        bos_file.map(String::as_str),
        cfs_file.map(String::as_str),
        hsm_file.map(String::as_str),
        ims_file.map(String::as_str),
        image_dir.map(String::as_str),
        prehook.map(String::as_str),
        posthook.map(String::as_str),
        overwrite,
      )
      .await?;
    }
  }
  Ok(())
}
