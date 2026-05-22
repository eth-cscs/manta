//! Implements the `manta add node` command.

use anyhow::Result;
use std::path::PathBuf;

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::{app_context::AppContext, audit};

/// CLI adapter for `manta add node`.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  group: &str,
  enabled: bool,
  arch_opt: Option<String>,
  _hardware_file_path: Option<&PathBuf>,
  output_opt: Option<&str>,
) -> Result<()> {
  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .add_node(token, id, group, enabled, arch_opt)
    .await?;

  // Audit
  audit::maybe_send_audit(
    ctx.kafka_audit_opt,
    token,
    "add node",
    Some(serde_json::json!(id)),
    Some(serde_json::json!([])),
  )
  .await;

  action_result::print(
    &format!("Node '{id}' created and added to group '{group}'"),
    output_opt,
  )?;

  Ok(())
}
