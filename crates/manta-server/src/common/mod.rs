//! Compatibility shim: re-export every `manta_shared::common::*` module so
//! existing `crate::common::X` import paths inside `manta-server` keep
//! resolving without each file having to switch to `manta_shared::common::X`.
//!
//! Also re-exports `crate::server::common::*` so the in-body
//! `common::X::Y` references (which come from `use crate::common;`) resolve
//! to the server-side helpers (`node_ops`, `hw_inventory_utils`, …).
//!
//! `app_context` and `authorization` are server-local (both reach for
//! `StaticBackendDispatcher`, which is server-only); they're declared
//! as sub-modules below rather than re-exported from `manta-shared`.

pub mod app_context;

#[allow(unused_imports)]
pub use crate::server::common::{
  authorization, boot_parameters, hw_inventory_utils, ims_ops, node_ops, vault,
};
#[allow(unused_imports)]
pub use manta_shared::common::{
  DATETIME_FORMAT, audit, config, jwt_ops, kafka, log_ops,
};
