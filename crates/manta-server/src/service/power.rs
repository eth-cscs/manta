//! Power on/off/reset operations against PCS.
//!
//! `POST /power` (handler `post_power`) now returns immediately with
//! the PCS transition id; the polling loop that used to live in
//! `pcs_transitions_post_block` runs CLI-side. The CLI snapshots the
//! transition with `GET /power/transitions/{id}` (handler
//! `get_power_transition`) every few seconds until it completes.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::pcs::PCSTrait;
use manta_backend_dispatcher::types::pcs::transitions::types::{
  TransitionResponse, TransitionStartOutput,
};

use crate::server::common::app_context::InfraContext;
use crate::service::node_ops;
pub use manta_shared::shared::params::power::{
  ApplyPowerParams, PowerAction, PowerTargetType,
};

/// Resolve the caller's `targets_expression` into the concrete xname
/// list to pass to `apply_power`. For `Cluster` targets the expression
/// is a single HSM group name and we fetch its members; for `Nodes`
/// it's a hosts expression. Returns `Error::BadRequest` when the
/// resolution yields an empty list (the caller would otherwise hit
/// PCS with no work to do).
pub async fn resolve_target_xnames(
  infra: &InfraContext<'_>,
  token: &str,
  target_type: PowerTargetType,
  targets_expression: &str,
) -> Result<Vec<String>, Error> {
  let xnames = match target_type {
    PowerTargetType::Cluster => {
      infra
        .backend
        .get_member_vec_from_group_name_vec(
          token,
          std::slice::from_ref(&targets_expression.to_string()),
        )
        .await?
    }
    PowerTargetType::Nodes => {
      node_ops::resolve_hosts_expression(
        infra.backend,
        token,
        targets_expression,
        false,
      )
      .await?
    }
  };

  if xnames.is_empty() {
    return Err(Error::BadRequest("No nodes to operate on".into()));
  }

  Ok(xnames)
}

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
  infra
    .backend
    .pcs_transitions_post(
      token,
      pcs_operation(params.action, params.force),
      &params.xnames,
    )
    .await
}

/// Map the CLI's typed `(PowerAction, force)` pair to PCS's
/// wire-level `operation` string. `force` is ignored for `On`
/// (PCS doesn't model a forceful power-on); for `Off` and `Reset`
/// it toggles between the graceful (`soft-…`) and forceful
/// (`force-off` / `hard-restart`) variants.
pub(crate) fn pcs_operation(action: PowerAction, force: bool) -> &'static str {
  match (action, force) {
    (PowerAction::On, _) => "on",
    (PowerAction::Off, false) => "soft-off",
    (PowerAction::Off, true) => "force-off",
    (PowerAction::Reset, false) => "soft-restart",
    (PowerAction::Reset, true) => "hard-restart",
  }
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

#[cfg(test)]
mod tests {
  //! Wire-mapping lock for `(PowerAction, force) -> PCS operation
  //! string`. PCS rejects anything outside its known set; renaming
  //! one of these strings would break power for everyone.

  use super::{PowerAction, pcs_operation};

  #[test]
  fn on_ignores_force_flag() {
    // PCS doesn't model a forceful "on" — the bool should not change
    // the wire string.
    assert_eq!(pcs_operation(PowerAction::On, false), "on");
    assert_eq!(pcs_operation(PowerAction::On, true), "on");
  }

  #[test]
  fn off_distinguishes_soft_from_force() {
    assert_eq!(pcs_operation(PowerAction::Off, false), "soft-off");
    assert_eq!(pcs_operation(PowerAction::Off, true), "force-off");
  }

  #[test]
  fn reset_distinguishes_soft_from_hard() {
    assert_eq!(pcs_operation(PowerAction::Reset, false), "soft-restart");
    assert_eq!(pcs_operation(PowerAction::Reset, true), "hard-restart");
  }
}
