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

/// Aggregate summary of CFS configurations + sessions + BOS templates +
/// IMS images flattened into image-centric rows
/// (`/api/v2/analysis/images`).
pub mod analysis;
/// Boot-parameter request/response bodies (`/api/v2/boot-config`,
/// `/api/v2/boot-parameters`).
pub mod boot_parameters;
/// CLI-built params for `GET /clusters`.
pub mod cluster;
/// CLI-built params for `GET /configurations`.
pub mod configuration;
/// Wire shape for the per-row configuration-deletion-safety verdict
/// returned by `/api/v2/configurations`.
pub mod configuration_analysis;
/// HSM group request/response bodies (`/api/v2/groups`,
/// `/api/v2/groups/{name}/members`).
pub mod group;
/// CLI-built params for `GET /groups/hardware` and the
/// `/hardware-nodes-list` family.
pub mod hardware;
/// Wire types for the `POST/DELETE /api/v2/hardware-clusters/{target}/*`
/// endpoints.
pub mod hw_cluster;
/// CLI-built params for `GET /images`.
pub mod image;
/// Kernel-parameter request/response bodies
/// (`/api/v2/kernel-parameters/*`). The internal `KernelParamOperation`
/// enum is server-only and lives in `service::kernel_parameters`.
pub mod kernel_parameters;
/// Wire types for the `POST /api/v2/migrate/*` endpoints.
pub mod migrate;
/// Node request/response bodies (`/api/v2/nodes`).
pub mod node;
/// Power request/response bodies (`/api/v2/power`).
pub mod power;
/// Shared `IntoParams` query-string structs for every non-trivial GET
/// and DELETE endpoint.
pub mod queries;
/// CLI-built params for `GET/POST/PUT /redfish-endpoints`.
pub mod redfish_endpoints;
/// Request body for `PUT /api/v2/runtime-configuration`.
pub mod runtime_configuration;
/// Tiny response shapes (`{ "created": true }`, `{ "id": "..." }`) so
/// the OpenAPI spec carries real types instead of `serde_json::Value`.
pub mod responses;
/// SAT-file element-apply request/response bodies (`POST
/// /api/v2/sat-file/*`) and CLI-built params for the whole-file
/// pass-through.
pub mod sat_file;
/// CFS session request/response bodies (`/api/v2/sessions`).
pub mod session;
/// BOS session-template request/response bodies (`/api/v2/templates`,
/// `/api/v2/templates/{name}/sessions`).
pub mod template;
