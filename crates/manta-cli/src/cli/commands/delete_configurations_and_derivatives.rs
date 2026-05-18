//! Implements the `manta delete configurations` command.

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use chrono::NaiveDateTime;

/// Delete CFS configurations and their derived artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  configuration_name_pattern_opt: Option<&str>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  _assume_yes: bool,
) -> Result<(), anyhow::Error> {
  let server_url = ctx.manta_server_url;
  let since_str = since_opt.map(|d| d.to_string());
  let until_str = until_opt.map(|d| d.to_string());
  let result = MantaClient::new(server_url, ctx.site_name)?
    .delete_configurations(
      token,
      configuration_name_pattern_opt,
      since_str.as_deref(),
      until_str.as_deref(),
      false,
    )
    .await?;
  println!(
    "{}",
    serde_json::to_string_pretty(&result).unwrap_or_default()
  );
  Ok(())
}
