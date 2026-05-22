//! Wire response types shared by both binaries.
//!
//! The CLI (in `http_client.rs` and `cli::output::*`) deserializes
//! responses from the manta-server using these types; the server
//! serializes them back over HTTP via the service layer.
//!
//! `NodeDetails` is mirrored locally (rather than re-exporting from
//! `csm-rs`) so `manta-shared` — and therefore `manta-cli` — does not
//! transitively depend on `csm-rs`. The server converts from
//! `csm_rs::node::types::NodeDetails` at the service-layer boundary
//! (see `crates/manta-server/src/wire_conv.rs`). The JSON wire shape
//! is byte-identical: same field names, no `#[serde(rename)]`.
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

/// Per-node details returned by `GET /api/v1/nodes`.
///
/// Mirror of `csm_rs::node::types::NodeDetails` with identical fields
/// and identical JSON wire format. The server converts from the
/// upstream type via `From` in `wire_conv.rs`.
///
/// All fields are wire-stringified (CSM serializes them that way);
/// callers parse them as needed for display or comparison.
#[derive(Debug, Serialize, Deserialize)]
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
