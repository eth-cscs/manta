//! [`AuthenticationTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the per-backend OIDC/Keycloak flow:
//!
//! - CSM: token exchange against the site's Keycloak realm (the
//!   `shasta` client) via csm-rs's `get_api_token`.
//! - Ochami: token exchange against the ochami-rs configured OIDC
//!   provider; both methods are implemented on the Ochami side.
//!
//! Both branches are real (no `UnsupportedBackend`). The wrapper adds
//! structured `tracing::debug` on entry and `tracing::warn` on the
//! error path so an auth failure surfaces backend + user metadata at
//! the dispatcher boundary.

use super::*;

impl AuthenticationTrait for StaticBackendDispatcher {
  /// Exchange `username` / `password` for an API bearer token.
  ///
  /// # Errors
  ///
  /// Returns [`Error::NetError`] / [`Error::RequestError`] when the
  /// IdP is unreachable, [`Error::CsmError`] / [`Error::BadRequest`]
  /// on rejection (bad credentials, MFA challenge, locked account),
  /// and [`Error::Message`] for backend-specific failures.
  async fn get_api_token(
    &self,
    username: &str,
    password: &str,
  ) -> Result<String, Error> {
    let backend = self.backend_kind();
    tracing::debug!(backend, user = %username, "dispatch: get_api_token");
    let result = dispatch!(self, get_api_token, username, password);
    if let Err(ref e) = result {
      tracing::warn!(
        backend,
        user = %username,
        error = %e,
        "dispatch: get_api_token returned error from backend client"
      );
    }
    result
  }

  /// Validate `auth_token` against the IdP / introspection endpoint.
  ///
  /// `Ok(())` means the token is currently accepted by the backend;
  /// no claims are returned through this trait.
  async fn validate_api_token(&self, auth_token: &str) -> Result<(), Error> {
    let backend = self.backend_kind();
    tracing::debug!(backend, "dispatch: validate_api_token");
    let result = dispatch!(self, validate_api_token, auth_token);
    if let Err(ref e) = result {
      tracing::warn!(
        backend,
        error = %e,
        "dispatch: validate_api_token returned error from backend client"
      );
    }
    result
  }
}
