//! HTTP request/response bodies and CLI-built parameter structs for
//! the node endpoints (`/api/v1/nodes`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/nodes`.
///
/// Paired with [`super::responses::AddNodeResponse`] on success.
///
/// # Wire shape
///
/// ```json
/// {
///   "id": "x3000c0s1b0n0",
///   "group": "alps",
///   "enabled": true,
///   "arch": "X86"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNodeRequest {
  /// Physical location ID (xname) of the node, e.g. `x3000c0s1b0n0`.
  pub id: String,
  /// Initial HSM group the node belongs to.
  pub group: String,
  /// Whether to register the node as enabled. Defaults to `false`
  /// (disabled) per serde's default for `bool`; the CLI's
  /// `manta add node` flips the polarity via `--disabled`.
  #[serde(default)]
  pub enabled: bool,
  /// Optional architecture tag: `"X86"`, `"ARM"`, or `"Other"`.
  pub arch: Option<String>,
}

/// Typed parameters for fetching node details.
pub struct GetNodesParams {
  /// Comma-separated xnames, NIDs, or hostlist expression
  /// (e.g. `x3000c0s1b0n[0-3]`).
  pub host_expression: String,
  /// When true, also return nodes sharing a power supply with any
  /// requested node.
  pub include_siblings: bool,
  /// Optional power-status filter (e.g. `ON`, `OFF`, `READY`).
  pub status_filter: Option<String>,
}
