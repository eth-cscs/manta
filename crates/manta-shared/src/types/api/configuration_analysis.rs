//! Wire shape for `GET /api/v1/analysis/configurations`.
//!
//! One row per CFS configuration, carrying its last-updated timestamp
//! and a `safe_to_delete` verdict derived from cross-resource
//! dependencies (BSS boot parameters and CFS components).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One row of the configuration-deletion-safety analysis.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfigurationAnalysis {
  /// CFS configuration name (`CfsConfigurationResponse.name`).
  pub name: String,
  /// `last_updated` timestamp from CFS.
  pub last_updated: String,
  /// `true` if no CFS component lists this configuration as its
  /// `desired_config` **and** no image built from this configuration
  /// is referenced by any BSS boot-parameter record. A row that is
  /// not safe is referenced by at least one of those two surfaces.
  pub safe_to_delete: bool,
}
