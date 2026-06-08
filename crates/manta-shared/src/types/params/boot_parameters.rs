//! Parameters for `GET` and `PUT` on `/boot-parameters`.

/// Typed parameters for fetching boot parameters.
pub struct GetBootParametersParams {
  /// Group whose members' boot parameters should be returned.
  pub group_name: Option<String>,
  /// Hosts expression (xnames, NIDs, or hostlist notation); mutually
  /// exclusive with `group_name`.
  pub host_expression: Option<String>,
  /// Operator default from `cli.toml`'s `parent_hsm_group`, used
  /// when neither `group_name` nor `host_expression` is supplied.
  pub settings_group_name: Option<String>,
}

/// Typed parameters for updating boot parameters.
#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct UpdateBootParametersParams {
  /// Target node xnames.
  pub hosts: Vec<String>,
  /// Node IDs corresponding to `hosts` (optional alternate identifier).
  pub nids: Option<Vec<u32>>,
  /// MAC addresses corresponding to `hosts` (optional alternate identifier).
  pub macs: Option<Vec<String>>,
  /// Kernel command-line parameters string.
  pub params: String,
  /// S3 path to the kernel image.
  pub kernel: String,
  /// S3 path to the initrd image.
  pub initrd: String,
}
