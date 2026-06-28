//! `manta migrate` subcommands.

pub mod nodes;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta migrate` subcommands. Today only `nodes` exists —
/// the `vCluster {backup,restore}` aliases have been removed; use
/// `manta backup vcluster` / `manta restore vcluster` instead.
///
/// # Errors
///
/// Returns an error when the auth token cannot be obtained, when
/// `--to` or the `XNAMES` positional is missing, when no subcommand
/// is provided or the name is unknown, when the
/// `get_available_groups` fallback fails (only when `--from` is also
/// unset), or when the [`nodes::exec`] call fails.
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

      // Build the client once; reuse it for both the optional
      // accessible-groups lookup and the migrate_nodes call in nodes::exec.
      let client = MantaClient::from_app_ctx(ctx, Some(&token))?;

      // If --from is set, use just that group; otherwise fan out to every
      // group the token can access. The accessible-group list comes from
      // the manta server. Server-side `validate_user_group_vec_access`
      // then re-checks each name in the resulting list.
      let from: Vec<String> = match from_opt.or(ctx.settings_group_name_opt) {
        Some(name) => vec![name.to_string()],
        None => client
          .openapi
          .get_available_groups(client.site_name())
          .await
          .into_anyhow()?,
      };
      let to = vec![to.to_string()];

      nodes::exec(
        &client,
        nodes::ExecParams {
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
