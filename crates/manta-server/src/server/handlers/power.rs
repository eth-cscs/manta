//! Power endpoints.
//!
//! - `POST /api/v1/power` starts a PCS power transition and returns
//!   immediately with `{ transition_id, operation }`. The CLI then
//!   polls the next endpoint until the transition reports `completed`.
//! - `GET /api/v1/power/transitions/{id}` returns the current snapshot
//!   of the named transition (status + task counts + per-task detail).

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// POST /api/v1/power — Power on/off/reset nodes or cluster
// ---------------------------------------------------------------------------

/// Power action to apply to the target nodes or cluster.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PowerAction {
  /// Power on the nodes.
  On,
  /// Power off the nodes.
  Off,
  /// Power-cycle (reset) the nodes.
  Reset,
}

/// Whether `targets` contains xnames (`nodes`) or a single cluster name (`cluster`).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PowerTargetType {
  /// `targets` is a list of xnames.
  Nodes,
  /// `targets` contains a single HSM group name whose members will be targeted.
  Cluster,
}

/// Request body for `POST /power`.
#[derive(Deserialize, ToSchema)]
pub struct PowerRequest {
  /// Power operation to perform.
  pub action: PowerAction,
  /// For nodes: hosts expression (xnames, nids, or hostlist notation).
  /// For cluster: the HSM group name.
  pub host_expression: String,
  /// Indicates whether `host_expression` is a node expression or a cluster name.
  pub target_type: PowerTargetType,
  /// Pass `--force` to the underlying power operation (forceful shutdown/reset).
  #[serde(default)]
  pub force: bool,
}

/// `POST /api/v1/power` — start a PCS power transition (on / off /
/// reset) against nodes or all members of a cluster and return the
/// transition id **immediately**. Does not block until the
/// transition completes — the CLI is responsible for polling
/// `GET /power/transitions/{id}` until the snapshot reports
/// `transitionStatus = "completed"`.
///
/// Returns a `TransitionStartOutput` (`{ transitionID, operation }`)
/// as JSON. Callers can hand the `transitionID` to
/// [`get_power_transition`].
#[utoipa::path(post, path = "/power", tag = "power",
  params(SiteHeader),
  request_body = PowerRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "PCS transition started; returns TransitionStartOutput", body = serde_json::Value),
    (status = 400, description = "Bad request",            body = ErrorResponse),
    (status = 401, description = "Unauthorized",           body = ErrorResponse),
    (status = 500, description = "Internal error",         body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn post_power(
  ctx: RequestCtx,
  Json(body): Json<PowerRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_power action={:?} target_type={:?}",
    body.action,
    body.target_type
  );
  let infra = ctx.infra();

  let target_type = match body.target_type {
    PowerTargetType::Cluster => service::power::PowerTargetType::Cluster,
    PowerTargetType::Nodes => service::power::PowerTargetType::Nodes,
  };
  let xnames = service::power::resolve_target_xnames(
    &infra,
    &ctx.token,
    target_type,
    &body.host_expression,
  )
  .await
  .map_err(to_handler_error)?;

  let params = service::power::ApplyPowerParams {
    action: match body.action {
      PowerAction::On => service::power::PowerAction::On,
      PowerAction::Off => service::power::PowerAction::Off,
      PowerAction::Reset => service::power::PowerAction::Reset,
    },
    xnames,
    force: body.force,
  };
  let result = service::power::apply_power(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/v1/power/transitions/{id} — Snapshot a PCS transition
// ---------------------------------------------------------------------------

/// `GET /api/v1/power/transitions/{id}` — fetch the current snapshot
/// of a PCS power transition (status, task counts, per-task detail).
/// Called by the CLI's poll loop after `POST /power` returns the id.
#[utoipa::path(get, path = "/power/transitions/{id}", tag = "power",
  params(SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Transition snapshot",  body = serde_json::Value),
    (status = 401, description = "Unauthorized",         body = ErrorResponse),
    (status = 404, description = "Unknown transition id",body = ErrorResponse),
    (status = 500, description = "Internal error",       body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_power_transition(
  ctx: RequestCtx,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::debug!("get_power_transition id={id}");
  let infra = ctx.infra();
  let snapshot = service::power::get_power_transition(&infra, &ctx.token, &id)
    .await
    .map_err(to_handler_error)?;
  Ok(Json(snapshot))
}
