//! Implements the `manta get analysis image` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;

/// CLI adapter for `manta get analysis image`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let output_opt = cli_args
    .get_one::<String>("output")
    .map_or("table", String::as_str);

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let rows = client
    .openapi
    .get_image_analysis(client.site_name())
    .await
    .into_anyhow()?;

  output::analysis_image::print(&rows, output_opt)?;
  Ok(())
}
