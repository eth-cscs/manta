//! Implements the `manta delete group` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::common::audit;
use crate::common::kafka::Kafka;

/// CLI adapter for `manta delete group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  label: &str,
  force: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  MantaClient::new(server_url, ctx.infra.site_name)?
    .delete_group(token, label, force)
    .await?;

  println!("Group '{}' deleted", label);

  // Audit
  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("Delete Group '{}'", label),
    None,
    Some(serde_json::json!(label)),
  )
  .await;

  Ok(())
}
