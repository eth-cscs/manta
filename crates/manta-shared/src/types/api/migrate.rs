//! Wire types for the `POST /api/v1/migrate/*` endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/migrate/nodes`.
///
/// Paired with [`super::responses::MigrateNodesResponse`] on success.
/// `target_hsm_names` and `parent_hsm_names` are length-matched: each
/// `parent_hsm_names[i]` donates members to `target_hsm_names[i]`.
///
/// # Wire shape
///
/// ```json
/// {
///   "target_hsm_names": ["zinal"],
///   "parent_hsm_names": ["alps"],
///   "hosts_expression": "x3000c0s1b0n[0-3]",
///   "dry_run": false,
///   "create_hsm_group": false
/// }
/// ```
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
///
/// Paired with [`super::responses::CompletedResponse`] on success.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MigrateBackupRequest {
  /// BOS session template name (or filter) to back up.
  pub bos: Option<String>,
  /// Filesystem path where backup files will be written.
  pub destination: Option<String>,
}

/// Request body for `POST /api/v1/migrate/restore`.
///
/// Paired with [`super::responses::CompletedResponse`] on success.
/// Every `*_file` field is optional independently — supplying only
/// `cfs_file` restores CFS configurations and leaves BOS, HSM, IMS,
/// and image layers untouched.
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
