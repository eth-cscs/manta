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
    // Same trait-vs-inherent name clash as `pcs_transitions_get`:
    // disambiguate explicitly.
    match self {
      Self::CSM(b) => {
        PCSTrait::pcs_transitions_post(b, auth_token, operation, nodes).await
      }
      Self::OCHAMI(b) => {
        PCSTrait::pcs_transitions_post(b, auth_token, operation, nodes).await
      }
    }
  }

  async fn pcs_transitions_get(
    &self,
    auth_token: &str,
    transition_id: &str,
  ) -> Result<TransitionResponse, Error> {
    // ShastaClient has an inherent `pcs_transitions_get` that takes
    // only the token and returns a Vec. Disambiguate via the trait so
    // the resolver picks the trait method (single transition by id),
    // not the inherent one.
    match self {
      Self::CSM(b) => {
        PCSTrait::pcs_transitions_get(b, auth_token, transition_id).await
      }
      Self::OCHAMI(b) => {
        PCSTrait::pcs_transitions_get(b, auth_token, transition_id).await
      }
    }
  }
}
