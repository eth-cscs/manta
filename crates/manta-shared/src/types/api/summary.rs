//! Wire shape for `GET /api/v1/summary`.
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
  /// CFS session whose `status.artifacts[*].result_id` contains
  /// this `image_id`. If multiple sessions match, the first one in
  /// name order wins.
  pub session_name: Option<String>,
  /// Echo of `image_id` whenever `session_name` is `Some(_)`. Carried
  /// explicitly so the row is self-describing.
  pub session_result_id: Option<String>,
  /// That session's own `configuration.name`. Usually equals
  /// `configuration_name` but they can drift if the image was
  /// re-tagged after the session completed.
  pub session_configuration_name: Option<String>,
  /// BOS session template whose `boot_sets[*].path` references this
  /// `image_id`. If multiple templates match, the first one in name
  /// order wins.
  pub bos_sessiontemplate: Option<String>,
  /// Echo of `image_id` whenever `bos_sessiontemplate` is `Some(_)`.
  /// Carried explicitly so the row is self-describing.
  pub bos_sessiontemplate_boot_image: Option<String>,
}
