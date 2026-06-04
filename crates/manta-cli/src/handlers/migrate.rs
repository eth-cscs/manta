//! Routes `manta migrate *` subcommands to their exec functions.

use crate::commands::backup::vcluster as backup_vcluster;
use crate::commands::migrate::nodes as migrate_nodes;
use crate::commands::restore::vcluster as restore_vcluster;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

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

      migrate_nodes::exec(
        ctx,
        &token,
        migrate_nodes::ExecParams {
          target_groups: &to,
          parent_groups: &from,
          hosts_expression: xnames_string,
          dry_run,
          create_group: false,
          output: output_opt,
        },
      )
      .await?;
    }
    Some(("vCluster", m)) => match m.subcommand() {
      Some(("backup", m)) => {
        eprintln!(
          "warning: 'manta migrate vCluster backup' is deprecated; \
           use 'manta backup vcluster' instead.",
        );
        backup_vcluster::exec(
          ctx,
          &token,
          backup_vcluster::ExecParams {
            bos: m.opt_str("bos"),
            destination: m.opt_str("destination"),
            prehook: m.opt_str("pre-hook"),
            posthook: m.opt_str("post-hook"),
            output: m.opt_str("output"),
          },
        )
        .await?;
      }
      Some(("restore", m)) => {
        eprintln!(
          "warning: 'manta migrate vCluster restore' is deprecated; \
           use 'manta restore vcluster' instead.",
        );
        restore_vcluster::exec(
          ctx,
          &token,
          restore_vcluster::ExecParams {
            bos_file: m.opt_str("bos-file"),
            cfs_file: m.opt_str("cfs-file"),
            hsm_file: m.opt_str("hsm-file"),
            ims_file: m.opt_str("ims-file"),
            image_dir: m.opt_str("image-dir"),
            prehook: m.opt_str("pre-hook"),
            posthook: m.opt_str("post-hook"),
            overwrite: m.get_flag("overwrite"),
            output: m.opt_str("output"),
          },
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
