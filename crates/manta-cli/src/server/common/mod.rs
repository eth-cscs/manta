//! Helpers used only by the manta HTTP server (handlers + service layer).
//!
//! `authorization` was promoted back to `crate::common::authorization`
//! because the CLI's `apply session` still calls its helpers.

pub mod boot_parameters;
pub mod hw_inventory_utils;
pub mod ims_ops;
pub mod node_ops;
pub mod vault;
