//! Parameters for `GET /groups`.

/// Typed parameters for fetching HSM groups.
#[derive(Debug)]
pub struct GetGroupParams {
  /// Exact group name to fetch; returns all groups when `None`.
  pub group_name: Option<String>,
  /// Operator default from `~/.config/manta/cli.toml`'s
  /// `group_name`; used to scope results when `group_name` is
  /// absent but a configured default exists.
  pub settings_group_name: Option<String>,
}
