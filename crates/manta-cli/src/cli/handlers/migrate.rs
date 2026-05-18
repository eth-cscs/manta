//! Routes `manta migrate *` subcommands to their exec functions.

use crate::cli::commands::{
  migrate_backup, migrate_nodes_between_hsm_groups, migrate_restore,
};
use crate::cli::http_client::MantaClient;
use crate::common::{app_context::AppContext, authentication::get_api_token};
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta migrate` subcommands (nodes, vCluster backup/restore).
pub async fn handle_migrate(
  cli_migrate: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_migrate.subcommand() {
    Some(("nodes", m)) => {
      let dry_run: bool = m.get_flag("dry-run");
      let from_opt = m.get_one::<String>("from").map(String::as_str);
      let to: &str = m
        .get_one::<String>("to")
        .map(String::as_str)
        .context("The 'to' argument is mandatory")?;
      let xnames_string: &String = m
        .get_one("XNAMES")
        .context("The 'XNAMES' argument must have a value")?;

      // If --from is set, use just that group; otherwise fan out to every
      // group the token can access. The accessible-group list comes from
      // the manta server. Server-side `validate_hsm_group_access` then
      // re-checks each name in the resulting list.
      let from: Vec<String> = match from_opt.or(ctx.settings_hsm_group_name_opt)
      {
        Some(name) => vec![name.to_string()],
        None => {
          MantaClient::new(ctx.manta_server_url, ctx.site_name)?
            .get_available_groups(&token)
            .await?
        }
      };
      let to = vec![to.to_string()];

      migrate_nodes_between_hsm_groups::exec(
        ctx,
        &token,
        &to,
        &from,
        xnames_string,
        dry_run,
        false,
      )
      .await?;
    }
    Some(("vCluster", m)) => match m.subcommand() {
      Some(("backup", m)) => {
        let bos: Option<&String> = m.get_one("bos");
        let destination: Option<&String> = m.get_one("destination");
        let prehook: Option<&String> = m.get_one("pre-hook");
        let posthook: Option<&String> = m.get_one("post-hook");
        migrate_backup::exec(
          ctx,
          &token,
          bos.map(String::as_str),
          destination.map(String::as_str),
          prehook.map(String::as_str),
          posthook.map(String::as_str),
        )
        .await?;
      }
      Some(("restore", m)) => {
        let bos_file: Option<&String> = m.get_one("bos-file");
        let cfs_file: Option<&String> = m.get_one("cfs-file");
        let hsm_file: Option<&String> = m.get_one("hsm-file");
        let ims_file: Option<&String> = m.get_one("ims-file");
        let image_dir: Option<&String> = m.get_one("image-dir");
        let prehook: Option<&String> = m.get_one("pre-hook");
        let posthook: Option<&String> = m.get_one("post-hook");
        let overwrite: bool = m.get_flag("overwrite");
        migrate_restore::exec(
          ctx,
          &token,
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
      Some((other, _)) => {
        bail!("Unknown 'migrate vCluster' subcommand: {other}")
      }
      None => bail!("No 'migrate vCluster' subcommand provided"),
    },
    Some((other, _)) => bail!("Unknown 'migrate' subcommand: {other}"),
    None => bail!("No 'migrate' subcommand provided"),
  }
  Ok(())
}
