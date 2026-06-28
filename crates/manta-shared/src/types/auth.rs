//! Wire types for the `POST /api/v1/auth/{token,validate}` endpoints.
//!
//! Carried over the wire by both `manta-cli` (sending requests via
//! `MantaClient`) and `manta-server` (deserializing them in handlers).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/auth/token`.
///
/// Paired with [`AuthTokenResponse`] on success. The `/auth/*`
/// sub-router is wrapped by `strip_body_for_logs`, so neither the
/// request body nor the issued token appears in access logs.
///
/// # Wire shape
///
/// ```json
/// { "username": "alice", "password": "hunter2" }
/// ```
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthTokenRequest {
  /// Keycloak username submitted to the configured backend.
  pub username: String,
  /// Keycloak password. Never logged by the server (the
  /// `/auth/*` sub-router is wrapped by `strip_body_for_logs`).
  pub password: String,
}

/// Response body for `POST /api/v1/auth/token`.
///
/// Returned in exchange for a valid [`AuthTokenRequest`].
///
/// # Wire shape
///
/// ```json
/// { "token": "eyJhbGciOi..." }
/// ```
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthTokenResponse {
  /// Bearer token issued by the backend (CSM or OpenCHAMI Keycloak).
  /// Pass this as `Authorization: Bearer <token>` on every
  /// subsequent request.
  pub token: String,
}

/// Request body for `POST /api/v1/auth/validate`.
///
/// Used to check a previously-issued [`AuthTokenResponse::token`]
/// before relying on it. The server returns `200 OK` for a valid
/// token and `401` otherwise — there is no dedicated response body.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ValidateTokenRequest {
  /// Bearer token to validate against the backend.
  pub token: String,
}
