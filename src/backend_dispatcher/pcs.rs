use manta_backend_dispatcher::{
  error::Error, interfaces::pcs::PCSTrait,
  types::pcs::transitions::types::TransitionResponse,
};

use StaticBackendDispatcher::*;

use serde_json::Value;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl PCSTrait for StaticBackendDispatcher {
  async fn power_on_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<TransitionResponse, Error> {
    match self {
      CSM(b) => b.power_on_sync(auth_token, nodes).await,
      OCHAMI(b) => b.power_on_sync(auth_token, nodes).await,
    }
  }

  async fn power_off_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    match self {
      CSM(b) => b.power_off_sync(auth_token, nodes, force).await,
      OCHAMI(b) => b.power_off_sync(auth_token, nodes, force).await,
    }
  }

  async fn power_reset_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    match self {
      CSM(b) => b.power_reset_sync(auth_token, nodes, force).await,
      OCHAMI(b) => b.power_reset_sync(auth_token, nodes, force).await,
    }
  }
}
