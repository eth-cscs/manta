//! Parameters for `GET /configurations`.

use chrono::NaiveDateTime;

/// Typed parameters for fetching CFS configurations.
pub struct GetConfigurationParams {
  /// Exact configuration name.
  pub name: Option<String>,
  /// Glob pattern matched against configuration names; mutually
  /// exclusive with `name`.
  pub pattern: Option<String>,
  /// Group whose associated configurations should be returned.
  pub group_name: Option<String>,
  /// Operator default from `cli.toml`'s `hsm_group`, used
  /// as a fallback for `group_name`.
  pub settings_hsm_group_name: Option<String>,
  /// Lower-bound timestamp (configurations last updated at or after
  /// this point).
  pub since: Option<NaiveDateTime>,
  /// Upper-bound timestamp (configurations last updated at or before
  /// this point).
  pub until: Option<NaiveDateTime>,
  /// Cap on the number of configurations returned (most recent first).
  pub limit: Option<u8>,
}

impl GetConfigurationParams {
  /// Returns the effective group name, preferring the explicit
  /// `--group` flag and falling back to the operator default from
  /// `cli.toml`.
  pub fn effective_group(&self) -> Option<&str> {
    self
      .group_name
      .as_deref()
      .or(self.settings_hsm_group_name.as_deref())
  }
}
