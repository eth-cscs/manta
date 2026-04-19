use anyhow::{Context, Error, bail};

use crate::common::{self, app_context::AppContext, audit};
use crate::service::group;

/// CLI adapter for `manta add group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  auth_token: &str,
  label: &str,
  description: Option<&str>,
  hosts_expression_opt: Option<&str>,
  assume_yes: bool,
  dryrun: bool,
) -> Result<(), Error> {
  let (grp, xname_vec_opt) = group::prepare_add_group(
    &ctx.infra,
    auth_token,
    label,
    description,
    hosts_expression_opt,
  )
  .await?;

  if !common::user_interaction::confirm(
    &format!(
      "This operation will create the group below:\n{}\nPlease confirm to proceed",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    ),
    assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    println!(
      "Dryrun mode: The group below would be created:\n{}",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    );
    return Ok(());
  }

  group::create_group(&ctx.infra, auth_token, grp).await?;

  println!("Group '{}' created", label);

  // Audit
  audit::maybe_send_audit(
    ctx.cli.kafka_audit_opt,
    auth_token,
    format!("Create Group '{}'", label),
    Some(serde_json::json!(xname_vec_opt.unwrap_or_default())),
    Some(serde_json::json!(label)),
  )
  .await;

  Ok(())
}
