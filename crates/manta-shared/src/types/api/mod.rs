//! Single-namespace API types shared between `manta-cli` (Serialize
//! side) and `manta-server` (Deserialize side).
//!
//! Each submodule owns one resource end-to-end: HTTP request/response
//! bodies, query-string structs, and CLI-built parameter structs all
//! sit next to each other rather than being split across parallel
//! `wire/` and `params/` namespaces. Cross-cutting query and response
//! shapes live in [`queries`] and [`responses`].
//!
//! Convention:
//!
//! - Owned types (`String`, `Vec<String>`) throughout. The CLI pays a
//!   small allocation cost on the outbound build; in exchange there is
//!   exactly one type to keep in sync.
//! - Derives are `#[derive(Serialize, Deserialize, ToSchema)]` (or
//!   `IntoParams` for query strings). The server uses `Deserialize` +
//!   the utoipa derives for OpenAPI; the CLI uses `Serialize`.
//! - Field-level docs describe wire semantics. They show up in both
//!   the OpenAPI spec and the rustdoc the CLI consumes.
//!
//! Types re-exported from `manta-backend-dispatcher` (response DTOs
//! owned by upstream crates) live in [`super::dto`], which is kept
//! separate because it serves a different concern: types we don't own.

pub mod boot_parameters;
pub mod cluster;
pub mod configuration;
pub mod group;
pub mod hardware;
pub mod hw_cluster;
pub mod image;
pub mod kernel_parameters;
pub mod migrate;
pub mod node;
pub mod power;
pub mod queries;
pub mod redfish_endpoints;
pub mod responses;
pub mod sat_file;
pub mod session;
pub mod template;
