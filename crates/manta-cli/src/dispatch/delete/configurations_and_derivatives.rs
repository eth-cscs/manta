//! Implements the `manta delete configurations` command, which also
//! cascades to derived artifacts (images, session templates, sessions)
//! — hence the longer filename.

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;
use chrono::NaiveDateTime;

pub struct ExecParams<'a> {
  pub configuration_name_pattern: Option<&'a str>,
  pub since: Option<NaiveDateTime>,
  pub until: Option<NaiveDateTime>,
  pub output: Option<&'a str>,
}

/// Delete CFS configurations and their derived artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), anyhow::Error> {
  let since_str = p.since.map(|d| d.to_string());
  let until_str = p.until.map(|d| d.to_string());
  let result = MantaClient::from_app_ctx(ctx)?
    .delete_configurations(
      token,
      p.configuration_name_pattern,
      since_str.as_deref(),
      until_str.as_deref(),
      false,
    )
    .await?;
  action_result::print_with_data("Configurations deleted", &result, p.output)?;
  Ok(())
}
