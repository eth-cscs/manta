//! Parameters for `GET /sessions`.

/// Typed parameters for fetching CFS sessions.
pub struct GetSessionParams {
  /// HSM group whose sessions should be returned.
  pub hsm_group: Option<String>,
  /// Filter to sessions whose `ansible_limit` mentions any of these
  /// xnames. Empty means "no xname filter".
  pub xnames: Vec<String>,
  /// Lower-bound session age expressed as a duration string
  /// (e.g. `"1h"`, `"2d"`).
  pub min_age: Option<String>,
  /// Upper-bound session age expressed as a duration string.
  pub max_age: Option<String>,
  /// Session type filter: `"image"` or `"runtime"`.
  pub session_type: Option<String>,
  /// Status filter: `"pending"`, `"running"`, or `"complete"`.
  pub status: Option<String>,
  /// Exact session name.
  pub name: Option<String>,
  /// Cap on the number of sessions returned (most recent first).
  pub limit: Option<u8>,
}
