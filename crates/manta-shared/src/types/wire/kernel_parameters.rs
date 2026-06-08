//! Wire types for the `POST /api/v1/kernel-parameters/*` and
//! `DELETE /api/v1/kernel-parameters` endpoints.

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
