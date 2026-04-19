use anyhow::Error;

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
  group::delete_group(&ctx.infra, token, label, force).await?;

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
