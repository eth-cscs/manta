//! Parameters for `GET /configurations`.

use chrono::NaiveDateTime;

/// Typed parameters for fetching CFS configurations.
pub struct GetConfigurationParams {
  pub name: Option<String>,
  pub pattern: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub since: Option<NaiveDateTime>,
  pub until: Option<NaiveDateTime>,
  pub limit: Option<u8>,
}
