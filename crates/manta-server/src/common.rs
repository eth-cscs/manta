//! Compatibility shim: re-export every `manta_shared::common::*` module so
//! existing `crate::common::X` import paths inside `manta-server` keep
//! resolving without each file having to switch to `manta_shared::common::X`.
//!
//! Also re-exports `crate::server::common::*` so the in-body
//! `common::X::Y` references (which come from `use crate::common;`) resolve
//! to the server-side helpers (`node_ops`, `hw_inventory_utils`, …).
//!
//! Will collapse once every caller has been audited and switched to
//! explicit `manta_shared::common::*` / `crate::server::common::*` imports.

#[allow(unused_imports)]
pub use manta_shared::common::{
  DATETIME_FORMAT, app_context, audit, authorization, config, jwt_ops,
  kafka, log_ops,
};
#[allow(unused_imports)]
pub use crate::server::common::{
  boot_parameters, hw_inventory_utils, ims_ops, node_ops, vault,
};
