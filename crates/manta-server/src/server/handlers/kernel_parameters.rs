//! Kernel-parameter handlers.
//!
//! - `GET    /api/v1/kernel-parameters`         → [`get_kernel_parameters`]
//! - `POST   /api/v1/kernel-parameters/apply`   → [`apply_kernel_parameters`]
//! - `POST   /api/v1/kernel-parameters/add`     → [`add_kernel_parameters`]
//! - `DELETE /api/v1/kernel-parameters`         → [`delete_kernel_parameters`]
//!
//! All wrap `crate::service::kernel_parameters::*`. Target nodes can
//! be supplied as an xname expression or an HSM group name; the
//! shared [`super::resolve_xnames_from_request`] helper does the
//! resolution.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};

use super::{
  ErrorResponse, RequestCtx, SiteHeader, resolve_xnames_from_request,
  serialize_or_500, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::queries::KernelParametersQuery;

/// GET /kernel-parameters — fetch BSS kernel parameters for a group or node list.
#[utoipa::path(get, path = "/kernel-parameters", tag = "kernel-parameters",
  params(KernelParametersQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Kernel parameters", body = Vec<manta_backend_dispatcher::types::bss::BootParameters>),
    (status = 401, description = "Unauthorized",      body = ErrorResponse),
    (status = 500, description = "Internal error",    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_kernel_parameters(
  ctx: RequestCtx,
  Query(q): Query<KernelParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let params = service::kernel_parameters::GetKernelParametersParams {
    group_name: q.hsm_group,
    nodes: q.nodes,
    settings_group_name: None,
  };

  let kernel_params = service::kernel_parameters::get_kernel_parameters(
    &infra, &ctx.token, &params,
  )
  .await
  .map_err(to_handler_error)?;

  Ok(Json(kernel_params))
}

// ---------------------------------------------------------------------------
// POST /api/v1/kernel-parameters/apply — Apply kernel parameter changes
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::kernel_parameters::{
  ApplyKernelParametersRequest, KernelParamOp,
};

/// `POST /api/v1/kernel-parameters/apply` — add, replace, or delete kernel parameters on nodes.
#[utoipa::path(post, path = "/kernel-parameters/apply", tag = "kernel-parameters",
  params(SiteHeader),
  request_body = ApplyKernelParametersRequest,
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
    (status = 200, description = "Kernel parameters applied or preview", body = serde_json::Value),
    (status = 400, description = "Bad request",                          body = ErrorResponse),
    (status = 401, description = "Unauthorized",                         body = ErrorResponse),
    (status = 500, description = "Internal error",                       body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn apply_kernel_parameters(
  ctx: RequestCtx,
  Json(body): Json<ApplyKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let xnames = resolve_xnames_from_request(
    &infra,
    &ctx.token,
    body.xnames_expression.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!(
    "apply_kernel_parameters xnames={:?} op={:?} dry_run={}",
    xnames,
    body.operation,
    body.dry_run
  );

  let operation = match body.operation {
    KernelParamOp::Add => {
      service::kernel_parameters::KernelParamOperation::Add {
        params: &body.params,
        overwrite: body.overwrite,
      }
    }
    KernelParamOp::Apply => {
      service::kernel_parameters::KernelParamOperation::Apply {
        params: &body.params,
      }
    }
    KernelParamOp::Delete => {
      service::kernel_parameters::KernelParamOperation::Delete {
        params: &body.params,
      }
    }
  };

  let changeset = service::kernel_parameters::prepare_kernel_params_changes(
    &infra, &ctx.token, &xnames, &operation,
  )
  .await
  .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  let images_to_project = service::kernel_parameters::build_images_to_project(
    &changeset,
    body.project_sbps,
  );

  service::kernel_parameters::apply_kernel_params_changes(
    &infra,
    &ctx.token,
    &changeset,
    &images_to_project,
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({
      "applied": true,
      "has_changes": changeset.has_changes,
      "xnames_to_reboot": changeset.xnames_to_reboot,
    })),
  ))
}

// ---------------------------------------------------------------------------
// POST /api/v1/kernel-parameters/add
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::kernel_parameters::AddKernelParametersRequest;

/// `POST /api/v1/kernel-parameters/add` — merge new kernel parameters into existing node BSS entries.
#[utoipa::path(post, path = "/kernel-parameters/add", tag = "kernel-parameters",
  params(SiteHeader),
  request_body = AddKernelParametersRequest,
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
    (status = 200, description = "Parameters added or preview", body = serde_json::Value),
    (status = 400, description = "Bad request",                 body = ErrorResponse),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 500, description = "Internal error",              body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn add_kernel_parameters(
  ctx: RequestCtx,
  Json(body): Json<AddKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();
  let xnames = resolve_xnames_from_request(
    &infra,
    &ctx.token,
    body.xnames_expression.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!(
    "add_kernel_parameters xnames={:?} dry_run={}",
    xnames,
    body.dry_run
  );

  let operation = service::kernel_parameters::KernelParamOperation::Add {
    params: &body.params,
    overwrite: body.overwrite,
  };

  let changeset = service::kernel_parameters::prepare_kernel_params_changes(
    &infra, &ctx.token, &xnames, &operation,
  )
  .await
  .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  let images_to_project = service::kernel_parameters::build_images_to_project(
    &changeset,
    body.project_sbps,
  );

  service::kernel_parameters::apply_kernel_params_changes(
    &infra,
    &ctx.token,
    &changeset,
    &images_to_project,
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({
      "applied": true,
      "has_changes": changeset.has_changes,
      "xnames_to_reboot": changeset.xnames_to_reboot,
    })),
  ))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::kernel_parameters::DeleteKernelParametersRequest;

/// `DELETE /api/v1/kernel-parameters` — remove named kernel parameters from node BSS entries.
#[utoipa::path(delete, path = "/kernel-parameters", tag = "kernel-parameters",
  params(SiteHeader),
  request_body = DeleteKernelParametersRequest,
  security(("bearerAuth" = [])),
  responses(
    // dry_run/real result union — kept as Value until the union shape is formalised
    (status = 200, description = "Parameters removed or preview", body = serde_json::Value),
    (status = 400, description = "Bad request",                   body = ErrorResponse),
    (status = 401, description = "Unauthorized",                  body = ErrorResponse),
    (status = 500, description = "Internal error",                body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_kernel_parameters(
  ctx: RequestCtx,
  Json(body): Json<DeleteKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();
  let xnames = resolve_xnames_from_request(
    &infra,
    &ctx.token,
    body.xnames_expression.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!(
    "delete_kernel_parameters xnames={:?} dry_run={}",
    xnames,
    body.dry_run
  );

  let operation = service::kernel_parameters::KernelParamOperation::Delete {
    params: &body.params,
  };

  let changeset = service::kernel_parameters::prepare_kernel_params_changes(
    &infra, &ctx.token, &xnames, &operation,
  )
  .await
  .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  service::kernel_parameters::apply_kernel_params_changes(
    &infra,
    &ctx.token,
    &changeset,
    &std::collections::HashMap::new(),
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::OK,
    Json(serde_json::json!({
      "applied": true,
      "has_changes": changeset.has_changes,
      "xnames_to_reboot": changeset.xnames_to_reboot,
    })),
  ))
}
