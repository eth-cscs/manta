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
    Some((other, _)) => bail!("Unknown 'backup' subcommand: {other}"),
    None => bail!("No 'backup' subcommand provided"),
  }
  Ok(())
}
