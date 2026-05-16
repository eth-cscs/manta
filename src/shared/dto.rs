//! Wire response types re-exported from `manta-backend-dispatcher::types`.
//!
//! Both the CLI (deserializing responses from the manta server in
//! `http_client.rs` and formatting them in `cli::output`) and the server
//! (serializing them back over HTTP) consume these. Re-exporting them through
//! this module keeps a single import path so that, when the workspace split
//! happens, only the `manta-shared` crate depends on
//! `manta-backend-dispatcher` for type definitions.

pub use manta_backend_dispatcher::types::{
  Group, K8sDetails, NodeSummary,
  bos::session_template::BosSessionTemplate,
  bss::BootParameters,
  cfs::{
    cfs_configuration_response::CfsConfigurationResponse,
    session::CfsSessionGetResponse,
  },
  ims::Image,
};
