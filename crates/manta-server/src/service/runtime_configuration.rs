//! Business logic for `PUT /api/v1/runtime-configuration`.
//!
//! Sets a CFS configuration as the `desired_configuration` on every
//! CFS component matching a hosts expression, and writes the
//! component's `enabled` flag in the same call. This is a narrow
//! subset of [`super::boot_parameters::persist_boot_config`] — the
//! latter also touches BSS boot parameters; this one only writes CFS
//! component state.

use manta_backend_dispatcher::{
  error::Error, interfaces::cfs::CfsTrait as _,
};

use crate::server::common::app_context::InfraContext;

use super::{authorization::validate_user_group_members_access, node_ops};

/// Assign `cfs_configuration_name` as the desired runtime config on
/// every node resolved from `hosts_expression`, setting each
/// component's `enabled` flag to `enabled`.
///
/// Returns the sorted, deduped xname list that was written (or would
/// have been written, in dry-run mode).
///
/// Ordering:
/// 1. Cheap validation of non-empty inputs.
/// 2. Resolve `hosts_expression` to xnames (backend call).
/// 3. Enforce access — the caller must be able to reach every xname
///    (backend call).
/// 4. Verify the CFS configuration exists (backend call). Runs *after*
///    the access check so unauthorized callers cannot probe config
///    existence by watching this endpoint's status codes.
/// 5. Patch each CFS component's `desired_configuration` and
///    `enabled` — skipped when `dry_run` is true.
///
/// # Errors
///
/// - [`Error::BadRequest`] — empty inputs, or `hosts_expression`
///   resolves to zero nodes.
/// - [`Error::InvalidPattern`] / [`Error::InvalidNodeId`] — malformed
///   `hosts_expression` (surfaced by
///   [`node_ops::from_user_hosts_expression_to_xname_vec`]).
/// - [`Error::Unauthorized`] — caller cannot reach one or more xnames.
/// - [`Error::NotFound`] — CFS configuration name does not exist.
/// - Backend errors from the final CFS component PATCH.
pub(crate) async fn apply_runtime_configuration(
  infra: &InfraContext<'_>,
  token: &str,
  cfs_configuration_name: &str,
  hosts_expression: &str,
  enabled: bool,
  dry_run: bool,
) -> Result<Vec<String>, Error> {
  if cfs_configuration_name.trim().is_empty() {
    return Err(Error::BadRequest(
      "cfs_configuration_name must not be empty".into(),
    ));
  }
  if hosts_expression.trim().is_empty() {
    return Err(Error::BadRequest(
      "hosts_expression must not be empty".into(),
    ));
  }

  let xnames = node_ops::from_user_hosts_expression_to_xname_vec(
    infra,
    token,
    hosts_expression,
    false,
  )
  .await?;

  if xnames.is_empty() {
    return Err(Error::BadRequest(format!(
      "hosts_expression '{hosts_expression}' resolved to zero nodes"
    )));
  }

  validate_user_group_members_access(infra, token, &xnames).await?;

  let configuration_name_owned = cfs_configuration_name.to_string();
  let configs = infra
    .backend
    .get_configuration(token, Some(&configuration_name_owned))
    .await?;
  if configs.is_empty() {
    return Err(Error::NotFound(format!(
      "CFS configuration '{cfs_configuration_name}'"
    )));
  }

  if dry_run {
    tracing::info!(
      "dry_run: skipping CFS component PATCH for {} nodes",
      xnames.len()
    );
    return Ok(xnames);
  }

  infra
    .backend
    .update_runtime_configuration(
      token,
      &xnames,
      cfs_configuration_name,
      enabled,
    )
    .await?;

  Ok(xnames)
}
