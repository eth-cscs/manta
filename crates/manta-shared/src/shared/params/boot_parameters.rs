//! Parameters for `GET` and `PUT` on `/boot-parameters`.

/// Typed parameters for fetching boot parameters.
pub struct GetBootParametersParams {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
  pub settings_hsm_group_name: Option<String>,
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
