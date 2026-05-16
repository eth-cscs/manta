//! Parameters for `GET /groups`.

/// Typed parameters for fetching HSM groups.
#[derive(Debug)]
pub struct GetGroupParams {
  pub group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}
