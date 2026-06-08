//! `PCSTrait` (power control) impl for `StaticBackendDispatcher`.
//!
//! `POST /power` returns immediately with a transition id (via
//! `pcs_transitions_post`); the CLI then polls `pcs_transitions_get`
//! until the transition is `completed`. The older blocking
//! `power_*_sync` trait methods (server-side polling loop) have been
//! removed.

use super::*;

impl PCSTrait for StaticBackendDispatcher {
  async fn pcs_transitions_post(
    &self,
    auth_token: &str,
    operation: &str,
    nodes: &[String],
  ) -> Result<TransitionStartOutput, Error> {
    dispatch!(self, pcs_transitions_post, auth_token, operation, nodes)
  }

  async fn pcs_transitions_get(
    &self,
    auth_token: &str,
    transition_id: &str,
  ) -> Result<TransitionResponse, Error> {
    dispatch!(self, pcs_transitions_get, auth_token, transition_id)
  }
}
