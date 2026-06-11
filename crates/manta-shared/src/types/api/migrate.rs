//! Wire types for the `POST /api/v1/migrate/*` endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/migrate/nodes`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MigrateNodesRequest {
  /// Destination HSM group names to move nodes into.
  pub target_hsm_names: Vec<String>,
  /// Source HSM group names the nodes currently belong to.
  pub parent_hsm_names: Vec<String>,
  /// Node-set expression (xnames, NIDs, or hostlist notation)
  /// selecting which nodes to migrate.
  pub hosts_expression: String,
  /// When true, validate the migration plan without modifying any
  /// group membership.
  #[serde(default)]
  pub dry_run: bool,
  /// Create the target HSM group if it does not already exist.
  #[serde(default)]
  pub create_hsm_group: bool,
}

/// Request body for `POST /api/v1/migrate/backup`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MigrateBackupRequest {
  /// BOS session template name (or filter) to back up.
  pub bos: Option<String>,
  /// Filesystem path where backup files will be written.
  pub destination: Option<String>,
}

/// Request body for `POST /api/v1/migrate/restore`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MigrateRestoreRequest {
  /// Path to the BOS session template backup file.
  pub bos_file: Option<String>,
  /// Path to the CFS configuration backup file.
  pub cfs_file: Option<String>,
  /// Path to the HSM group backup file.
  pub hsm_file: Option<String>,
  /// Path to the IMS image metadata backup file.
  pub ims_file: Option<String>,
  /// Directory containing the image layer tarballs.
  pub image_dir: Option<String>,
  /// When true, overwrite existing resources that conflict with the
  /// backup.
  #[serde(default)]
  pub overwrite: bool,
}
