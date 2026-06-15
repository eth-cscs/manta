//! Wire shape for `GET /api/v1/configurations`.
//!
//! One row per CFS configuration, carrying the full
//! `CfsConfigurationResponse` and a `safe_to_delete` verdict derived
//! from CFS components (a configuration is unsafe iff some component
//! lists it as `desired_config`).

use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One row of the configuration-deletion-safety analysis.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfigurationAnalysis {
  /// Full CFS configuration record as returned by CFS. `CfsConfigurationResponse`
  /// lives in `manta-backend-dispatcher` (third-party, no ToSchema), so the
  /// OpenAPI schema falls back to `serde_json::Value`.
  #[schema(value_type = serde_json::Value)]
  pub configuration: CfsConfigurationResponse,
  /// `true` if no CFS component lists this configuration as its
  /// `desired_config`. The verdict is components-only; deletion may
  /// still be unsafe if a BSS-referenced image was built from this
  /// configuration — that fuller check isn't surfaced on this wire
  /// shape.
  pub safe_to_delete: bool,
}
