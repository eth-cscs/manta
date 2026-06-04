//! Routes `manta migrate *` subcommands to their exec functions.

use crate::cli::commands::migrate::{
  backup as migrate_backup, nodes_between_groups as migrate_nodes_between_hsm_groups,
  restore as migrate_restore,
};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use crate::cli::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::cli::common::app_context::AppContext;

/// Dispatch `manta migrate` subcommands (nodes, vCluster backup/restore).
pub async fn handle_migrate(
  cli_migrate: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_migrate.subcommand() {
    Some(("nodes", m)) => {
      let dry_run: bool = m.get_flag("dry-run");
      let from_opt = m.opt_str("from");
      let to = m.req_str("to")?;
      let xnames_string = m.req_str("XNAMES")?;
      let output_opt = m.opt_str("output");

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
        output_opt,
      )
      .await?;
    }
    Some(("vCluster", m)) => match m.subcommand() {
      Some(("backup", m)) => {
        eprintln!(
          "warning: 'manta migrate vCluster backup' is deprecated; \
           use 'manta backup vcluster' instead.",
        );
        let output_opt = m.opt_str("output");
        migrate_backup::exec(
          ctx,
          &token,
          m.opt_str("bos"),
          m.opt_str("destination"),
          m.opt_str("pre-hook"),
          m.opt_str("post-hook"),
          output_opt,
        )
        .await?;
      }
      Some(("restore", m)) => {
        eprintln!(
          "warning: 'manta migrate vCluster restore' is deprecated; \
           use 'manta restore vcluster' instead.",
        );
        let overwrite: bool = m.get_flag("overwrite");
        let output_opt = m.opt_str("output");
        migrate_restore::exec(
          ctx,
          &token,
          m.opt_str("bos-file"),
          m.opt_str("cfs-file"),
          m.opt_str("hsm-file"),
          m.opt_str("ims-file"),
          m.opt_str("image-dir"),
          m.opt_str("pre-hook"),
          m.opt_str("post-hook"),
          overwrite,
          output_opt,
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
