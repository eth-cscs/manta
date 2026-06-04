//! Parameters for `GET /groups`.

/// Typed parameters for fetching HSM groups.
#[derive(Debug)]
pub struct GetGroupParams {
  /// Exact group name to fetch; returns all groups when `None`.
  pub group_name: Option<String>,
  /// Operator default from `~/.config/manta/cli.toml`'s
  /// `parent_hsm_group`; used to scope results when `group_name` is
  /// absent but a configured default exists.
  pub settings_hsm_group_name: Option<String>,
}
