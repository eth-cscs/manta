//! Implements the `manta migrate nodes` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::MigrateNodesRequest;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub target_groups: &'a [String],
  pub parent_groups: &'a [String],
  pub hosts_expression: &'a str,
  pub dry_run: bool,
  pub create_group: bool,
  pub output: Option<&'a str>,
}

/// Move nodes between HSM groups with validation.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .migrate_nodes(
      client.site_name(),
      &MigrateNodesRequest {
        target_hsm_names: p.target_groups.to_vec(),
        parent_hsm_names: p.parent_groups.to_vec(),
        hosts_expression: p.hosts_expression.to_string(),
        dry_run: Some(p.dry_run),
        create_hsm_group: Some(p.create_group),
      },
    )
    .await
    .into_anyhow()?;
  let message = if p.dry_run {
    "dry-run enabled, changes not persisted."
  } else {
    "Nodes migrated."
  };
  action_result::print_with_data(message, &result, p.output)?;

  Ok(())
}
