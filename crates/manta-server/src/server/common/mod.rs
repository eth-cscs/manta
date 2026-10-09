//! Helpers used only by the manta HTTP server (handlers + service layer).
//!
//! - [`app_context`] — per-request [`app_context::InfraContext`] bundle
//!   (backend dispatcher + base URLs + TLS material + optional
//!   Vault/k8s URLs). Built by [`super::ServerState::infra_context`]
//!   on every request.
//! - [`jwt_ops`] — extract `name`, `preferred_username`, and
//!   `realm_access.roles` from a bearer JWT without verifying the
//!   signature (see the module-level security caveat).
//! - [`vault`] — HTTP client for the HashiCorp Vault used by
//!   handlers that need backend secrets (Gitea token, k8s creds).

pub mod app_context;
pub mod jwt_ops;
pub mod vault;
