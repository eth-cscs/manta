//! Session handlers (get/create/delete + log streaming).

use std::convert::Infallible;

use axum::{
  Json,
  extract::{Path, Query},
  http::StatusCode,
  response::{
    IntoResponse,
    sse::{Event, KeepAlive, Sse},
  },
};
use futures::{AsyncBufReadExt, StreamExt};
use manta_backend_dispatcher::types::{K8sAuth, K8sDetails};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, require_k8s_url, require_vault,
  serialize_or_500, to_handler_error, validate_repo_list_lengths,
};
use crate::service;

// ---------------------------------------------------------------------------
// GET /api/v1/sessions
// ---------------------------------------------------------------------------

/// Query parameters for `GET /sessions`.
#[derive(Deserialize, IntoParams)]
pub struct SessionQuery {
  /// HSM group whose sessions should be returned.
  pub hsm_group: Option<String>,
  /// Filter to sessions whose `ansible_limit` mentions any of these
  /// comma-separated xnames.
  pub xnames: Option<String>,
  /// Lower-bound session age expressed as a duration string
  /// (e.g. `"1h"`, `"2d"`).
  pub min_age: Option<String>,
  /// Upper-bound session age expressed as a duration string.
  pub max_age: Option<String>,
  /// Session type filter: `"image"` or `"runtime"`.
  pub session_type: Option<String>,
  /// Status filter: `"pending"`, `"running"`, or `"complete"`.
  pub status: Option<String>,
  /// Exact session name.
  pub name: Option<String>,
  /// Cap on the number of sessions returned (most recent first).
  pub limit: Option<u8>,
}

/// GET /sessions — list CFS sessions with optional filters.
#[utoipa::path(get, path = "/sessions", tag = "sessions",
  params(SessionQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "List of sessions", body = serde_json::Value),
    (status = 400, description = "Bad request",      body = ErrorResponse),
    (status = 401, description = "Unauthorized",     body = ErrorResponse),
    (status = 500, description = "Internal error",   body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_sessions(
  ctx: RequestCtx,
  Query(q): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = ctx.infra();

  let xnames: Vec<String> = q
    .xnames
    .map(|s| {
      s.split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect()
    })
    .unwrap_or_default();

  let params = service::session::GetSessionParams {
    group: q.hsm_group,
    xnames,
    min_age: q.min_age,
    max_age: q.max_age,
    session_type: q.session_type,
    status: q.status,
    name: q.name,
    limit: q.limit,
  };

  let sessions = service::session::get_sessions(&infra, &ctx.token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(sessions))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/sessions/{name} — with ?dry_run=true support
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /sessions/{name}`.
#[derive(Deserialize, IntoParams)]
pub struct DeleteSessionQuery {
  /// When true, return deletion context without actually deleting (default: false).
  #[serde(default)]
  pub dry_run: bool,
}

/// DELETE /sessions/{name} — cancel and delete a CFS session; `?dry_run=true` previews.
#[utoipa::path(delete, path = "/sessions/{name}", tag = "sessions",
  params(("name" = String, Path, description = "Session name"), DeleteSessionQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "Session deleted or deletion preview", body = serde_json::Value),
    (status = 401, description = "Unauthorized",                        body = ErrorResponse),
    (status = 404, description = "Not found",                           body = ErrorResponse),
    (status = 500, description = "Internal error",                      body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_session(
  ctx: RequestCtx,
  Path(name): Path<String>,
  Query(q): Query<DeleteSessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_session name={} dry_run={}", name, q.dry_run);
  let infra = ctx.infra();

  let deletion_ctx =
    service::session::prepare_session_deletion(&infra, &ctx.token, &name, None)
      .await
      .map_err(to_handler_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&deletion_ctx)?)));
  }

  service::session::execute_session_deletion(
    &infra,
    &ctx.token,
    &deletion_ctx,
    false,
  )
  .await
  .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": name }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sessions — Create CFS session
// ---------------------------------------------------------------------------

pub use manta_shared::types::wire::session::CreateSessionRequest;

/// `POST /api/v1/sessions` — create a CFS session from one or more git repositories.
#[utoipa::path(post, path = "/sessions", tag = "sessions",
  params(SiteHeader),
  request_body = CreateSessionRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 201, description = "Session created",               body = serde_json::Value),
    (status = 400, description = "Bad request",                   body = ErrorResponse),
    (status = 401, description = "Unauthorized",                  body = ErrorResponse),
    (status = 500, description = "Internal error",                body = ErrorResponse),
    (status = 501, description = "Vault not configured",          body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn create_session(
  ctx: RequestCtx,
  Json(body): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  validate_repo_list_lengths(&body.repo_names, &body.repo_last_commit_ids)?;
  tracing::info!("create_session repos={:?}", body.repo_names);
  let infra = ctx.infra();

  // Authorization: requested HSM group must be accessible to the token.
  if let Some(ref hsm_group) = body.hsm_group {
    service::authorization::validate_user_group_access(
      &infra, &ctx.token, hsm_group,
    )
    .await
    .map_err(to_handler_error)?;
  }

  // Authorization: every xname in ansible_limit must belong to a group
  // the token can access.
  if let Some(ref ansible_limit) = body.ansible_limit {
    service::authorization::validate_ansible_limit_membership_access(
      &infra,
      &ctx.token,
      ansible_limit,
    )
    .await
    .map_err(to_handler_error)?;
  }

  let vault_base_url = require_vault(infra.vault_base_url)?;

  let gitea_token =
    crate::server::common::vault::http_client::get_shasta_vcs_token(
      &ctx.token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(to_handler_error)?;

  let repo_name_refs: Vec<&str> = body
    .repo_names
    .iter()
    .map(std::string::String::as_str)
    .collect();
  let repo_commit_refs: Vec<&str> = body
    .repo_last_commit_ids
    .iter()
    .map(std::string::String::as_str)
    .collect();

  let (session_name, config_name) = service::session::create_cfs_session(
    &infra,
    &ctx.token,
    &gitea_token,
    body.cfs_conf_sess_name.as_deref(),
    body.playbook_yaml_file_name.as_deref(),
    body.hsm_group.as_deref(),
    &repo_name_refs,
    &repo_commit_refs,
    body.ansible_limit.as_deref(),
    body.ansible_verbosity.as_deref(),
    body.ansible_passthrough.as_deref(),
  )
  .await
  .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({
      "session_name": session_name,
      "configuration_name": config_name,
    })),
  ))
}

// ---------------------------------------------------------------------------
// GET /api/v1/sessions/{name}/logs — Stream CFS session logs via SSE
// ---------------------------------------------------------------------------

/// Query parameters for `GET /sessions/{name}/logs`.
#[derive(Deserialize, IntoParams)]
pub struct SessionLogsQuery {
  /// When true, prefix each log line with its timestamp.
  #[serde(default)]
  pub timestamps: bool,
}

/// `GET /api/v1/sessions/{name}/logs` — stream CFS session pod logs via Server-Sent Events.
#[utoipa::path(get, path = "/sessions/{name}/logs", tag = "sessions",
  params(("name" = String, Path, description = "Session name"), SessionLogsQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 200, description = "SSE log stream"),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured",    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all)]
pub async fn get_session_logs(
  ctx: RequestCtx,
  Path(name): Path<String>,
  Query(q): Query<SessionLogsQuery>,
) -> Result<
  Sse<impl futures::Stream<Item = Result<Event, Infallible>>>,
  (StatusCode, Json<ErrorResponse>),
> {
  let infra = ctx.infra();

  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;
  let vault_base_url = require_vault(infra.vault_base_url)?;

  // Authorization: the caller's accessible groups must overlap the
  // session's target.groups. Session logs frequently carry
  // credentials, kernel-cmdline secrets, and ansible variable dumps;
  // without this check any authenticated user could stream any
  // session's logs.
  service::session::validate_session_access(&infra, &ctx.token, &name)
    .await
    .map_err(to_handler_error)?;

  let k8s = K8sDetails {
    api_url: k8s_api_url.to_string(),
    authentication: K8sAuth::Vault {
      base_url: vault_base_url.to_string(),
    },
  };

  let logs_stream = infra
    .get_session_logs_stream(&ctx.token, &name, q.timestamps, &k8s)
    .await
    .map_err(to_handler_error)?;

  let sse_stream = logs_stream.lines().map(|result| {
    Ok::<Event, Infallible>(
      Event::default().data(result.unwrap_or_else(|e| format!("error: {e}"))),
    )
  });

  Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}
