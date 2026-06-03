//! Routes `manta power *` subcommands to their exec functions.

use crate::cli::commands::power_common::{self, PowerAction};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use crate::cli::common::app_context::AppContext;

/// Dispatch a single `power on group/cluster` invocation. Shared
/// between the canonical `group` arm and the deprecated `cluster`
/// arm so both stay in lockstep.
async fn dispatch_power_on_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
  let assume_yes: bool = m.get_flag("assume-yes");
  let no_wait: bool = m.get_flag("no-wait");
  let output = m.req_str("output")?;
  power_common::exec_cluster(
    ctx,
    PowerAction::On,
    hsm_group_name_arg,
    false,
    no_wait,
    assume_yes,
    output,
    token,
  )
  .await
}

/// Shared dispatch for `power off group/cluster`.
async fn dispatch_power_off_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
  let graceful = m
    .get_one::<bool>("graceful")
    .context("The 'graceful' argument must have a value")?;
  let output = m.req_str("output")?;
  let assume_yes: bool = m.get_flag("assume-yes");
  let no_wait: bool = m.get_flag("no-wait");
  power_common::exec_cluster(
    ctx,
    PowerAction::Off,
    hsm_group_name_arg,
    !graceful,
    no_wait,
    assume_yes,
    output,
    token,
  )
  .await
}

/// Shared dispatch for `power reset group/cluster`.
async fn dispatch_power_reset_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let hsm_group_name_arg = m.req_str("CLUSTER_NAME")?;
  let force = m
    .get_one::<bool>("graceful")
    .context("The 'graceful' argument must have a value")?;
  let output = m.req_str("output")?;
  let assume_yes: bool = m.get_flag("assume-yes");
  let no_wait: bool = m.get_flag("no-wait");
  power_common::exec_cluster(
    ctx,
    PowerAction::Reset,
    hsm_group_name_arg,
    *force,
    no_wait,
    assume_yes,
    output,
    token,
  )
  .await
}

fn warn_cluster_deprecated(action: &str) {
  eprintln!(
    "warning: 'manta power {action} cluster' is deprecated; \
     use 'manta power {action} group' instead.",
  );
}

/// Dispatch `manta power` subcommands (on, off, reset —
/// each targeting nodes or groups).
pub async fn handle_power(
  cli_power: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_power.subcommand() {
    Some(("on", m)) => match m.subcommand() {
      Some(("group", m)) => dispatch_power_on_group(m, ctx, &token).await?,
      Some(("cluster", m)) => {
        warn_cluster_deprecated("on");
        dispatch_power_on_group(m, ctx, &token).await?;
      }
      Some(("nodes", m)) => {
        let xname_requested = m.req_str("VALUE")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let no_wait: bool = m.get_flag("no-wait");
        let output = m.req_str("output")?;
        power_common::exec_nodes(
          ctx,
          PowerAction::On,
          xname_requested,
          false,
          no_wait,
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
      Some(("group", m)) => dispatch_power_off_group(m, ctx, &token).await?,
      Some(("cluster", m)) => {
        warn_cluster_deprecated("off");
        dispatch_power_off_group(m, ctx, &token).await?;
      }
      Some(("nodes", m)) => {
        let xname_requested = m.req_str("VALUE")?;
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let force = !graceful;
        let assume_yes: bool = m.get_flag("assume-yes");
        let no_wait: bool = m.get_flag("no-wait");
        let output = m.req_str("output")?;
        power_common::exec_nodes(
          ctx,
          PowerAction::Off,
          xname_requested,
          force,
          no_wait,
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
      Some(("group", m)) => dispatch_power_reset_group(m, ctx, &token).await?,
      Some(("cluster", m)) => {
        warn_cluster_deprecated("reset");
        dispatch_power_reset_group(m, ctx, &token).await?;
      }
      Some(("nodes", m)) => {
        let xname_requested = m.req_str("VALUE")?;
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let force = !graceful;
        let assume_yes: bool = m.get_flag("assume-yes");
        let no_wait: bool = m.get_flag("no-wait");
        let output = m.req_str("output")?;
        power_common::exec_nodes(
          ctx,
          PowerAction::Reset,
          xname_requested,
          force,
          no_wait,
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
