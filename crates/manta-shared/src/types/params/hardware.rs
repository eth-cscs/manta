//! Parameters for `GET /hardware-clusters` and `GET /hardware-nodes-list`.

/// Typed parameters for fetching cluster hardware inventory.
pub struct GetHardwareClusterParams {
  /// Cluster group name to inventory; `None` falls back to the
  /// operator default.
  pub group_name: Option<String>,
  /// Operator default from `cli.toml`'s `parent_hsm_group`, used when
  /// `hsm_group_name` is absent.
  pub settings_hsm_group_name: Option<String>,
}

/// Typed parameters for fetching hardware inventory for a list of nodes.
#[derive(Debug)]
pub struct GetHardwareNodesListParams {
  /// Comma-separated xnames.
  pub host_expression: String,
}
