//! Business logic layer — orchestrates backend calls and enforces
//! domain rules for every resource type exposed by the CLI and HTTP
//! server.
//!
//! # Position in the layering
//!
//! Sits between [`crate::server`] (HTTP handlers) and
//! [`crate::backend_dispatcher`] (CSM / OCHAMI dispatch). Per
//! `CLAUDE.md`'s boundary rule, handlers MUST only call functions in
//! this module; they never reach the backend dispatcher directly. The
//! service layer in turn calls backend trait methods on the
//! [`InfraContext`]'s `backend` field, which routes to the active
//! [`crate::dispatcher::StaticBackendDispatcher`] variant.
//!
//! [`InfraContext`]: crate::server::common::app_context::InfraContext
//!
//! # Conventions
//!
//! - Every public async function takes a `&InfraContext<'_>` and a
//!   `token: &str` so authorization can run before any backend call.
//! - All fallible paths return
//!   [`manta_backend_dispatcher::error::Error`]; the handler layer
//!   maps these to HTTP responses through `to_handler_error`.
//! - Authorization helpers in [`authorization`] are called from every
//!   function that takes a node, group, or session name from the
//!   caller — including read-only endpoints, so listings can't leak
//!   resources the caller couldn't have queried directly.
//!
//! # Module map
//!
//! - Cross-cutting: [`authorization`], [`infra_backend`], [`analysis`],
//!   [`sat_groups`], [`node_ops`], [`ims_ops`].
//! - Per-resource: [`auth`], [`boot_parameters`], [`configuration`],
//!   [`group`], [`hardware`], [`image`], [`kernel_parameters`],
//!   [`node`], [`node_details`], [`power`], [`redfish`], [`session`],
//!   [`template`].
//! - Composite operations: [`cluster`], [`ephemeral_env`], [`migrate`],
//!   [`hw_cluster`] (the last is a subdirectory module).

pub mod analysis;
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
pub mod node_details;
pub mod node_ops;
pub mod power;
pub mod redfish;
pub mod sat_groups;
pub mod session;
pub mod template;
