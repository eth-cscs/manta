//! Authentication service — proxies CLI credential exchange to the
//! configured CSM/OCHAMI backend.
//!
//! The CLI never talks to Keycloak directly; it POSTs username+password
//! to `manta-server /api/v1/auth/token`, which calls
//! `backend.get_api_token` on the user's behalf and returns the CSM
//! bearer token. `validate_api_token` mirrors the CLI's pre-Phase-6
//! token-still-valid check.

use manta_backend_dispatcher::{
  error::Error, interfaces::authentication::AuthenticationTrait,
};

use crate::common::app_context::InfraContext;

/// Exchange `username` + `password` for a CSM bearer token via the
/// site's configured backend.
pub async fn get_api_token(
  infra: &InfraContext<'_>,
  username: &str,
  password: &str,
) -> Result<String, Error> {
  infra.backend.get_api_token(username, password).await
}

/// Verify that `token` is still accepted by the site's backend.
pub async fn validate_api_token(
  infra: &InfraContext<'_>,
  token: &str,
) -> Result<(), Error> {
  infra.backend.validate_api_token(token).await
}
