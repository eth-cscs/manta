//! Power on/off/reset operations against PCS.
//!
//! `POST /power` (handler `post_power`) now returns immediately with
//! the PCS transition id; the polling loop that used to live in
//! `pcs_transitions_post_block` runs CLI-side. The CLI snapshots the
//! transition with `GET /power/transitions/{id}` (handler
//! `get_power_transition`) every few seconds until it completes.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::pcs::transitions::types::{
  TransitionResponse, TransitionStartOutput,
};

use crate::server::common::app_context::InfraContext;
pub use manta_shared::shared::params::power::{ApplyPowerParams, PowerAction};

/// Start a PCS power transition (`on`, `soft-off`, `force-off`,
/// `soft-restart`, `hard-restart`) against `params.xnames` and return
/// the transition id immediately. The CLI is responsible for polling
/// `get_power_transition` until the transition reports `completed`.
///
/// `params.force` only changes the wire-level PCS operation for
/// `Off` and `Reset` — it's ignored for `On`, matching today's
/// behaviour.
pub async fn apply_power(
  infra: &InfraContext<'_>,
  token: &str,
  params: &ApplyPowerParams,
) -> Result<TransitionStartOutput, Error> {
  let operation = match (params.action, params.force) {
    (PowerAction::On, _) => "on",
    (PowerAction::Off, false) => "soft-off",
    (PowerAction::Off, true) => "force-off",
    (PowerAction::Reset, false) => "soft-restart",
    (PowerAction::Reset, true) => "hard-restart",
  };
  infra
    .backend
    .pcs_transitions_post(token, operation, &params.xnames)
    .await
}

/// Fetch the current snapshot of a PCS power transition by id. The
/// CLI's poll loop calls this every few seconds after `apply_power`
/// returned the transition id.
pub async fn get_power_transition(
  infra: &InfraContext<'_>,
  token: &str,
  transition_id: &str,
) -> Result<TransitionResponse, Error> {
  infra.backend.pcs_transitions_get(token, transition_id).await
}
