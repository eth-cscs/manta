//! Migrate nodes/backup/restore handlers.

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/nodes — Migrate nodes between HSM groups
// ---------------------------------------------------------------------------

/// Request body for `POST /migrate/nodes`.
#[derive(Deserialize, ToSchema)]
pub struct MigrateNodesRequest {
  /// Destination HSM group names to move nodes into.
  pub target_hsm_names: Vec<String>,
  /// Source HSM group names the nodes currently belong to.
  pub parent_hsm_names: Vec<String>,
  /// Node-set expression selecting which nodes to migrate.
  pub hosts_expression: String,
  /// When true, validates the migration plan without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
  /// Create the target HSM group if it does not already exist.
  #[serde(default)]
  pub create_hsm_group: bool,
}

/// `POST /api/v1/migrate/nodes` — move nodes between HSM groups.
#[utoipa::path(post, path = "/migrate/nodes", tag = "migrate",
  params(SiteHeader),
  request_body = MigrateNodesRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Migration result", body = serde_json::Value),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn migrate_nodes(
  ctx: RequestCtx,
  Json(body): Json<MigrateNodesRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_nodes dry_run={}", body.dry_run);
  let infra = ctx.infra();

  // Authorization: every named group on both sides must be accessible.
  for name in body
    .target_hsm_names
    .iter()
    .chain(body.parent_hsm_names.iter())
  {
    service::group::validate_hsm_group_access(&infra, &ctx.token, name)
      .await
      .map_err(to_handler_error)?;
  }

  let (xnames, results) = service::migrate::migrate_nodes(
    &infra,
    &ctx.token,
    &body.target_hsm_names,
    &body.parent_hsm_names,
    &body.hosts_expression,
    body.dry_run,
    body.create_hsm_group,
  )
  .await
  .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "xnames": xnames,
    "results": results,
  })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/backup — Backup BOS session templates
// ---------------------------------------------------------------------------

/// Request body for `POST /migrate/backup`.
#[derive(Deserialize, ToSchema)]
pub struct MigrateBackupRequest {
  /// BOS session template name (or filter) to back up.
  pub bos: Option<String>,
  /// Filesystem path where backup files will be written.
  pub destination: Option<String>,
}

/// `POST /api/v1/migrate/backup` — export BOS session templates to backup files.
#[utoipa::path(post, path = "/migrate/backup", tag = "migrate",
  params(SiteHeader),
  request_body = MigrateBackupRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Backup completed",      body = serde_json::Value),
    (status = 401, description = "Unauthorized",          body = ErrorResponse),
    (status = 500, description = "Internal error",        body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn migrate_backup(
  ctx: RequestCtx,
  Json(body): Json<MigrateBackupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_backup");
  let infra = ctx.infra();

  infra
    .migrate_backup(
      &ctx.token,
      body.bos.as_deref(),
      body.destination.as_deref(),
    )
    .await
    .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({ "completed": true })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/restore — Restore from backup files
// ---------------------------------------------------------------------------

/// Request body for `POST /migrate/restore`.
#[derive(Deserialize, ToSchema)]
pub struct MigrateRestoreRequest {
  /// Path to the BOS session template backup file.
  pub bos_file: Option<String>,
  /// Path to the CFS configuration backup file.
  pub cfs_file: Option<String>,
  /// Path to the HSM group backup file.
  pub hsm_file: Option<String>,
  /// Path to the IMS image metadata backup file.
  pub ims_file: Option<String>,
  /// Directory containing the image layer tarballs.
  pub image_dir: Option<String>,
  /// When true, overwrite existing resources that conflict with the backup.
  #[serde(default)]
  pub overwrite: bool,
}

/// `POST /api/v1/migrate/restore` — import BOS session templates and related artifacts from backup.
#[utoipa::path(post, path = "/migrate/restore", tag = "migrate",
  params(SiteHeader),
  request_body = MigrateRestoreRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Restore completed",  body = serde_json::Value),
    (status = 401, description = "Unauthorized",       body = ErrorResponse),
    (status = 500, description = "Internal error",     body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn migrate_restore(
  ctx: RequestCtx,
  Json(body): Json<MigrateRestoreRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_restore overwrite={}", body.overwrite);
  let infra = ctx.infra();

  infra
    .migrate_restore(
      &ctx.token,
      body.bos_file.as_deref(),
      body.cfs_file.as_deref(),
      body.hsm_file.as_deref(),
      body.ims_file.as_deref(),
      body.image_dir.as_deref(),
      body.overwrite,
    )
    .await
    .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({ "completed": true })))
}
