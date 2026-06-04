//! Routes the `manta upgrade` command to its exec function.
//!
//! Unlike every other handler, this one does NOT call
//! `get_api_token(ctx)` — `manta upgrade` talks to GitHub releases, not
//! to the manta server, so there's no token to bootstrap.

use anyhow::Error;
use clap::ArgMatches;

use crate::dispatch::upgrade;
use crate::common::app_context::AppContext;

/// Dispatch the `manta upgrade` command.
pub async fn handle_upgrade(
  cli_upgrade: &ArgMatches,
  _ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let check_only = cli_upgrade.get_flag("check");
  let dry_run = cli_upgrade.get_flag("dry-run");
  let assume_yes = cli_upgrade.get_flag("assume-yes");
  let output_owned: Option<String> = cli_upgrade.get_one::<String>("output").cloned();

  // The upgrade flow uses blocking I/O (reqwest::blocking +
  // xz2 + tar + fs::rename); off-load to a blocking thread to
  // keep the Tokio runtime free.
  tokio::task::spawn_blocking(move || {
    upgrade::exec(check_only, dry_run, assume_yes, output_owned.as_deref())
  })
  .await
  .map_err(|e| anyhow::anyhow!("upgrade task panicked: {e}"))?
}
