//! Routes `manta power *` subcommands to their exec functions.

use crate::cli::commands::power_common::{self, PowerAction};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;

/// Dispatch `manta power` subcommands (on, off, reset —
/// each targeting nodes or clusters).
pub async fn handle_power(
  cli_power: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_power.subcommand() {
    Some(("on", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output = m.req_str("output")?;
        power_common::exec_cluster(
          ctx,
          PowerAction::On,
          hsm_group_name_arg,
          false,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some(("nodes", m)) => {
        let xname_requested = m.req_str("VALUE")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output = m.req_str("output")?;
        power_common::exec_nodes(
          ctx,
          PowerAction::On,
          xname_requested,
          false,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'power on' subcommand: {other}"),
      None => bail!("No 'power on' subcommand provided"),
    },
    Some(("off", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let output = m.req_str("output")?;
        let assume_yes: bool = m.get_flag("assume-yes");

        let force = !graceful;

        power_common::exec_cluster(
          ctx,
          PowerAction::Off,
          hsm_group_name_arg,
          force,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some(("nodes", m)) => {
        let xname_requested = m.req_str("VALUE")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output = m.req_str("output")?;
        power_common::exec_nodes(
          ctx,
          PowerAction::Off,
          xname_requested,
          *force,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'power off' subcommand: {other}"),
      None => bail!("No 'power off' subcommand provided"),
    },
    Some(("reset", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let output = m.req_str("output")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        power_common::exec_cluster(
          ctx,
          PowerAction::Reset,
          hsm_group_name_arg,
          *force,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some(("nodes", m)) => {
        let xname_requested = m.req_str("VALUE")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output = m.req_str("output")?;
        power_common::exec_nodes(
          ctx,
          PowerAction::Reset,
          xname_requested,
          *force,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some((other, _)) => bail!("Unknown 'power reset' subcommand: {other}"),
      None => bail!("No 'power reset' subcommand provided"),
    },
    Some((other, _)) => bail!("Unknown 'power' subcommand: {other}"),
    None => bail!("No 'power' subcommand provided"),
  }
  Ok(())
}
