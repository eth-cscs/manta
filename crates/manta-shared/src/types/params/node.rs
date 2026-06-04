//! Parameters for `GET /nodes`.

/// Typed parameters for fetching node details.
pub struct GetNodesParams {
  /// Comma-separated xnames, NIDs, or hostlist expression
  /// (e.g. `x3000c0s1b0n[0-3]`).
  pub xname: String,
  /// When true, also return nodes sharing a power supply with any
  /// requested node.
  pub include_siblings: bool,
  /// Optional power-status filter (e.g. `ON`, `OFF`, `READY`).
  pub status_filter: Option<String>,
}
