//! Parameters for `GET /images`.

/// Typed parameters for fetching IMS images.
pub struct GetImagesParams {
  pub id: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub limit: Option<u8>,
}
