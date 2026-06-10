//! Migrate nodes/backup/restore handlers.

use std::path::{Path, PathBuf};

use axum::{Json, http::StatusCode, response::IntoResponse};
use manta_backend_dispatcher::error::Error as BackendError;
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;
use manta_backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;

use super::{ErrorResponse, RequestCtx, SiteHeader, to_handler_error};
use crate::service;

pub use manta_shared::types::wire::migrate::{
  MigrateBackupRequest, MigrateNodesRequest, MigrateRestoreRequest,
};

/// Resolve a user-supplied filesystem path against the configured
/// `migrate_backup_root` and reject anything that escapes it.
///
/// Used by both migrate-backup (where the path is a `destination` to
/// be written) and migrate-restore (where it is a file to be read).
/// For backup the destination may not exist yet, so we canonicalise
/// the nearest existing ancestor and append the not-yet-existing
/// suffix back. Symlinks are followed by `canonicalize`, so a
/// symlink pointing outside the root fails the `starts_with` check.
///
/// Returns the canonicalised path on success — callers should forward
/// THAT instead of the original user input to close the
/// canonicalise-then-open TOCTOU window.
fn confine_to_root(
  user_path: &str,
  backup_root: &Path,
) -> Result<PathBuf, BackendError> {
  let candidate = Path::new(user_path);

  // Reject relative paths up-front: they would resolve against the
  // server's CWD, which is unrelated to backup_root.
  if !candidate.is_absolute() {
    return Err(BackendError::BadRequest(format!(
      "migrate path '{user_path}' must be absolute"
    )));
  }

  // Walk up until we find an existing prefix; lets the destination
  // file/dir not exist yet for backup writes.
  let mut existing: &Path = candidate;
  while !existing.exists() {
    existing = existing.parent().ok_or_else(|| {
      BackendError::BadRequest(format!(
        "migrate path '{user_path}' has no existing ancestor"
      ))
    })?;
  }

  let resolved_existing = existing.canonicalize().map_err(|e| {
    BackendError::BadRequest(format!(
      "could not resolve migrate path '{}': {e}",
      existing.display()
    ))
  })?;

  if !resolved_existing.starts_with(backup_root) {
    return Err(BackendError::BadRequest(format!(
      "migrate path '{user_path}' resolves outside the configured \
       migrate_backup_root '{}'",
      backup_root.display()
    )));
  }

  let suffix = candidate
    .strip_prefix(existing)
    .expect("existing is a prefix of candidate by construction");
  Ok(resolved_existing.join(suffix))
}

/// Validate every `Some(path)` in `paths` against `backup_root`.
/// Returns the canonicalised paths in the same order so the caller
/// can forward those instead of the original user input.
fn confine_all(
  paths: &[Option<&str>],
  backup_root: &Path,
) -> Result<Vec<Option<String>>, BackendError> {
  paths
    .iter()
    .map(|p| {
      p.map(|raw| {
        confine_to_root(raw, backup_root)
          .map(|pb| pb.to_string_lossy().into_owned())
      })
      .transpose()
    })
    .collect()
}

/// Resolve `state.migrate_backup_root` or reject with `BadRequest`.
/// Operators must opt in to server-side filesystem writes — there is
/// no built-in default root.
fn require_backup_root(ctx: &RequestCtx) -> Result<&Path, BackendError> {
  ctx.state.migrate_backup_root.as_deref().ok_or_else(|| {
    BackendError::BadRequest(
      "migrate endpoints disabled: server has no [server] migrate_backup_root \
         configured. Set it to an absolute, existing directory and restart."
        .to_string(),
    )
  })
}

/// `POST /api/v1/migrate/nodes` — move nodes between HSM groups.
#[utoipa::path(post, path = "/migrate/nodes", tag = "migrate",
  params(SiteHeader),
  request_body = MigrateNodesRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Migration result", body = manta_shared::types::wire::responses::MigrateNodesResponse),
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
    (status = 200, description = "Backup completed",      body = manta_shared::types::wire::responses::CompletedResponse),
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
    return Err(to_handler_error(BackendError::BadRequest(
      "migrate backup requires admin privileges".to_string(),
    )));
  }

  // Confine the destination to `[server] migrate_backup_root`. Even
  // admin tokens can't write outside that directory.
  let backup_root = require_backup_root(&ctx).map_err(to_handler_error)?;
  let confined = confine_all(&[body.destination.as_deref()], backup_root)
    .map_err(to_handler_error)?;
  let destination = confined.into_iter().next().flatten();

  infra
    .backend
    .migrate_backup(&ctx.token, body.bos.as_deref(), destination.as_deref())
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
    (status = 200, description = "Restore completed",  body = manta_shared::types::wire::responses::CompletedResponse),
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
    return Err(to_handler_error(BackendError::BadRequest(
      "migrate restore requires admin privileges".to_string(),
    )));
  }

  // Confine every supplied file path to `[server] migrate_backup_root`.
  // The five paths are independent (some restores omit subsets), so
  // we validate them through a uniform helper that preserves
  // None-ness.
  let backup_root = require_backup_root(&ctx).map_err(to_handler_error)?;
  let confined = confine_all(
    &[
      body.bos_file.as_deref(),
      body.cfs_file.as_deref(),
      body.hsm_file.as_deref(),
      body.ims_file.as_deref(),
      body.image_dir.as_deref(),
    ],
    backup_root,
  )
  .map_err(to_handler_error)?;
  let mut iter = confined.into_iter();
  let bos_file = iter.next().flatten();
  let cfs_file = iter.next().flatten();
  let hsm_file = iter.next().flatten();
  let ims_file = iter.next().flatten();
  let image_dir = iter.next().flatten();

  infra
    .backend
    .migrate_restore(
      &ctx.token,
      bos_file.as_deref(),
      cfs_file.as_deref(),
      hsm_file.as_deref(),
      ims_file.as_deref(),
      image_dir.as_deref(),
      body.overwrite,
      body.overwrite,
      body.overwrite,
      body.overwrite,
    )
    .await
    .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({ "completed": true })))
}

#[cfg(test)]
mod tests {
  use super::*;

  // Pin the four lines of defence `confine_to_root` enforces. The
  // helper is the only thing standing between an admin token and the
  // server process's full filesystem write capability, so regressions
  // here would re-open the migrate-backup arbitrary-write surface.
  use std::fs;

  fn tmp_root() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
  }

  /// Production code canonicalises `migrate_backup_root` once at
  /// startup (see `main.rs`) — the helper assumes that contract.
  /// On macOS the tempdir lives under `/var/folders/...` which is
  /// itself a symlink, so tests must canonicalise too.
  fn canonical(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().canonicalize().expect("canonical tempdir")
  }

  #[test]
  fn accepts_existing_file_under_root() {
    let root = tmp_root();
    let canon = canonical(&root);
    let file = canon.join("bos.yaml");
    fs::write(&file, "").unwrap();
    let resolved = confine_to_root(file.to_str().unwrap(), &canon).unwrap();
    assert!(resolved.starts_with(&canon));
  }

  #[test]
  fn accepts_yet_to_exist_destination_when_parent_is_under_root() {
    let root = tmp_root();
    let canon = canonical(&root);
    let dest = canon.join("new-subdir").join("dest.tar");
    let resolved = confine_to_root(dest.to_str().unwrap(), &canon).unwrap();
    assert!(resolved.starts_with(&canon));
    assert!(resolved.ends_with("dest.tar"));
  }

  #[test]
  fn rejects_relative_path() {
    let root = tmp_root();
    let canon = canonical(&root);
    let err = confine_to_root("relative/path.yaml", &canon).unwrap_err();
    assert!(matches!(err, BackendError::BadRequest(_)));
  }

  #[test]
  fn rejects_path_outside_root() {
    let root = tmp_root();
    let other = tmp_root();
    let canon = canonical(&root);
    let file = canonical(&other).join("hsm.yaml");
    fs::write(&file, "").unwrap();
    let err = confine_to_root(file.to_str().unwrap(), &canon).unwrap_err();
    assert!(matches!(err, BackendError::BadRequest(_)));
  }

  #[test]
  fn rejects_symlink_that_escapes_root() {
    let root = tmp_root();
    let outside = tmp_root();
    let canon = canonical(&root);
    let target = canonical(&outside).join("secret.yaml");
    fs::write(&target, "").unwrap();
    let link = canon.join("link.yaml");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target, &link).unwrap();
    let err = confine_to_root(link.to_str().unwrap(), &canon).unwrap_err();
    assert!(matches!(err, BackendError::BadRequest(_)));
  }

  #[test]
  fn rejects_dotdot_traversal() {
    let root = tmp_root();
    let canon = canonical(&root);
    let escape = format!("{}/../escape", canon.display());
    let err = confine_to_root(&escape, &canon).unwrap_err();
    assert!(matches!(err, BackendError::BadRequest(_)));
  }
}
