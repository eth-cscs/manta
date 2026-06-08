//! Migrate nodes/backup/restore handlers.

use axum::{Json, http::StatusCode, response::IntoResponse};

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

pub use manta_shared::types::wire::migrate::{
  MigrateBackupRequest, MigrateNodesRequest, MigrateRestoreRequest,
};

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
    service::authorization::validate_user_group_access(
      &infra, &ctx.token, name,
    )
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

  // Authorization: backup writes to a server-side filesystem path
  // chosen by the caller. Restrict to admin to prevent
  // non-privileged users from triggering arbitrary writes via the
  // server process's UID.
  if !crate::server::common::jwt_ops::is_user_admin(&ctx.token) {
    return Err(to_handler_error(
      manta_backend_dispatcher::error::Error::BadRequest(
        "migrate backup requires admin privileges".to_string(),
      ),
    ));
  }

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

  // Authorization: restore reads from server-side filesystem paths
  // chosen by the caller and rewrites CFS/HSM/IMS state — high
  // blast radius. Restrict to admin.
  if !crate::server::common::jwt_ops::is_user_admin(&ctx.token) {
    return Err(to_handler_error(
      manta_backend_dispatcher::error::Error::BadRequest(
        "migrate restore requires admin privileges".to_string(),
      ),
    ));
  }

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
