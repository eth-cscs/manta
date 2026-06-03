//! Business logic layer — orchestrates backend calls and enforces domain rules
//! for every resource type exposed by the CLI and HTTP server.

pub mod auth;
pub mod authorization;
pub mod boot_parameters;
pub mod cluster;
pub mod configuration;
pub mod ephemeral_env;
pub mod group;
pub mod hardware;
pub mod hw_cluster;
pub mod image;
pub mod ims_ops;
pub mod infra_backend;
pub mod kernel_parameters;
pub mod migrate;
pub mod node;
pub mod node_ops;
pub mod power;
pub mod redfish_endpoints;
pub mod sat_file;
pub mod session;
pub mod template;
