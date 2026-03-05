use manta_backend_dispatcher::{
  error::Error, interfaces::pcs::PCSTrait,
  types::pcs::transitions::types::TransitionResponse,
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl PCSTrait for StaticBackendDispatcher {
  async fn power_on_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<TransitionResponse, Error> {
    dispatch!(self, power_on_sync, auth_token, nodes)
  }

  async fn power_off_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    dispatch!(self, power_off_sync, auth_token, nodes, force)
  }

  async fn power_reset_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    dispatch!(self, power_reset_sync, auth_token, nodes, force)
  }
}
