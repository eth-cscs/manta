//! [`AuthenticationTrait`] impl for [`StaticBackendDispatcher`].
//!
//! - `exchange_api_token` is called on every request (from the
//!   `RequestCtx` extractor): it exchanges the caller's CSCS Keycloak
//!   token for one the site accepts, at the site's Keycloak on CSM
//!   (RFC 8693) or the site's Hydra on OpenCHAMI (RFC 7523).
//! - `get_api_token` (username/password grant) is no longer called by
//!   manta: the CLI obtains its token from the CSCS Keycloak directly.
//!   The forwarding is kept so `tests/dispatcher_coverage.rs` keeps
//!   holding every trait method to an override.
//!
//! The wrapper adds structured `tracing::debug` on entry and
//! `tracing::warn` on the error path so an auth failure surfaces the
//! backend at the dispatcher boundary.

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

  /// Exchange the caller's CSCS Keycloak token for the token this
  /// site's backend accepts.
  ///
  /// Called once per request by the `RequestCtx` extractor; the
  /// result replaces the caller's token for every backend, Vault and
  /// role check made on its behalf.
  async fn exchange_api_token(
    &self,
    subject_token: &str,
  ) -> Result<String, Error> {
    let backend = self.backend_kind();
    tracing::debug!(backend, "dispatch: exchange_api_token");
    let result = dispatch!(self, exchange_api_token, subject_token);
    match &result {
      // A refused token is routine (a user the site does not know, an
      // ended session): no warning for each one.
      Err(e @ (Error::Unauthorized(_) | Error::Forbidden(_))) => {
        tracing::info!(backend, error = %e, "dispatch: token exchange refused");
      }
      Err(e) => tracing::warn!(
        backend,
        error = %e,
        "dispatch: exchange_api_token returned error from backend client"
      ),
      Ok(_) => {}
    }
    result
  }
}
