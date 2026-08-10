//! Implements the `manta apply runtime-configuration group` command.
//!
//! Sets a CFS configuration as the runtime `desired_configuration` on
//! every member of an HSM group.
//!
//! The server's `PUT /runtime-configuration` endpoint takes a hosts
//! expression (xnames / NIDs / hostlist notation), not a group name,
//! so this fetches the group's members first and forwards them as a
//! comma-separated xname list — same pattern as
//! [`super::boot_group`].

use anyhow::{Error, anyhow, bail};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::ApplyRuntimeConfigurationRequest;

pub struct ExecParams<'a> {
  pub configuration_name: &'a str,
  pub group_name: &'a str,
  /// When true, stage the runtime configuration without enabling CFS
  /// to apply it (`enabled: false` on the wire).
  pub disable: bool,
  /// When true, run server-side validations without persisting the
  /// CFS component PATCH.
  pub dry_run: bool,
}

/// Set the runtime CFS configuration on every member of `group_name`.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built, when the
/// `get_groups` lookup fails, when the named group is absent or has
/// no members, or when the `apply_runtime_configuration` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;

  let groups = client
    .openapi
    .get_groups(Some(p.group_name), client.site_name())
    .await
    .into_anyhow()
    .await?;
  let group = groups
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("HSM group '{}' not found", p.group_name))?;
  let xnames = group.members.and_then(|m| m.ids).unwrap_or_default();
  if xnames.is_empty() {
    bail!("HSM group '{}' has no members", p.group_name);
  }

  let result = client
    .openapi
    .apply_runtime_configuration(
      client.site_name(),
      &ApplyRuntimeConfigurationRequest {
        cfs_configuration_name: p.configuration_name.to_string(),
        hosts_expression: xnames.join(","),
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
