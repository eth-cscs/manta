//! Parameters for `GET /clusters`.

/// Typed parameters for fetching cluster node details.
pub struct GetClusterParams {
  /// Cluster group name to query; falls back to
  /// `settings_group_name` when absent.
  pub group_name: Option<String>,
  /// Operator default from `cli.toml`'s `hsm_group`.
  pub settings_group_name: Option<String>,
  /// Optional power-status filter (e.g. `OFF`, `ON`, `READY`,
  /// `STANDBY`, `PENDING`, `FAILED`, `CONFIGURED`); returns all nodes
  /// when absent.
  pub status_filter: Option<String>,
}
