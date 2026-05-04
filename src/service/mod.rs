//! Business logic layer — orchestrates backend calls and enforces domain rules
//! for every resource type exposed by the CLI and HTTP server.

pub mod boot_parameters;
pub mod cluster;
pub mod configuration;
pub mod group;
pub mod hardware;
pub mod image;
pub mod kernel_parameters;
pub mod node;
pub mod redfish_endpoints;
pub mod session;
pub mod migrate;
pub mod sat_file;
pub mod template;
