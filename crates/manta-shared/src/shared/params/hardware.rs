//! Parameters for `GET /hardware-clusters` and `GET /hardware-nodes-list`.

/// Typed parameters for fetching cluster hardware inventory.
pub struct GetHardwareClusterParams {
  pub hsm_group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Typed parameters for fetching hardware inventory for a list of nodes.
#[derive(Debug)]
pub struct GetHardwareNodesListParams {
  /// Comma-separated xnames.
  pub xnames: String,
}
