//! Routes `manta backup *` subcommands to their exec functions.

use crate::cli::commands::migrate::backup as migrate_backup;
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::cli::common::app_context::AppContext;

/// Dispatch `manta backup` subcommands.
pub async fn handle_backup(
  cli_backup: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_backup.subcommand() {
    Some(("vcluster", m)) => {
      migrate_backup::exec(
        ctx,
        &token,
        migrate_backup::ExecParams {
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
