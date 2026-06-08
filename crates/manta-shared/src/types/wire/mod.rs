//! HTTP request / response body types shared between `manta-cli`
//! (Serialize side) and `manta-server` (Deserialize side).
//!
//! Each sub-module owns one resource's wire shapes. Both sides of the
//! HTTP boundary import the same struct, so a field rename can no
//! longer drift between the CLI and the server — the compiler enforces
//! the contract.
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

pub mod boot_parameters;
pub mod kernel_parameters;
pub mod session;
