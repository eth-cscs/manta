//! Implements the `manta apply runtime-configuration nodes` command.
//!
//! Sets a CFS configuration as the runtime `desired_configuration` on
//! a set of nodes named by a hosts expression via
//! `PUT /api/v2/runtime-configuration`. Sibling of
//! [`super::runtime_configuration_group`] which takes a group name and
//! resolves its members first.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::ApplyRuntimeConfigurationRequest;

pub struct ExecParams<'a> {
  pub configuration_name: &'a str,
  pub hosts_expression: &'a str,
  /// When true, stage the runtime configuration without enabling CFS
  /// to apply it (`enabled: false` on the wire).
  pub disable: bool,
  /// When true, run server-side validations without persisting the
  /// CFS component PATCH.
  pub dry_run: bool,
}

/// Set the runtime CFS configuration on the nodes named by
/// `hosts_expression`.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built or when the
/// `apply_runtime_configuration` call fails (server-side validation:
/// bad hosts expression, unauthorized nodes, missing configuration).
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .apply_runtime_configuration(
      client.site_name(),
      &ApplyRuntimeConfigurationRequest {
        cfs_configuration_name: p.configuration_name.to_string(),
        hosts_expression: p.hosts_expression.to_string(),
        enabled: !p.disable,
        dry_run: Some(p.dry_run),
      },
    )
    .await
    .into_anyhow()
    .await?;
  println!("{}", serde_json::to_string_pretty(&result)?);
  Ok(())
}
