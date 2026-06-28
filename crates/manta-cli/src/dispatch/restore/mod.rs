//! `manta restore` subcommands.
//!
//! Today only [`vcluster`] exists; it drives a cluster-data restore
//! via `POST /api/v1/migrate/restore`. Inverse of
//! [`super::backup`].

pub mod vcluster;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;

/// Dispatch `manta restore` subcommands.
///
/// # Errors
///
/// Returns an error when the auth token cannot be obtained, when no
/// subcommand is provided or the name is unknown, or when the leaf
/// handler fails.
pub async fn handle_restore(
  cli_restore: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_restore.subcommand() {
    Some(("vcluster", m)) => {
      vcluster::exec(
        ctx,
        &token,
        vcluster::ExecParams {
          bos_file: m.opt_str("bos-file"),
          cfs_file: m.opt_str("cfs-file"),
          hsm_file: m.opt_str("hsm-file"),
          ims_file: m.opt_str("ims-file"),
          image_dir: m.opt_str("image-dir"),
          prehook: m.opt_str("pre-hook"),
          posthook: m.opt_str("post-hook"),
          overwrite: m.get_flag("overwrite"),
          output: m.opt_str("output"),
          dry_run: m.get_flag("dry-run"),
        },
      )
      .await?;
    }
    Some((other, _)) => bail!("Unknown 'restore' subcommand: {other}"),
    None => bail!("No 'restore' subcommand provided"),
  }
  Ok(())
}
