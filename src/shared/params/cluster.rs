//! Parameters for `GET /clusters`.

/// Typed parameters for fetching cluster node details.
pub struct GetClusterParams {
  pub hsm_group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub status_filter: Option<String>,
}
