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
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeDetails {
  pub xname: String,
  pub nid: String,
  pub hsm: String,
  pub power_status: String,
  pub desired_configuration: String,
  pub configuration_status: String,
  pub enabled: String,
  pub error_count: String,
  pub boot_image_id: String,
  pub boot_configuration: String,
  pub kernel_params: String,
}
