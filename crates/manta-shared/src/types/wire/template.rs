//! Wire types for `POST /api/v1/templates/{name}/sessions` (create a
//! BOS session from a BOS session template).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// BOS session operation to run against the template's node list.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BosOperation {
  /// Boot nodes that are currently off.
  Boot,
  /// Reboot (power-cycle) nodes.
  Reboot,
  /// Shut down nodes.
  Shutdown,
}

impl BosOperation {
  /// Wire-level string form expected by the backend (`"boot"` /
  /// `"reboot"` / `"shutdown"`).
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Boot => "boot",
      Self::Reboot => "reboot",
      Self::Shutdown => "shutdown",
    }
  }
}

/// Request body for `POST /api/v1/templates/{name}/sessions`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostTemplateSessionRequest {
  /// BOS operation to run (boot, reboot, or shutdown).
  pub operation: BosOperation,
  /// Ansible limit expression restricting which template nodes are
  /// targeted.
  pub limit: String,
  /// Optional explicit name for the BOS session; auto-generated when
  /// absent.
  pub session_name: Option<String>,
  /// When true, include nodes marked as disabled.
  #[serde(default)]
  pub include_disabled: bool,
  /// When true, validate the session parameters without creating a
  /// BOS session.
  #[serde(default)]
  pub dry_run: bool,
}
