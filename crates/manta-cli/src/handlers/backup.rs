//! Routes `manta backup *` subcommands to their exec functions.

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::dispatch::backup::vcluster as backup_vcluster;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta backup` subcommands.
pub async fn handle_backup(
  cli_backup: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_backup.subcommand() {
    Some(("vcluster", m)) => {
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
    Some((other, _)) => bail!("Unknown 'backup' subcommand: {other}"),
    None => bail!("No 'backup' subcommand provided"),
  }
  Ok(())
}
