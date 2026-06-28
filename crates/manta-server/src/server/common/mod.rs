//! Helpers used only by the manta HTTP server (handlers + service layer).
//!
//! - [`app_context`] — per-request [`app_context::InfraContext`] bundle
//!   (backend dispatcher + base URLs + TLS material + optional
//!   Vault/k8s URLs). Built by [`super::ServerState::infra_context`]
//!   on every request.
//! - [`audit`] — structured `auth_attempt` event builder that publishes
//!   to Kafka via [`kafka::Kafka`]. Fire-and-forget; failures are
//!   logged but never propagated.
//! - [`jwt_ops`] — extract `name`, `preferred_username`, and
//!   `realm_access.roles` from a bearer JWT without verifying the
//!   signature (see the module-level security caveat).
//! - [`kafka`] — lazily-initialised Kafka producer used by [`audit`].
//! - [`vault`] — HTTP client for the HashiCorp Vault used by
//!   handlers that need backend secrets (Gitea token, k8s creds).

pub mod app_context;
pub mod audit;
pub mod jwt_ops;
pub mod kafka;
pub mod vault;
