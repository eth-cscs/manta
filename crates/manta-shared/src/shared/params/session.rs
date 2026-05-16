//! Parameters for `GET /sessions`.

/// Typed parameters for fetching CFS sessions.
pub struct GetSessionParams {
  pub hsm_group: Option<String>,
  pub xnames: Vec<String>,
  pub min_age: Option<String>,
  pub max_age: Option<String>,
  pub session_type: Option<String>,
  pub status: Option<String>,
  pub name: Option<String>,
  pub limit: Option<u8>,
}
