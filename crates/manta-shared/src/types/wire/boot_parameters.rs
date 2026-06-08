//! Wire types for `POST /api/v1/boot-config` (apply a combined boot
//! configuration to a set of nodes).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/boot-config`.
///
/// Applies a combined boot configuration (image + runtime config +
/// kernel parameters) to the nodes named by `hosts_expression`. The
/// field is a hosts expression — xnames, NIDs, or hostlist notation;
/// HSM group names are not accepted here (resolve them client-side
/// first if needed).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyBootConfigRequest {
  /// Hosts expression (xnames, NIDs, or hostlist notation) naming
  /// the target nodes.
  pub hosts_expression: String,
  /// IMS image ID to set as the boot image.
  pub boot_image_id: Option<String>,
  /// CFS configuration name associated with the boot image; the
  /// server resolves the most recent image built against this
  /// configuration when `boot_image_id` is absent.
  pub boot_image_configuration: Option<String>,
  /// Kernel command-line parameters to apply.
  pub kernel_parameters: Option<String>,
  /// CFS configuration to assign as the runtime desired-config.
  pub runtime_configuration: Option<String>,
  /// When true, return the computed changeset without persisting it.
  #[serde(default)]
  pub dry_run: bool,
}
