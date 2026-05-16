//! Parameters for `GET /nodes`.

/// Typed parameters for fetching node details.
pub struct GetNodesParams {
  pub xname: String,
  pub include_siblings: bool,
  pub status_filter: Option<String>,
}
