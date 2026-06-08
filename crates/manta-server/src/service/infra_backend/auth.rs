//! Auth-related backend methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::authentication::AuthenticationTrait;

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Exchange username/password for a CSM bearer token.
  pub async fn get_api_token(
    &self,
    username: &str,
    password: &str,
  ) -> Result<String, Error> {
    self.backend.get_api_token(username, password).await
  }

  /// Verify a CSM bearer token is still accepted by the backend.
  pub async fn validate_api_token(&self, token: &str) -> Result<(), Error> {
    self.backend.validate_api_token(token).await
  }
}
