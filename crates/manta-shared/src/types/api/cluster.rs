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

impl GetClusterParams {
  /// Returns the effective group name, preferring the explicit
  /// positional argument and falling back to the operator default from
  /// `cli.toml`.
  pub fn effective_group(&self) -> Option<&str> {
    self
      .group_name
      .as_deref()
      .or(self.settings_group_name.as_deref())
  }
}
