//! Parameters for `GET /templates` and `POST /templates/{name}/sessions`.

/// Typed parameters for fetching BOS session templates.
pub struct GetTemplateParams {
  /// Exact template name.
  pub name: Option<String>,
  /// HSM group whose associated templates should be returned.
  pub hsm_group: Option<String>,
  /// Operator default from `cli.toml`'s `parent_hsm_group`, used
  /// as a fallback for `hsm_group`.
  pub settings_hsm_group_name: Option<String>,
  /// Cap on the number of templates returned (most recent first).
  pub limit: Option<u8>,
}

/// Parameters for applying a BOS session template.
pub struct ApplyTemplateParams {
  /// Optional explicit name for the created BOS session;
  /// auto-generated when absent.
  pub bos_session_name: Option<String>,
  /// Name of an existing BOS session template to instantiate.
  pub bos_sessiontemplate_name: String,
  /// Operation to perform: `"boot"`, `"reboot"`, or `"shutdown"`.
  pub bos_session_operation: String,
  /// Ansible-style limit expression scoping which template nodes
  /// are targeted (xnames, NIDs, groups, or roles).
  pub limit: String,
  /// When true, include nodes marked as disabled in the hardware
  /// state manager.
  pub include_disabled: bool,
}
