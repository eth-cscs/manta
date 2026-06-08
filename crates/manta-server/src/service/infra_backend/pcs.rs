//! PCS power-control methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::pcs::transitions::types::{
  TransitionResponse, TransitionStartOutput,
};

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Start a PCS power transition (on / off / restart) for the given xnames.
  pub async fn pcs_transitions_post(
    &self,
    token: &str,
    operation: &str,
    nodes: &[String],
  ) -> Result<TransitionStartOutput, Error> {
    self
      .backend
      .pcs_transitions_post(token, operation, nodes)
      .await
  }

  /// Fetch a single PCS power-transition snapshot by id.
  pub async fn pcs_transitions_get(
    &self,
    token: &str,
    transition_id: &str,
  ) -> Result<TransitionResponse, Error> {
    self.backend.pcs_transitions_get(token, transition_id).await
  }
}
