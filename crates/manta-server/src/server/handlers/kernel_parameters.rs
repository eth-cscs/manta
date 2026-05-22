//! Kernel-parameters handlers (get/apply/add/delete).

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::{
  ErrorResponse, RequestCtx, SiteHeader, default_true,
  resolve_xnames_from_request, serialize_or_500, to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /kernel-parameters`.
#[derive(Deserialize, IntoParams)]
pub struct KernelParametersQuery {
  /// HSM group whose members' kernel parameters should be returned.
  pub hsm_group: Option<String>,
  /// Explicit comma-separated xnames; mutually exclusive with
  /// `hsm_group`.
  pub nodes: Option<String>,
}

/// GET /kernel-parameters — fetch BSS kernel parameters for a group or node list.
#[utoipa::path(get, path = "/kernel-parameters", tag = "kernel-parameters",
  params(KernelParametersQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Kernel parameters", body = serde_json::Value),
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
    hsm_group: q.hsm_group,
    nodes: q.nodes,
    settings_hsm_group_name: None,
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

/// Which kernel-parameter mutation to perform (`add`, `apply`, or `delete`).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum KernelParamOp {
  /// Merge new parameters into the existing set.
  Add,
  /// Replace the entire parameter set.
  Apply,
  /// Remove named parameters from the existing set.
  Delete,
}

/// Request body for `POST /kernel-parameters/apply`.
#[derive(Deserialize, ToSchema)]
pub struct ApplyKernelParametersRequest {
  /// Hosts expression (xnames, nids, or hostlist notation); mutually exclusive with `hsm_group`.
  pub xnames_expression: Option<String>,
  /// Target HSM group; all members are resolved to xnames.
  pub hsm_group: Option<String>,
  /// Which mutation to perform: add, apply (replace), or delete.
  pub operation: KernelParamOp,
  /// Space-separated kernel parameter key=value pairs.
  pub params: String,
  /// Only relevant for the `add` operation.
  #[serde(default)]
  pub overwrite: bool,
  /// Whether to project SBPS images (default true).
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  /// When true, returns the computed changeset without applying it.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/kernel-parameters/apply` — add, replace, or delete kernel parameters on nodes.
#[utoipa::path(post, path = "/kernel-parameters/apply", tag = "kernel-parameters",
  params(SiteHeader),
  request_body = ApplyKernelParametersRequest,
  security(("bearerAuth" = [])),
  responses(
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
    infra.backend,
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

/// Request body for `POST /kernel-parameters/add`.
#[derive(Deserialize, ToSchema)]
pub struct AddKernelParametersRequest {
  /// Space-separated kernel parameter key=value pairs to add.
  pub params: String,
  /// Hosts expression (xnames, nids, or hostlist notation); mutually exclusive with `hsm_group`.
  pub xnames_expression: Option<String>,
  /// Target HSM group; all members are resolved to xnames.
  pub hsm_group: Option<String>,
  /// When true, overwrite parameters that already exist.
  #[serde(default)]
  pub overwrite: bool,
  /// Whether to project SBPS images (default true).
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  /// When true, returns the computed changeset without applying it.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/kernel-parameters/add` — merge new kernel parameters into existing node BSS entries.
#[utoipa::path(post, path = "/kernel-parameters/add", tag = "kernel-parameters",
  params(SiteHeader),
  request_body = AddKernelParametersRequest,
  security(("bearerAuth" = [])),
  responses(
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
    infra.backend,
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

/// Request body for `DELETE /kernel-parameters`.
#[derive(Deserialize, ToSchema)]
pub struct DeleteKernelParametersRequest {
  /// Space-separated kernel parameter names (or key=value pairs) to remove.
  pub params: String,
  /// Hosts expression (xnames, nids, or hostlist notation); mutually exclusive with `hsm_group`.
  pub xnames_expression: Option<String>,
  /// Target HSM group; all members are resolved to xnames.
  pub hsm_group: Option<String>,
  /// When true, returns the computed changeset without applying it.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/kernel-parameters` — remove named kernel parameters from node BSS entries.
#[utoipa::path(delete, path = "/kernel-parameters", tag = "kernel-parameters",
  params(SiteHeader),
  request_body = DeleteKernelParametersRequest,
  security(("bearerAuth" = [])),
  responses(
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
    infra.backend,
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
