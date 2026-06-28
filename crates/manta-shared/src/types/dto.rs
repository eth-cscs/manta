//! Wire response types shared by both binaries.
//!
//! The CLI (in `manta_cli::http_client::*` and `manta_cli::output::*`)
//! deserializes responses from the manta-server using these types;
//! the server serializes them back over HTTP via the service layer.
//!
//! `NodeDetails` is mirrored locally (rather than re-exporting from
//! `csm-rs`) so `manta-shared` — and therefore `manta-cli` — does not
//! transitively depend on `csm-rs`. No in-process conversion is
//! needed: the type boundary is HTTP, and the JSON wire shape is
//! byte-identical between `csm_rs::node::types::NodeDetails` and the
//! mirror below (same field names, no `#[serde(rename)]`), so the CLI
//! just deserializes the response directly into this struct.
//!
//! The remaining re-exports come from the lightweight
//! `manta-backend-dispatcher` crate (types + traits only, no csm-rs
//! / ochami-rs deps). Mirroring those too is a separate, optional
//! follow-up.

pub use manta_backend_dispatcher::types::{
  Group, NodeSummary,
  bos::session_template::BosSessionTemplate,
  bss::BootParameters,
  cfs::{
    cfs_configuration_response::CfsConfigurationResponse,
    session::CfsSessionGetResponse,
  },
  ims::Image,
};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Per-node details returned by `GET /api/v1/nodes`.
///
/// Mirror of `csm_rs::node::types::NodeDetails` with identical fields
/// and identical JSON wire format. No conversion impl is needed in
/// the server crate — the response is serialised straight from the
/// csm-rs type and the CLI deserialises it into this mirror.
///
/// All fields are wire-stringified (CSM serializes them that way);
/// callers parse them as needed for display or comparison. The empty
/// string is the conventional "unset" sentinel — see
/// `cluster_status::compute_summary_status` for case-insensitive
/// matching on the status fields.
///
/// # Wire shape
///
/// ```json
/// {
///   "xname": "x3000c0s1b0n0",
///   "nid": "nid001313",
///   "hsm": "alps,zinal",
///   "power_status": "On",
///   "desired_configuration": "cos-2.5",
///   "configuration_status": "configured",
///   "enabled": "true",
///   "error_count": "0",
///   "boot_image_id": "0a1b2c3d-...",
///   "boot_configuration": "cos-2.5",
///   "kernel_params": "console=ttyS0 ..."
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NodeDetails {
  /// Physical location ID, e.g. `x3000c0s1b0n0`.
  pub xname: String,
  /// Numeric node ID as a string, e.g. `"nid001313"`.
  pub nid: String,
  /// Comma-separated HSM group names this node belongs to.
  pub hsm: String,
  /// Current power state reported by PCS (`"On"`, `"Off"`, `"Ready"`,
  /// etc.).
  pub power_status: String,
  /// CFS desired-configuration name targeting this node.
  pub desired_configuration: String,
  /// CFS configuration status (`"configured"`, `"pending"`,
  /// `"failed"`, etc.).
  pub configuration_status: String,
  /// `"true"` or `"false"` — whether the node is enabled in the
  /// hardware state manager.
  pub enabled: String,
  /// Stringified count of recent CFS failures.
  pub error_count: String,
  /// IMS image ID currently set as the boot image.
  pub boot_image_id: String,
  /// CFS configuration linked to the boot image.
  pub boot_configuration: String,
  /// Kernel command-line parameters as last reported by BSS.
  pub kernel_params: String,
}
