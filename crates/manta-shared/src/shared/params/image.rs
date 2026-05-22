//! Parameters for `GET /images`.

/// Typed parameters for fetching IMS images.
pub struct GetImagesParams {
  /// Exact IMS image ID; returns just that image when set.
  pub id: Option<String>,
  /// HSM group to filter by (images associated with this group).
  pub hsm_group: Option<String>,
  /// Operator default from `cli.toml`'s `parent_hsm_group`, used
  /// as a fallback for `hsm_group`.
  pub settings_hsm_group_name: Option<String>,
  /// Cap on the number of images returned (most recent first).
  pub limit: Option<u8>,
}
