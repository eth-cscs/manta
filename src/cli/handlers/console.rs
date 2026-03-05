use crate::cli::commands::{
  console_cfs_session_image_target_ansible, console_node,
};
use crate::common::app_context::AppContext;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use std::io::IsTerminal;

/// Dispatch `manta console` subcommands (node,
/// target-ansible).
pub async fn handle_console(
  cli_console: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  if let Some(cli_console_node) = cli_console.subcommand_matches("node") {
    if !std::io::stdout().is_terminal() {
      bail!("This command needs to run in interactive mode");
    }
    let site = ctx
      .configuration
      .sites
      .get(&ctx.configuration.site)
      .context("Site not found in configuration")?;
    let xname = cli_console_node
      .get_one::<String>("XNAME")
      .context("The 'XNAME' argument must have a value")?;
    let k8s_details = site
      .k8s
      .as_ref()
      .context("k8s section not found in configuration")?;
    console_node::exec(ctx.backend, ctx.site_name, xname, k8s_details).await?;
  } else if let Some(cli_console_target_ansible) =
    cli_console.subcommand_matches("target-ansible")
  {
    if !std::io::stdout().is_terminal() {
      bail!("This command needs to run in interactive mode");
    }
    let site = ctx
      .configuration
      .sites
      .get(&ctx.configuration.site)
      .context("Site not found in configuration")?;
    let session_name = cli_console_target_ansible
      .get_one::<String>("SESSION_NAME")
      .context("The 'SESSION_NAME' argument must have a value")?;
    let k8s_details = site
      .k8s
      .as_ref()
      .context("k8s section not found in configuration")?;
    console_cfs_session_image_target_ansible::exec(
      ctx.backend,
      ctx.site_name,
      ctx.settings_hsm_group_name_opt,
      ctx.shasta_base_url,
      ctx.shasta_root_cert,
      session_name,
      k8s_details,
    )
    .await?;
  }
  Ok(())
}
