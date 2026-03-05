use crate::cli::commands;
use crate::common::app_context::AppContext;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch the `manta log` command to stream Kubernetes
/// pod logs for a CFS session or a node's console.
pub async fn handle_log(
  cli_log: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let user_input = cli_log
    .get_one::<String>("VALUE")
    .context("The 'VALUE' argument is mandatory")?;
  let timestamps = cli_log.get_flag("timestamps");
  let site = ctx
    .configuration
    .sites
    .get(&ctx.configuration.site)
    .context("Site not found in configuration")?;
  let k8s_details = site
    .k8s
    .as_ref()
    .context("k8s section not found in configuration")?;
  match commands::log::exec(
    ctx.backend,
    ctx.site_name,
    ctx.shasta_base_url,
    ctx.shasta_root_cert,
    user_input,
    timestamps,
    k8s_details,
  )
  .await
  {
    Ok(_) => {
      println!("Log streaming ended");
      Ok(())
    }
    Err(e) => bail!("{e}"),
  }
}
