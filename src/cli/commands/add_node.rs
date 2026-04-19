use anyhow::Result;
use std::path::PathBuf;

use crate::common::{app_context::AppContext, audit};
use crate::service::node;

/// CLI adapter for `manta add node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  group: &str,
  enabled: bool,
  arch_opt: Option<String>,
  hardware_file_path: Option<&PathBuf>,
) -> Result<()> {
  node::add_node(
    &ctx.infra,
    token,
    id,
    group,
    enabled,
    arch_opt,
    hardware_file_path,
  )
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
