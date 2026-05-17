//! Compatibility shim: re-export every `manta_shared::common::*` module so
//! existing `crate::common::X` import paths inside `manta-cli` keep
//! resolving without each file having to switch to `manta_shared::common::X`.
//!
//! Also re-exports `crate::cli::common::*` so the in-body `common::X::Y`
//! references (which come from `use crate::common;`) keep resolving.
//!
//! Once every caller has been audited and switched to explicit
//! `manta_shared::common::*` or `crate::cli::common::*` imports, this file
//! can collapse to a one-liner or go away entirely.

#[allow(unused_imports)]
pub use manta_shared::common::{
  DATETIME_FORMAT, app_context, audit, authorization, config, jwt_ops,
  kafka, log_ops,
};
#[allow(unused_imports)]
pub use crate::cli::common::{
  authentication, hooks, kernel_parameters_ops, local_git_repo, user_interaction,
};
