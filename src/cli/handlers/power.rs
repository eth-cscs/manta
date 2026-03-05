use crate::cli::commands::power_common::{self, PowerAction};
use crate::common::app_context::AppContext;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta power` subcommands (on, off, reset —
/// each targeting nodes or clusters).
pub async fn handle_power(
  cli_power: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(cli_power_on) = cli_power.subcommand_matches("on") {
    if let Some(cli_power_on_cluster) =
      cli_power_on.subcommand_matches("cluster")
    {
      let hsm_group_name_arg = cli_power_on_cluster
        .get_one::<String>("CLUSTER_NAME")
        .context("The 'cluster name' argument must have a value")?;
      let assume_yes: bool = cli_power_on_cluster.get_flag("assume-yes");
      let output: &str = cli_power_on_cluster
        .get_one::<String>("output")
        .context("'output' argument is required")?;
      power_common::exec_cluster(
        ctx,
        PowerAction::On,
        hsm_group_name_arg,
        false,
        assume_yes,
        output,
      )
      .await?;
    } else if let Some(cli_power_on_node) =
      cli_power_on.subcommand_matches("nodes")
    {
      let xname_requested: &str = cli_power_on_node
        .get_one::<String>("VALUE")
        .context("The 'xnames' argument must have values")?;
      let assume_yes: bool = cli_power_on_node.get_flag("assume-yes");
      let output: &str = cli_power_on_node
        .get_one::<String>("output")
        .context("'output' argument is required")?;
      power_common::exec_nodes(
        ctx,
        PowerAction::On,
        xname_requested,
        false,
        assume_yes,
        output,
      )
      .await?;
    } else {
      bail!("Unknown 'power on' subcommand");
    }
  } else if let Some(cli_power_off) = cli_power.subcommand_matches("off") {
    if let Some(cli_power_off_cluster) =
      cli_power_off.subcommand_matches("cluster")
    {
      let hsm_group_name_arg = cli_power_off_cluster
        .get_one::<String>("CLUSTER_NAME")
        .context("The 'cluster name' argument must have a value")?;
      let force = cli_power_off_cluster
        .get_one::<bool>("graceful")
        .context("The 'graceful' argument must have a value")?;
      let output: &str = cli_power_off_cluster
        .get_one::<String>("output")
        .context("'output' argument is required")?;
      let assume_yes: bool = cli_power_off_cluster.get_flag("assume-yes");
      power_common::exec_cluster(
        ctx,
        PowerAction::Off,
        hsm_group_name_arg,
        *force,
        assume_yes,
        output,
      )
      .await?;
    } else if let Some(cli_power_off_node) =
      cli_power_off.subcommand_matches("nodes")
    {
      let xname_requested: &str = cli_power_off_node
        .get_one::<String>("VALUE")
        .context("The 'xnames' argument must have values")?;
      let force = cli_power_off_node
        .get_one::<bool>("graceful")
        .context("The 'graceful' argument must have a value")?;
      let assume_yes: bool = cli_power_off_node.get_flag("assume-yes");
      let output: &str = cli_power_off_node
        .get_one::<String>("output")
        .context("'output' argument is required")?;
      power_common::exec_nodes(
        ctx,
        PowerAction::Off,
        xname_requested,
        *force,
        assume_yes,
        output,
      )
      .await?;
    } else {
      bail!("Unknown 'power off' subcommand");
    }
  } else if let Some(cli_power_reset) = cli_power.subcommand_matches("reset") {
    if let Some(cli_power_reset_cluster) =
      cli_power_reset.subcommand_matches("cluster")
    {
      let hsm_group_name_arg = cli_power_reset_cluster
        .get_one::<String>("CLUSTER_NAME")
        .context(
          "The 'cluster name' argument must have \
             a value",
        )?;
      let force = cli_power_reset_cluster
        .get_one::<bool>("graceful")
        .context("The 'graceful' argument must have a value")?;
      let output: &str = cli_power_reset_cluster
        .get_one::<String>("output")
        .context("'output' argument is required")?;
      let assume_yes: bool = cli_power_reset_cluster.get_flag("assume-yes");
      power_common::exec_cluster(
        ctx,
        PowerAction::Reset,
        hsm_group_name_arg,
        *force,
        assume_yes,
        output,
      )
      .await?;
    } else if let Some(cli_power_reset_node) =
      cli_power_reset.subcommand_matches("nodes")
    {
      let xname_requested: &str = cli_power_reset_node
        .get_one::<String>("VALUE")
        .context("The 'xnames' argument must have values")?;
      let force = cli_power_reset_node
        .get_one::<bool>("graceful")
        .context("The 'graceful' argument must have a value")?;
      let assume_yes: bool = cli_power_reset_node.get_flag("assume-yes");
      let output: &str = cli_power_reset_node
        .get_one::<String>("output")
        .context("'output' argument is required")?;
      power_common::exec_nodes(
        ctx,
        PowerAction::Reset,
        xname_requested,
        *force,
        assume_yes,
        output,
      )
      .await?;
    } else {
      bail!("Unknown 'power reset' subcommand");
    }
  } else {
    bail!("Unknown 'power' subcommand");
  }
  Ok(())
}
