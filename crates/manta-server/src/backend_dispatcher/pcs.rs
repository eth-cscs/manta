//! [`PCSTrait`] (power control) impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the CSM PCS (Power Control Service)
//! `/apis/power-control/v1/{power-status,transitions}` endpoints.
//! Both CSM and Ochami implement this trait natively.
//!
//! `POST /transitions` returns immediately with a transition id (via
//! [`pcs_transitions_post`](StaticBackendDispatcher::pcs_transitions_post));
//! the CLI then polls
//! [`pcs_transitions_get`](StaticBackendDispatcher::pcs_transitions_get)
//! until the transition is `completed`. The older blocking
//! `power_*_sync` trait methods (server-side polling loop) have been
//! removed.

use manta_backend_dispatcher::types::pcs::power_status::types::PowerStatusAll;

use super::*;

impl PCSTrait for StaticBackendDispatcher {
  /// `GET /power-status?xname=...` — current power state for each
  /// `nodes` entry. `power_status_filter` and `management_state_filter`
  /// map to the upstream `powerStatusFilter` / `managementStateFilter`
  /// query parameters.
  async fn power_status(
    &self,
    auth_token: &str,
    nodes: &[String],
    power_status_filter: Option<&str>,
    management_state_filter: Option<&str>,
  ) -> Result<PowerStatusAll, Error> {
    dispatch!(
      self,
      power_status,
      auth_token,
      nodes,
      power_status_filter,
      management_state_filter
    )
  }

  /// `POST /transitions` — start a power `operation` (e.g. `on`,
  /// `off`, `soft-restart`) against `nodes`. Returns the transition
  /// id; the caller polls
  /// [`pcs_transitions_get`](Self::pcs_transitions_get) for the
  /// outcome.
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

  /// `GET /transitions/{id}` — single transition's progress / final
  /// status. Polled by the CLI to drive the power-op wait loop.
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
