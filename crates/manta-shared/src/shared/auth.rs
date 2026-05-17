//! Wire types for the `POST /api/v1/auth/{token,validate}` endpoints.
//!
//! Carried over the wire by both `manta-cli` (sending requests via
//! `MantaClient`) and `manta-server` (deserializing them in handlers).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/auth/token`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthTokenRequest {
  pub username: String,
  pub password: String,
}

/// Response body for `POST /api/v1/auth/token`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthTokenResponse {
  pub token: String,
}

/// Request body for `POST /api/v1/auth/validate`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ValidateTokenRequest {
  pub token: String,
}
