//! Wire response types re-exported from `manta-backend-dispatcher::types`.
//!
//! Both the CLI (deserializing responses from the manta server in
//! `http_client.rs` and formatting them in `cli::output`) and the server
//! (serializing them back over HTTP) consume these. Re-exporting them through
//! this module keeps a single import path so that, when the workspace split
//! happens, only the `manta-shared` crate depends on
//! `manta-backend-dispatcher` for type definitions.

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

/// Per-node details returned by `GET /api/v1/nodes` and consumed by the CLI's
/// output formatters. Re-exported from `csm-rs` so callers in `manta-cli`
/// don't need `csm-rs` as a direct dep.
pub use csm_rs::node::types::NodeDetails;
