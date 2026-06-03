//! Routes `manta restore *` subcommands to their exec functions.

use crate::cli::commands::migrate_restore;
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::cli::common::app_context::AppContext;

/// Dispatch `manta restore` subcommands.
pub async fn handle_restore(
  cli_restore: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_restore.subcommand() {
    Some(("vcluster", m)) => {
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
    Some((other, _)) => bail!("Unknown 'restore' subcommand: {other}"),
    None => bail!("No 'restore' subcommand provided"),
  }
  Ok(())
}
