//! Parameters for `GET /templates` and `POST /templates/{name}/sessions`.

/// Typed parameters for fetching BOS session templates.
pub struct GetTemplateParams {
  pub name: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub limit: Option<u8>,
}

/// Parameters for applying a BOS session template.
pub struct ApplyTemplateParams {
  pub bos_session_name: Option<String>,
  pub bos_sessiontemplate_name: String,
  pub bos_session_operation: String,
  pub limit: String,
  pub include_disabled: bool,
}
