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
use manta_backend_dispatcher::{
  interfaces::cfs::CfsTrait,
  types::{K8sAuth, K8sDetails},
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

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
  pub hsm_group: Option<String>,
  pub xnames: Option<String>,
  pub min_age: Option<String>,
  pub max_age: Option<String>,
  pub session_type: Option<String>,
  pub status: Option<String>,
  pub name: Option<String>,
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
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let xnames = match q.xnames {
    Some(expr) => crate::server::common::node_ops::resolve_hosts_expression(
      infra.backend,
      &token,
      &expr,
      false,
    )
    .await
    .map_err(to_handler_error)?,
    None => vec![],
  };

  let params = service::session::GetSessionParams {
    hsm_group: q.hsm_group,
    xnames,
    min_age: q.min_age,
    max_age: q.max_age,
    session_type: q.session_type,
    status: q.status,
    name: q.name,
    limit: q.limit,
  };

  let sessions = service::session::get_sessions(&infra, &token, &params)
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
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let deletion_ctx =
    service::session::prepare_session_deletion(&infra, &token, &name, None)
      .await
      .map_err(to_handler_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&deletion_ctx)?)));
  }

  service::session::execute_session_deletion(
    &infra,
    &token,
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

/// Request body for `POST /sessions`.
#[derive(Deserialize, ToSchema)]
pub struct CreateSessionRequest {
  /// Explicit name for the CFS session and configuration; auto-generated when absent.
  pub cfs_conf_sess_name: Option<String>,
  /// Ansible playbook filename inside the repository.
  pub playbook_yaml_file_name: Option<String>,
  /// Target HSM group name.
  pub hsm_group: Option<String>,
  /// Git repository names (parallel-indexed with `repo_last_commit_ids`).
  pub repo_names: Vec<String>,
  /// Git commit SHAs matching each entry in `repo_names`.
  pub repo_last_commit_ids: Vec<String>,
  /// Ansible `--limit` expression to restrict which hosts are targeted.
  pub ansible_limit: Option<String>,
  /// Ansible verbosity level (e.g. `"-v"`, `"-vvv"`).
  pub ansible_verbosity: Option<String>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
}

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
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  // Authorization: requested HSM group must be accessible to the token.
  if let Some(ref hsm_group) = body.hsm_group {
    service::group::validate_hsm_group_access(&infra, &token, hsm_group)
      .await
      .map_err(to_handler_error)?;
  }
  // Authorization: every xname in ansible_limit must belong to a group
  // the token can access.
  if let Some(ref ansible_limit) = body.ansible_limit {
    let xnames: Vec<String> = ansible_limit
      .split(',')
      .map(|s| s.trim().to_string())
      .collect();
    crate::server::common::authorization::validate_target_hsm_members(
      infra.backend,
      &token,
      &xnames,
    )
    .await
    .map_err(to_handler_error)?;
  }

  let vault_base_url = require_vault(infra.vault_base_url)?;

  let gitea_token =
    crate::server::common::vault::http_client::fetch_shasta_vcs_token(
      &token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(to_handler_error)?;

  let repo_name_refs: Vec<&str> =
    body.repo_names.iter().map(|s| s.as_str()).collect();
  let repo_commit_refs: Vec<&str> = body
    .repo_last_commit_ids
    .iter()
    .map(|s| s.as_str())
    .collect();

  let (session_name, config_name) = service::session::create_cfs_session(
    &infra,
    &token,
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
  let (state, token, site_name) = ctx.into_parts();
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;
  let vault_base_url = require_vault(infra.vault_base_url)?;

  let k8s = K8sDetails {
    api_url: k8s_api_url.to_string(),
    authentication: K8sAuth::Vault {
      base_url: vault_base_url.to_string(),
    },
  };

  let logs_stream = infra
    .backend
    .get_session_logs_stream(&token, infra.site_name, &name, q.timestamps, &k8s)
    .await
    .map_err(to_handler_error)?;

  let sse_stream = logs_stream.lines().map(|result| {
    Ok::<Event, Infallible>(
      Event::default().data(result.unwrap_or_else(|e| format!("error: {}", e))),
    )
  });

  Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}
