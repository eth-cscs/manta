//! Wire types for the `POST /api/v1/auth/{token,validate}` endpoints.
//!
//! Carried over the wire by both `manta-cli` (sending requests via
//! `MantaClient`) and `manta-server` (deserializing them in handlers).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/auth/token`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthTokenRequest {
  /// Keycloak username submitted to the configured backend.
  pub username: String,
  /// Keycloak password. Never logged by the server (the
  /// `/auth/*` sub-router is wrapped by `strip_body_for_logs`).
  pub password: String,
}

/// Response body for `POST /api/v1/auth/token`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthTokenResponse {
  /// Bearer token issued by the backend (CSM or OpenCHAMI Keycloak).
  /// Pass this as `Authorization: Bearer <token>` on every
  /// subsequent request.
  pub token: String,
}

/// Request body for `POST /api/v1/auth/validate`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ValidateTokenRequest {
  /// Bearer token to validate against the backend.
  pub token: String,
}
