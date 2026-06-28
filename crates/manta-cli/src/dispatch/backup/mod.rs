//! `manta backup` subcommands.
//!
//! Today only [`vcluster`] exists; it drives a cluster-data backup via
//! `POST /api/v1/migrate/backup`. Paired with the symmetric
//! [`super::restore`] commands.

pub mod vcluster;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta backup` subcommands.
///
/// # Errors
///
/// Returns an error when the auth token cannot be obtained, when no
/// subcommand is provided or the name is unknown, or when the leaf
/// handler fails.
pub async fn handle_backup(
  cli_backup: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_backup.subcommand() {
    Some(("vcluster", m)) => {
      vcluster::exec(
        ctx,
        &token,
        vcluster::ExecParams {
          bos: m.opt_str("bos"),
          destination: m.opt_str("destination"),
          prehook: m.opt_str("pre-hook"),
          posthook: m.opt_str("post-hook"),
          output: m.opt_str("output"),
        },
      )
      .await?;
    }
    Some((other, _)) => bail!("Unknown 'backup' subcommand: {other}"),
    None => bail!("No 'backup' subcommand provided"),
  }
  Ok(())
}
