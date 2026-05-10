//! Implements the `manta delete group` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::common::audit;
use crate::common::kafka::Kafka;
use crate::service::group;

/// CLI adapter for `manta delete group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  label: &str,
  force: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  if let Some(server_url) = ctx.infra.manta_server_url {
    MantaClient::new(server_url, ctx.infra.site_name)?
      .delete_group(token, label, force)
      .await?;
  } else {
    group::delete_group(&ctx.infra, token, label, force).await?;
  }

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
