//! POST /api/v1/power.

use axum::{Json, http::StatusCode, response::IntoResponse};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
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
  pub targets_expression: String,
  /// Indicates whether `targets_expression` is a node expression or a cluster name.
  pub target_type: PowerTargetType,
  /// Pass `--force` to the underlying power operation (forceful shutdown/reset).
  #[serde(default)]
  pub force: bool,
}

/// `POST /api/v1/power` — power on, off, or reset nodes or all members of a cluster.
#[utoipa::path(post, path = "/power", tag = "power",
  params(SiteHeader),
  request_body = PowerRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Power operation result", body = serde_json::Value),
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
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let xnames: Vec<String> = match body.target_type {
    PowerTargetType::Cluster => infra
      .backend
      .get_member_vec_from_group_name_vec(
        &token,
        std::slice::from_ref(&body.targets_expression),
      )
      .await
      .map_err(to_handler_error)?,
    PowerTargetType::Nodes => {
      crate::server::common::node_ops::resolve_hosts_expression(
        infra.backend,
        &token,
        &body.targets_expression,
        false,
      )
      .await
      .map_err(to_handler_error)?
    }
  };

  if xnames.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "No nodes to operate on".into(),
      }),
    ));
  }

  let params = service::power::ApplyPowerParams {
    action: match body.action {
      PowerAction::On => service::power::PowerAction::On,
      PowerAction::Off => service::power::PowerAction::Off,
      PowerAction::Reset => service::power::PowerAction::Reset,
    },
    xnames,
    force: body.force,
  };
  let result = service::power::apply_power(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(result))
}
