//! Routes `manta migrate *` subcommands to their exec functions.

use crate::commands::migrate::nodes as migrate_nodes;
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta migrate` subcommands. Today only `nodes` exists —
/// the `vCluster {backup,restore}` aliases have been removed; use
/// `manta backup vcluster` / `manta restore vcluster` instead.
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
    Some((other, _)) => bail!("Unknown 'migrate' subcommand: {other}"),
    None => bail!("No 'migrate' subcommand provided"),
  }
  Ok(())
}
