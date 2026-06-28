//! HTTP request/response bodies and CLI-built parameter structs for
//! the boot-parameter endpoints (`/api/v1/boot-config` and
//! `/api/v1/boot-parameters`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/boot-config`.
///
/// Applies a combined boot configuration (image + runtime config +
/// kernel parameters) to the nodes named by `hosts_expression`. The
/// field is a hosts expression — xnames, NIDs, or hostlist notation;
/// HSM group names are not accepted here (resolve them client-side
/// first if needed).
///
/// At least one of `boot_image_id`, `boot_image_configuration`,
/// `kernel_parameters`, or `runtime_configuration` should be set;
/// `null` fields are left unchanged on the targeted nodes.
///
/// # Wire shape
///
/// ```json
/// {
///   "hosts_expression": "x3000c0s1b0n[0-3]",
///   "boot_image_id": "0a1b2c3d-...",
///   "boot_image_configuration": null,
///   "kernel_parameters": "console=ttyS0 nosmt",
///   "runtime_configuration": "cos-2.5",
///   "dry_run": false
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyBootConfigRequest {
  /// Hosts expression (xnames, NIDs, or hostlist notation) naming
  /// the target nodes.
  pub hosts_expression: String,
  /// IMS image ID to set as the boot image. Mutually exclusive with
  /// `boot_image_configuration`: set one or the other, not both.
  pub boot_image_id: Option<String>,
  /// CFS configuration name associated with the boot image; the
  /// server resolves the most recent image built against this
  /// configuration when `boot_image_id` is absent.
  pub boot_image_configuration: Option<String>,
  /// Kernel command-line parameters to apply.
  pub kernel_parameters: Option<String>,
  /// CFS configuration to assign as the runtime desired-config.
  pub runtime_configuration: Option<String>,
  /// When true, return the computed changeset without persisting it.
  #[serde(default)]
  pub dry_run: bool,
}

/// Typed parameters for fetching boot parameters.
///
/// Precedence: `host_expression` > `group_name` >
/// `settings_group_name`. The first one set is used; the others are
/// ignored. If all three are unset the request fails.
pub struct GetBootParametersParams {
  /// Group whose members' boot parameters should be returned.
  pub group_name: Option<String>,
  /// Hosts expression (xnames, NIDs, or hostlist notation); mutually
  /// exclusive with `group_name`.
  pub host_expression: Option<String>,
  /// Operator default from `cli.toml`'s `hsm_group`, used
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
