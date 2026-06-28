//! Output renderer modules — one per resource (table + JSON).
//!
//! Each `crate::dispatch::*` handler ends by calling into one of
//! these modules. The shape is consistent across the family: every
//! `print(...)` accepts an `output` flag string (`"table"` |
//! `"json"`, sometimes more) sourced from the
//! `-o/--output` clap flag declared in `crate::build::output_flag`.
//!
//! - [`action_result`] — mutating verbs (status line / JSON envelope)
//! - [`boot_parameters`] — BSS boot parameters (JSON only)
//! - [`config_summary`] — `manta config show` (table or JSON)
//! - [`configuration`] — CFS configurations (table)
//! - [`group`] — HSM groups (table)
//! - [`hardware`] — hardware inventory (multiple table formats + JSON)
//! - [`image`] — IMS images (table)
//! - [`kernel_parameters`] — BSS kernel-parameter grouping (table)
//! - [`node`] — HSM nodes (table or summary table)
//! - [`redfish_endpoints`] — Redfish endpoints (table or JSON)
//! - [`session`] — CFS sessions (table or JSON)
//! - [`template`] — BOS session templates (table or JSON)

pub mod action_result;
pub mod boot_parameters;
pub mod config_summary;
pub mod configuration;
pub mod group;
pub mod hardware;
pub mod image;
pub mod kernel_parameters;
pub mod node;
pub mod redfish_endpoints;
pub mod session;
pub mod template;
