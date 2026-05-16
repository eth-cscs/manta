//! Power on/off/reset operations against PCS.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::pcs::transitions::types::TransitionResponse;

use crate::common::app_context::InfraContext;
pub use crate::shared::params::power::{ApplyPowerParams, PowerAction};

/// Dispatch the requested power action to the backend PCS trait.
///
/// `force` is ignored for [`PowerAction::On`] (the backend trait has no
/// `force` parameter for the on transition).
pub async fn apply_power(
  infra: &InfraContext<'_>,
  token: &str,
  params: &ApplyPowerParams,
) -> Result<TransitionResponse, Error> {
  match params.action {
    PowerAction::On => infra.backend.power_on_sync(token, &params.xnames).await,
    PowerAction::Off => {
      infra.backend.power_off_sync(token, &params.xnames, params.force).await
    }
    PowerAction::Reset => {
      infra.backend.power_reset_sync(token, &params.xnames, params.force).await
    }
  }
}
