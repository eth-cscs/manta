//! Dispatches `AuthenticationTrait` methods to csm-rs or ochami-rs.

use manta_backend_dispatcher::{
  error::Error, interfaces::authentication::AuthenticationTrait,
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl AuthenticationTrait for StaticBackendDispatcher {
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
