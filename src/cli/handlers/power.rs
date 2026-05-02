use crate::cli::commands::power_common::{self, PowerAction};
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta power` subcommands (on, off, reset —
/// each targeting nodes or clusters).
pub async fn handle_power(
  cli_power: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  match cli_power.subcommand() {
    Some(("on", m)) => match m.subcommand() {
      Some(("cluster", m)) => {
        let hsm_group_name_arg = m
          .get_one::<String>("CLUSTER_NAME")
          .context("The 'cluster name' argument must have a value")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output: &str = m
          .get_one::<String>("output")
          .context("'output' argument is required")?;
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
        let xname_requested: &str = m
          .get_one::<String>("VALUE")
          .context("The 'xnames' argument must have values")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output: &str = m
          .get_one::<String>("output")
          .context("'output' argument is required")?;
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
        let hsm_group_name_arg = m
          .get_one::<String>("CLUSTER_NAME")
          .context("The 'cluster name' argument must have a value")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let output: &str = m
          .get_one::<String>("output")
          .context("'output' argument is required")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        power_common::exec_cluster(
          ctx,
          PowerAction::Off,
          hsm_group_name_arg,
          *force,
          assume_yes,
          output,
          &token,
        )
        .await?;
      }
      Some(("nodes", m)) => {
        let xname_requested: &str = m
          .get_one::<String>("VALUE")
          .context("The 'xnames' argument must have values")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output: &str = m
          .get_one::<String>("output")
          .context("'output' argument is required")?;
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
        let hsm_group_name_arg = m
          .get_one::<String>("CLUSTER_NAME")
          .context("The 'cluster name' argument must have a value")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let output: &str = m
          .get_one::<String>("output")
          .context("'output' argument is required")?;
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
        let xname_requested: &str = m
          .get_one::<String>("VALUE")
          .context("The 'xnames' argument must have values")?;
        let force = m
          .get_one::<bool>("graceful")
          .context("The 'graceful' argument must have a value")?;
        let assume_yes: bool = m.get_flag("assume-yes");
        let output: &str = m
          .get_one::<String>("output")
          .context("'output' argument is required")?;
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
