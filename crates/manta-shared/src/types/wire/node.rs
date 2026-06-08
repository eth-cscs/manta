//! Wire types for `POST /api/v1/nodes` (register a new HSM
//! component).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/nodes`.
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
