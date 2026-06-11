//! HTTP request/response bodies and CLI-built parameter structs for
//! the BOS session template endpoints (`/api/v1/templates`,
//! `/api/v1/templates/{name}/sessions`).

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

/// Typed parameters for fetching BOS session templates.
pub struct GetTemplateParams {
  /// Exact template name.
  pub name: Option<String>,
  /// Group whose associated templates should be returned.
  pub group_name: Option<String>,
  /// Operator default from `cli.toml`'s `parent_hsm_group`, used
  /// as a fallback for `group_name`.
  pub settings_group_name: Option<String>,
  /// Cap on the number of templates returned (most recent first).
  pub limit: Option<u8>,
}

/// Parameters for applying a BOS session template.
pub struct ApplyTemplateParams {
  /// Optional explicit name for the created BOS session;
  /// auto-generated when absent.
  pub bos_session_name: Option<String>,
  /// Name of an existing BOS session template to instantiate.
  pub bos_sessiontemplate_name: String,
  /// Operation to perform: `"boot"`, `"reboot"`, or `"shutdown"`.
  pub bos_session_operation: String,
  /// Ansible-style limit expression scoping which template nodes
  /// are targeted (xnames, NIDs, groups, or roles).
  pub limit: String,
  /// When true, include nodes marked as disabled in the hardware
  /// state manager.
  pub include_disabled: bool,
}
