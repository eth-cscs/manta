//! Parameters for `GET /groups/hardware` (and the deprecated
//! `/hardware-clusters` alias) and `GET /hardware-nodes-list`.

/// Typed parameters for fetching cluster hardware inventory.
pub struct GetHardwareClusterParams {
  /// Cluster group name to inventory; `None` falls back to the
  /// operator default.
  pub group_name: Option<String>,
  /// Operator default from `cli.toml`'s `hsm_group`, used when
  /// `group_name` is absent.
  pub settings_hsm_group_name: Option<String>,
}

impl GetHardwareClusterParams {
  /// Returns the effective group name, preferring the explicit
  /// positional argument and falling back to the operator default
  /// from `cli.toml`.
  pub fn effective_group(&self) -> Option<&str> {
    self
      .group_name
      .as_deref()
      .or(self.settings_hsm_group_name.as_deref())
  }
}

/// Typed parameters for fetching hardware inventory for a list of nodes.
#[derive(Debug)]
pub struct GetHardwareNodesListParams {
  /// Hosts expression (xnames, NIDs, or hostlist notation).
  pub host_expression: String,
}
