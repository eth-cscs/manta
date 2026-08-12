//! HTTP request/response bodies for the runtime-configuration endpoint
//! (`PUT /v2/runtime-configuration`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `PUT /v2/runtime-configuration`.
///
/// Assigns a CFS configuration as the `desired_configuration` on every
/// CFS component matching `hosts_expression`, and sets the component's
/// `enabled` flag. Idempotent: the same request repeated leaves the
/// nodes in the same state.
///
/// # Wire shape
///
/// ```json
/// {
///   "cfs_configuration_name": "base-mc-compute-config-cscs-26.3.0-pa",
///   "hosts_expression": "x8000c1s0b1n[0-3]",
///   "enabled": true,
///   "dry_run": false
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyRuntimeConfigurationRequest {
  /// CFS configuration name to write as each component's
  /// `desired_configuration`. Must already exist in CFS.
  pub cfs_configuration_name: String,
  /// Hosts expression (xnames, NIDs, or hostlist notation) naming
  /// the target nodes. HSM group names are not accepted here.
  pub hosts_expression: String,
  /// Value to set on each targeted CFS component's `enabled` flag.
  /// `true` lets CFS reconfigure on its next pass; `false` stages
  /// the desired configuration without triggering CFS.
  pub enabled: bool,
  /// When true, run all validations (hosts resolve, access check,
  /// configuration existence) but skip the final CFS component
  /// PATCH. The response still echoes the resolved xname list and
  /// the configuration/enabled values that *would* have been
  /// written, with `applied: false` and `dry_run: true`.
  #[serde(default)]
  pub dry_run: bool,
}
