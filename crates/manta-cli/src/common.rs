//! Compatibility shim: re-export every `manta_shared::common::*` module so
//! existing `crate::common::X` import paths inside `manta-cli` keep
//! resolving without each file having to switch to `manta_shared::common::X`.
//!
//! Combined here with re-exports for `crate::cli::common::*` and
//! `crate::server::common::*` so the in-body `common::X::Y` references
//! (which come from `use crate::common;`) also resolve to the right place.
//!
//! Once we've audited every caller and switched them to explicit
//! `manta_shared::common::*` or `crate::{cli,server}::common::*` imports,
//! this file can shrink to just `pub use manta_shared::common::*;` or go
//! away entirely.

#[allow(unused_imports)]
pub use manta_shared::common::{
  DATETIME_FORMAT, app_context, audit, authorization, config, jwt_ops,
  kafka, log_ops,
};
#[allow(unused_imports)]
pub use crate::cli::common::{
  authentication, hooks, kernel_parameters_ops, local_git_repo, user_interaction,
};
#[allow(unused_imports)]
pub use crate::server::common::{
  boot_parameters, hw_inventory_utils, ims_ops, node_ops, vault,
};
