//! HTTP request/response bodies and CLI-built parameter structs for
//! the kernel-parameter endpoints (`/api/v1/kernel-parameters/*`).
//!
//! The internal `KernelParamOperation` enum used by the server's
//! kernel-parameter orchestration is not exposed here — it lives in
//! `crate::service::kernel_parameters` because it carries operational
//! logic (mutate, handles_sbps_images) rather than wire data.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Which kernel-parameter mutation to perform on
/// `POST /api/v1/kernel-parameters/apply`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum KernelParamOp {
  /// Merge new parameters into the existing set.
  Add,
  /// Replace the entire parameter set.
  Apply,
  /// Remove the named parameters from the existing set.
  Delete,
}

/// Request body for `POST /api/v1/kernel-parameters/apply`.
///
/// One of `xnames_expression` or `hsm_group` must be set (not both).
/// The chosen [`KernelParamOp`] determines what `params` means:
/// merge-keys, replace-set, or remove-set.
///
/// # Wire shape
///
/// ```json
/// {
///   "xnames_expression": "x3000c0s1b0n[0-3]",
///   "hsm_group": null,
///   "operation": "add",
///   "params": "console=ttyS0 nosmt",
///   "overwrite": false,
///   "project_sbps": true,
///   "dry_run": false
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyKernelParametersRequest {
  /// Hosts expression (xnames, NIDs, or hostlist notation); mutually
  /// exclusive with `hsm_group`.
  pub xnames_expression: Option<String>,
  /// Target HSM group; all members are resolved to xnames.
  pub hsm_group: Option<String>,
  /// Which mutation to perform: add, apply (replace), or delete.
  pub operation: KernelParamOp,
  /// Space-separated kernel parameter `key=value` pairs.
  pub params: String,
  /// Only relevant for the `Add` operation.
  #[serde(default)]
  pub overwrite: bool,
  /// Whether to project SBPS images (default `true`).
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  /// When true, return the computed changeset without applying it.
  #[serde(default)]
  pub dry_run: bool,
}

/// Request body for `POST /api/v1/kernel-parameters/add` (append mode).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddKernelParametersRequest {
  /// Space-separated kernel parameter `key=value` pairs to add.
  pub params: String,
  /// Hosts expression (xnames, NIDs, or hostlist notation); mutually
  /// exclusive with `hsm_group`.
  pub xnames_expression: Option<String>,
  /// Target HSM group; all members are resolved to xnames.
  pub hsm_group: Option<String>,
  /// When true, overwrite parameters that already exist.
  #[serde(default)]
  pub overwrite: bool,
  /// Whether to project SBPS images (default `true`).
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  /// When true, return the computed changeset without applying it.
  #[serde(default)]
  pub dry_run: bool,
}

/// Request body for `DELETE /api/v1/kernel-parameters`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteKernelParametersRequest {
  /// Space-separated parameter names (or `key=value` pairs) to
  /// remove.
  pub params: String,
  /// Hosts expression (xnames, NIDs, or hostlist notation); mutually
  /// exclusive with `hsm_group`.
  pub xnames_expression: Option<String>,
  /// Target HSM group; all members are resolved to xnames.
  pub hsm_group: Option<String>,
  /// When true, return the computed changeset without applying it.
  #[serde(default)]
  pub dry_run: bool,
}

fn default_true() -> bool {
  true
}

/// Typed parameters for fetching kernel boot parameters.
///
/// Precedence: `nodes` > `group_name` > `settings_group_name`.
/// At least one must resolve to a non-empty value.
pub struct GetKernelParametersParams {
  /// Group whose members' kernel parameters should be returned.
  pub group_name: Option<String>,
  /// Explicit comma-separated xnames; mutually exclusive with
  /// `group_name`.
  pub nodes: Option<String>,
  /// Operator default from `cli.toml`'s `parent_group_group`, used
  /// when neither `group_name` nor `nodes` is supplied.
  pub settings_group_name: Option<String>,
}

impl GetKernelParametersParams {
  /// Returns the effective group name, preferring the explicit
  /// `--group` flag and falling back to the operator default from
  /// `cli.toml`.
  pub fn effective_group(&self) -> Option<&str> {
    self
      .group_name
      .as_deref()
      .or(self.settings_group_name.as_deref())
  }
}
