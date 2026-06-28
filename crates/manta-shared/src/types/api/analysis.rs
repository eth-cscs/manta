//! Wire shape for `GET /api/v1/analysis/images`.
//!
//! Each `BackendSummary` is one row in an image-centric projection of
//! the four backend resource lists. See the field-population rules in
//! the implementation plan for how each column is derived.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One row of the backend-data summary, anchored on an IMS image.
///
/// Every IMS image visible to the caller produces exactly one row.
/// The row's `image_id` and `name` are always populated; the rest are
/// `Option<String>` and are filled in only when the corresponding
/// relation resolves.
///
/// See also [`super::configuration_analysis::ConfigurationAnalysis`]
/// for the parallel configuration-centric projection.
///
/// # Wire shape
///
/// ```json
/// {
///   "image_id": "0a1b2c3d-...",
///   "name": "compute-cos-2.5",
///   "image_created": "2026-05-12T10:14:22Z",
///   "configuration_name": "cos-2.5",
///   "safe_to_delete": false
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BackendSummary {
  /// IMS image id (`Image.id`). Row anchor.
  pub image_id: String,
  /// IMS image name (`Image.name`).
  pub name: String,
  /// IMS image `created` timestamp (`Image.created`).
  pub image_created: Option<String>,
  /// CFS configuration the image was built with
  /// (`Image.configuration`).
  pub configuration_name: Option<String>,
  /// `true` if no BSS boot-parameter record references this image as
  /// its boot image. An image referenced by BSS is currently booting
  /// (or scheduled to boot) at least one node, so deleting it would
  /// break that node's next boot.
  pub safe_to_delete: bool,
}
