//! Implements the `manta get analysis configuration` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;

/// CLI adapter for `manta get analysis configuration`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let output_opt = cli_args
    .get_one::<String>("output")
    .map_or("table", String::as_str);
  let only_safe = cli_args.get_flag("only-safe-to-delete");
  let only_unsafe = cli_args.get_flag("only-unsafe-to-delete");

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let rows = client
    .openapi
    .get_configuration_analysis(client.site_name())
    .await
    .into_anyhow()?;

  let rows: Vec<_> = rows
    .into_iter()
    .filter(|r| {
      (!only_safe || r.safe_to_delete) && (!only_unsafe || !r.safe_to_delete)
    })
    .collect();

  output::analysis_configuration::print(&rows, output_opt)?;
  Ok(())
}
