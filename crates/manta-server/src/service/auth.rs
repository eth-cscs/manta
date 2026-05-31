//! Authentication service — proxies CLI credential exchange to the
//! configured CSM/OCHAMI backend.
//!
//! The CLI never talks to Keycloak directly; it POSTs username+password
//! to `manta-server /api/v1/auth/token`, which calls
//! `backend.get_api_token` on the user's behalf and returns the CSM
//! bearer token. `validate_api_token` exposes a lightweight
//! "is-this-token-still-valid" probe the CLI can call before sending
//! a long-running request that would otherwise fail mid-flight.

use std::time::Instant;

use manta_backend_dispatcher::{
  error::Error, interfaces::authentication::AuthenticationTrait,
};

use crate::server::common::app_context::InfraContext;

/// Exchange `username` + `password` for a CSM bearer token via the
/// site's configured backend.
#[tracing::instrument(
  skip_all,
  fields(
    site = %infra.site_name,
    backend = %infra.backend.backend_kind(),
    backend_url = %infra.shasta_base_url,
  )
)]
pub async fn get_api_token(
  infra: &InfraContext<'_>,
  username: &str,
  password: &str,
) -> Result<String, Error> {
  tracing::info!(user = %username, "backend: requesting token");
  let started = Instant::now();
  let result = infra.backend.get_api_token(username, password).await;
  let elapsed_ms = started.elapsed().as_millis() as u64;
  match &result {
    Ok(_) => tracing::debug!(
      user = %username,
      elapsed_ms,
      "backend: token issued"
    ),
    Err(e) => tracing::warn!(
      user = %username,
      elapsed_ms,
      error = %e,
      "backend: token request rejected"
    ),
  }
  result
}

/// Verify that `token` is still accepted by the site's backend.
#[tracing::instrument(
  skip_all,
  fields(
    site = %infra.site_name,
    backend = %infra.backend.backend_kind(),
    backend_url = %infra.shasta_base_url,
  )
)]
pub async fn validate_api_token(
  infra: &InfraContext<'_>,
  token: &str,
) -> Result<(), Error> {
  tracing::info!("backend: validating token");
  let started = Instant::now();
  let result = infra.backend.validate_api_token(token).await;
  let elapsed_ms = started.elapsed().as_millis() as u64;
  match &result {
    Ok(()) => tracing::debug!(elapsed_ms, "backend: token accepted"),
    Err(e) => tracing::warn!(
      elapsed_ms,
      error = %e,
      "backend: token validation rejected"
    ),
  }
  result
}
