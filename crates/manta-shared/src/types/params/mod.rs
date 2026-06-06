//! Request parameter structs passed from the CLI to the server.
//!
//! Each submodule mirrors a `service::*` module on the server side. The CLI
//! constructs these structs from clap arguments; the server consumes them
//! after deserializing the corresponding query/body. Keeping one definition
//! per type ensures the wire format stays consistent.

pub mod boot_parameters;
pub mod cluster;
pub mod configuration;
pub mod group;
pub mod hardware;
pub mod hw_cluster;
pub mod image;
pub mod kernel_parameters;
pub mod node;
pub mod power;
pub mod redfish_endpoints;
pub mod session;
pub mod template;
