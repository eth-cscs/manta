//! Template + post_template_session handlers.

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::IntoResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::{
  ErrorResponse, RequestCtx, SiteHeader, serialize_or_500, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/templates
// ---------------------------------------------------------------------------

/// Query parameters for `GET /templates`.
#[derive(Deserialize, IntoParams)]
pub struct TemplateQuery {
  /// Exact template name.
  pub name: Option<String>,
  /// HSM group whose associated templates should be returned.
  pub hsm_group: Option<String>,
  /// Cap on the number of templates returned (most recent first).
  pub limit: Option<u8>,
}

/// GET /templates — list BOS session templates with optional filters.
#[utoipa::path(get, path = "/templates", tag = "templates",
  params(TemplateQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of session templates", body = serde_json::Value),
    (status = 401, description = "Unauthorized",              body = ErrorResponse),
    (status = 500, description = "Internal error",            body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_templates(
  ctx: RequestCtx,
  Query(q): Query<TemplateQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = service::template::GetTemplateParams {
    name: q.name,
    group_name: q.hsm_group,
    settings_hsm_group_name: None,
    limit: q.limit,
  };

  let templates = service::template::get_templates(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(templates))
}

// ---------------------------------------------------------------------------
// POST /api/v1/templates/{name}/sessions — Create BOS session from template
// ---------------------------------------------------------------------------

/// BOS session operation to run against the template's node list.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BosOperation {
  /// Boot nodes that are currently off.
  Boot,
  /// Reboot (power-cycle) nodes.
  Reboot,
  /// Shut down nodes.
  Shutdown,
}

impl BosOperation {
  fn as_str(&self) -> &'static str {
    match self {
      Self::Boot => "boot",
      Self::Reboot => "reboot",
      Self::Shutdown => "shutdown",
    }
  }
}

/// Request body for `POST /templates/{name}/sessions`.
#[derive(Deserialize, ToSchema)]
pub struct PostTemplateSessionRequest {
  /// BOS operation to run (boot, reboot, or shutdown).
  pub operation: BosOperation,
  /// Ansible limit expression restricting which template nodes are targeted.
  pub limit: String,
  /// Optional explicit name for the BOS session.
  pub session_name: Option<String>,
  /// When true, include nodes marked as disabled.
  #[serde(default)]
  pub include_disabled: bool,
  /// When true, validates the session parameters without creating a BOS session.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/templates/{name}/sessions` — create a BOS session from a session template.
#[utoipa::path(post, path = "/templates/{name}/sessions", tag = "templates",
  params(("name" = String, Path, description = "Template name"), SiteHeader),
  request_body = PostTemplateSessionRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Dry run preview",  body = serde_json::Value),
    (status = 201, description = "Session created",  body = serde_json::Value),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn post_template_session(
  ctx: RequestCtx,
  Path(name): Path<String>,
  Json(body): Json<PostTemplateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_template_session template={} op={:?} dry_run={}",
    name,
    body.operation,
    body.dry_run
  );
  let infra = ctx.infra();

  let params = service::template::ApplyTemplateParams {
    bos_session_name: body.session_name,
    bos_sessiontemplate_name: name,
    bos_session_operation: body.operation.as_str().to_string(),
    limit: body.limit,
    include_disabled: body.include_disabled,
  };

  let (bos_session, _) =
    service::template::validate_and_prepare_template_session(
      &infra, &ctx.token, &params,
    )
    .await
    .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&bos_session)?)));
  }

  let created =
    service::template::create_bos_session(&infra, &ctx.token, bos_session)
      .await
      .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json(serialize_or_500(&created)?)))
}
