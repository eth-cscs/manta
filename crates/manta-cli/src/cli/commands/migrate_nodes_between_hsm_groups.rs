//! Implements the `manta migrate nodes` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use manta_shared::common::{app_context::AppContext, audit};

/// Move nodes between HSM groups with validation.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name_vec: &[String],
  parent_hsm_name_vec: &[String],
  hosts_expression: &str,
  dry_run: bool,
  create_hsm_group: bool,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .migrate_nodes(
      token,
      target_hsm_name_vec,
      parent_hsm_name_vec,
      hosts_expression,
      dry_run,
      create_hsm_group,
    )
    .await?;
  if dry_run {
    println!("dry-run enabled, changes not persisted.");
  }
  println!(
    "{}",
    serde_json::to_string_pretty(&result).unwrap_or_default()
  );

  audit::maybe_send_audit(
    ctx.kafka_audit_opt,
    token,
    format!(
      "Migrate nodes from {parent_hsm_name_vec:?} to {target_hsm_name_vec:?}"
    ),
    None,
    Some(serde_json::json!(vec![
      parent_hsm_name_vec,
      target_hsm_name_vec
    ])),
  )
  .await;

  Ok(())
}
