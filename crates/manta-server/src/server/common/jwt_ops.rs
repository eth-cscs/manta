//! JWT decode helpers (relocated to `manta_shared::common::jwt_ops`).
//!
//! This module is a thin re-export so server-side call sites that
//! reach `crate::server::common::jwt_ops::*` continue to resolve
//! without churn. New code should prefer the canonical path
//! `manta_shared::common::jwt_ops`.

pub use manta_shared::common::jwt_ops::*;
