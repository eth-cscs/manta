//! Routes `manta power *` subcommands to their exec functions.

use crate::commands::power::common::{
  self as power_common, PowerAction, PowerOpts,
};
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

/// Dispatch a single `power on group/cluster` invocation. Shared
/// between the canonical `group` arm and the deprecated `cluster`
/// arm so both stay in lockstep.
async fn dispatch_power_on_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  power_common::exec_cluster(
    ctx,
    token,
    PowerOpts {
      action: PowerAction::On,
      target: m.req_str("CLUSTER_NAME")?,
      force: false,
      no_wait: m.get_flag("no-wait"),
      assume_yes: m.get_flag("assume-yes"),
      output: m.req_str("output")?,
    },
  )
  .await
}

/// Shared dispatch for `power off group/cluster`.
async fn dispatch_power_off_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let graceful = m
    .get_one::<bool>("graceful")
    .context("The 'graceful' argument must have a value")?;
  power_common::exec_cluster(
    ctx,
    token,
    PowerOpts {
      action: PowerAction::Off,
      target: m.req_str("CLUSTER_NAME")?,
      force: !graceful,
      no_wait: m.get_flag("no-wait"),
      assume_yes: m.get_flag("assume-yes"),
      output: m.req_str("output")?,
    },
  )
  .await
}

/// Shared dispatch for `power reset group/cluster`.
async fn dispatch_power_reset_group(
  m: &ArgMatches,
  ctx: &AppContext<'_>,
  token: &str,
) -> Result<(), Error> {
  let force = m
    .get_one::<bool>("graceful")
    .context("The 'graceful' argument must have a value")?;
  power_common::exec_cluster(
    ctx,
    token,
    PowerOpts {
      action: PowerAction::Reset,
      target: m.req_str("CLUSTER_NAME")?,
      force: *force,
      no_wait: m.get_flag("no-wait"),
      assume_yes: m.get_flag("assume-yes"),
      output: m.req_str("output")?,
    },
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
        power_common::exec_nodes(
          ctx,
          &token,
          PowerOpts {
            action: PowerAction::On,
            target: m.req_str("VALUE")?,
            force: false,
            no_wait: m.get_flag("no-wait"),
            assume_yes: m.get_flag("assume-yes"),
            output: m.req_str("output")?,
          },
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
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        power_common::exec_nodes(
          ctx,
          &token,
          PowerOpts {
            action: PowerAction::Off,
            target: m.req_str("VALUE")?,
            force: !graceful,
            no_wait: m.get_flag("no-wait"),
            assume_yes: m.get_flag("assume-yes"),
            output: m.req_str("output")?,
          },
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
        let graceful = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        power_common::exec_nodes(
          ctx,
          &token,
          PowerOpts {
            action: PowerAction::Reset,
            target: m.req_str("VALUE")?,
            force: !graceful,
            no_wait: m.get_flag("no-wait"),
            assume_yes: m.get_flag("assume-yes"),
            output: m.req_str("output")?,
          },
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
