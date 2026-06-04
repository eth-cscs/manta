//! Routes `manta restore *` subcommands to their exec functions.

use crate::commands::restore::vcluster as restore_vcluster;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

/// Dispatch `manta restore` subcommands.
pub async fn handle_restore(
  cli_restore: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_restore.subcommand() {
    Some(("vcluster", m)) => {
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
    Some((other, _)) => bail!("Unknown 'restore' subcommand: {other}"),
    None => bail!("No 'restore' subcommand provided"),
  }
  Ok(())
}
