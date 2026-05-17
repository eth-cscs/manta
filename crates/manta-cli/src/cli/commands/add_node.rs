//! Implements the `manta add node` command.

use anyhow::Result;
use std::path::PathBuf;

use crate::cli::http_client::MantaClient;
use crate::common::{app_context::AppContext, audit};

/// CLI adapter for `manta add node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  group: &str,
  enabled: bool,
  arch_opt: Option<String>,
  _hardware_file_path: Option<&PathBuf>,
) -> Result<()> {
  let server_url = ctx.cli.manta_server_url;
  MantaClient::new(server_url, ctx.infra.site_name)?
    .add_node(token, id, group, enabled, arch_opt)
    .await?;

  // Audit
  audit::maybe_send_audit(
    ctx.cli.kafka_audit_opt,
    token,
    "add node",
    Some(serde_json::json!(id)),
    Some(serde_json::json!([])),
  )
  .await;

  println!("Node '{}' created and added to group '{}'", id, group);

  Ok(())
}
