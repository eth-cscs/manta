//! Parameters for `GET /images`.

use chrono::NaiveDateTime;

/// Typed parameters for fetching IMS images.
pub struct GetImagesParams {
  /// Exact IMS image ID; returns just that image when set.
  pub id: Option<String>,
  /// Glob pattern matched against image name; applied server-side.
  /// Invalid glob returns HTTP 400.
  pub pattern: Option<String>,
  /// Lower-bound timestamp (images created at or after this point).
  /// Inclusive.
  pub since: Option<NaiveDateTime>,
  /// Upper-bound timestamp (images created at or before this point).
  /// Inclusive.
  pub until: Option<NaiveDateTime>,
  /// Cap on the number of images returned (the newest N; listed oldest
  /// first, newest last).
  pub limit: Option<u8>,
}
