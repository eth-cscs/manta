//! [`AuthenticationTrait`] impl for [`StaticBackendDispatcher`].
//!
//! - `exchange_api_token` is called on every request (from the
//!   `RequestCtx` extractor): CSM exchanges the caller's CSCS Keycloak
//!   token for a CSM token at the site's Keycloak; OpenCHAMI passes it
//!   through for now.
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
    if let Err(ref e) = result {
      tracing::warn!(
        backend,
        error = %e,
        "dispatch: exchange_api_token returned error from backend client"
      );
    }
    result
  }
}
