//! POST /api/v1/ephemeral-env.

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};

// ---------------------------------------------------------------------------
// POST /api/v1/ephemeral-env — Create ephemeral CFS environment
// ---------------------------------------------------------------------------

/// Request body for `POST /ephemeral-env`.
#[derive(Deserialize, ToSchema)]
pub struct CreateEphemeralEnvRequest {
  /// IMS image ID to boot the ephemeral environment from.
  pub image_id: String,
}

/// `POST /api/v1/ephemeral-env` — launch an ephemeral CFS environment from an IMS image.
#[utoipa::path(post, path = "/ephemeral-env", tag = "ephemeral-env",
  params(SiteHeader),
  request_body = CreateEphemeralEnvRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Ephemeral env created", body = manta_shared::types::wire::responses::EphemeralEnvResponse),
    (status = 401, description = "Unauthorized",          body = ErrorResponse),
    (status = 500, description = "Internal error",        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn create_ephemeral_env(
  ctx: RequestCtx,
  Json(body): Json<CreateEphemeralEnvRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("create_ephemeral_env image_id={}", body.image_id);
  let infra = ctx.infra();

  let hostname =
    crate::service::ephemeral_env::exec(&infra, &ctx.token, &body.image_id)
      .await
      .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({ "hostname": hostname })),
  ))
}
