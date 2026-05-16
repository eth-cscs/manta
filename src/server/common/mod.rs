//! Helpers used only by the manta HTTP server (handlers + service layer).
//!
//! Modules here have no CLI-side consumers and will move into the
//! `manta-server` crate when the workspace split lands.

pub mod authorization;
pub mod boot_parameters;
pub mod hw_inventory_utils;
pub mod ims_ops;
pub mod node_ops;
pub mod vault;
