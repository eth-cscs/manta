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
    dispatch!(self, get_api_token, username, password)
  }

  async fn validate_api_token(&self, auth_token: &str) -> Result<(), Error> {
    dispatch!(self, validate_api_token, auth_token)
  }
}
