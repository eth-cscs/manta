//! Axum handler functions for every HTTP and WebSocket endpoint, plus shared
//! request/response types and the bearer-token extractor.

use std::sync::Arc;
use std::convert::Infallible;

use axum::{
  Json,
  extract::{FromRequestParts, Path, Query, State, ws::{Message, WebSocket, WebSocketUpgrade}},
  http::{StatusCode, header, request::Parts},
  response::{
    IntoResponse,
    sse::{Event, KeepAlive, Sse},
  },
};
use futures::{AsyncBufReadExt, StreamExt};
use manta_backend_dispatcher::{
  error::Error as BackendError,
  interfaces::{
    cfs::CfsTrait,
    console::ConsoleTrait,
    hsm::group::GroupTrait,
    pcs::PCSTrait,
  },
  types::{K8sAuth, K8sDetails},
};
use tokio::io::AsyncWriteExt;
use serde::{Deserialize, Serialize};

use super::ServerState;
use crate::service;

// ---------------------------------------------------------------------------
// Bearer-token extractor — eliminates token-extraction boilerplate
// ---------------------------------------------------------------------------

/// Axum extractor that pulls the token from `Authorization: Bearer <token>`.
pub struct BearerToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for BearerToken {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let auth_header = parts
      .headers
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .ok_or_else(|| {
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorResponse {
            error: "Missing Authorization header".to_string(),
          }),
        )
      })?;

    let token = auth_header
      .strip_prefix("Bearer ")
      .or_else(|| auth_header.strip_prefix("bearer "))
      .ok_or_else(|| {
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorResponse {
            error: "Authorization header must use Bearer scheme".to_string(),
          }),
        )
      })?;

    Ok(BearerToken(token.to_string()))
  }
}

/// Axum extractor that reads the target site name from `X-Manta-Site`.
///
/// Every handler that touches backend APIs requires this header so the server
/// knows which site's CA certificate, base URL, and credentials to use.
pub struct SiteName(pub String);

impl<S: Send + Sync> FromRequestParts<S> for SiteName {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let site = parts
      .headers
      .get("X-Manta-Site")
      .and_then(|v| v.to_str().ok())
      .ok_or_else(|| {
        (
          StatusCode::BAD_REQUEST,
          Json(ErrorResponse {
            error: "Missing X-Manta-Site header".to_string(),
          }),
        )
      })?;
    Ok(SiteName(site.to_string()))
  }
}

/// Convert a `BackendError` into the best-fitting HTTP error response.
pub(crate) fn to_handler_error(e: BackendError) -> (StatusCode, Json<ErrorResponse>) {
  let status = match &e {
    BackendError::NotFound(_)
    | BackendError::SessionNotFound
    | BackendError::ConfigurationNotFound => StatusCode::NOT_FOUND,
    BackendError::Conflict(_)
    | BackendError::ConfigurationAlreadyExistsError(_) => StatusCode::CONFLICT,
    BackendError::BadRequest(_)
    | BackendError::InvalidPattern(_)
    | BackendError::UnsupportedBackend(_)
    | BackendError::InvalidNodeId(_) => StatusCode::BAD_REQUEST,
    BackendError::AuthenticationTokenNotFound(_)
    | BackendError::JwtMalformed(_) => StatusCode::UNAUTHORIZED,
    BackendError::InsufficientResources(_) => StatusCode::UNPROCESSABLE_ENTITY,
    _ => StatusCode::INTERNAL_SERVER_ERROR,
  };
  if status == StatusCode::INTERNAL_SERVER_ERROR {
    tracing::error!("Internal error: {}", e);
  } else {
    tracing::debug!("Service error {}: {}", status, e);
  }
  (status, Json(ErrorResponse { error: e.to_string() }))
}

/// Convert any `Display` error (e.g. anyhow) into an HTTP error response.
fn display_error<E: std::fmt::Display>(e: E) -> (StatusCode, Json<ErrorResponse>) {
  to_handler_error(BackendError::Message(e.to_string()))
}

fn serialize_or_500<T: Serialize>(v: &T) -> Result<serde_json::Value, (StatusCode, Json<ErrorResponse>)> {
  serde_json::to_value(v).map_err(|e| {
    let msg = format!("Failed to serialize: {}", e);
    tracing::error!("{}", msg);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: msg }))
  })
}

fn require_vault(url: Option<&str>) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
  url.ok_or_else(|| (StatusCode::NOT_IMPLEMENTED, Json(ErrorResponse { error: "vault_base_url not configured on this server".into() })))
}

fn require_k8s_url(url: Option<&str>) -> Result<&str, (StatusCode, Json<ErrorResponse>)> {
  url.ok_or_else(|| (StatusCode::NOT_IMPLEMENTED, Json(ErrorResponse { error: "k8s_api_url not configured on this server".into() })))
}

fn validate_repo_list_lengths(
  repo_names: &[String],
  repo_last_commit_ids: &[String],
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
  if repo_names.len() != repo_last_commit_ids.len() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!(
          "repo_names ({}) and repo_last_commit_ids ({}) must have the same length",
          repo_names.len(),
          repo_last_commit_ids.len()
        ),
      }),
    ));
  }
  Ok(())
}

fn parse_iso_datetime(
  field: &str,
  value: &str,
) -> Result<chrono::NaiveDateTime, (StatusCode, Json<ErrorResponse>)> {
  chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: format!("Invalid '{}' datetime '{}': {}", field, value, e),
      }),
    )
  })
}

// ---------------------------------------------------------------------------
// Shared response types
// ---------------------------------------------------------------------------

/// Standard JSON error body returned by all failed endpoints.
#[derive(Serialize)]
pub struct ErrorResponse {
  pub error: String,
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// GET /health — liveness probe; returns `{"status":"ok"}`.
#[tracing::instrument(skip_all)]
pub async fn health() -> impl IntoResponse {
  Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/sessions
// ---------------------------------------------------------------------------

/// Query parameters for `GET /sessions`.
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn get_sessions(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let xnames = match q.xnames {
    Some(expr) => crate::common::node_ops::resolve_hosts_expression(
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
// GET /api/v1/configurations
// ---------------------------------------------------------------------------

/// Query parameters for `GET /configurations`.
#[derive(Deserialize)]
pub struct ConfigurationQuery {
  pub name: Option<String>,
  pub pattern: Option<String>,
  pub hsm_group: Option<String>,
  pub limit: Option<u8>,
}

/// GET /configurations — list CFS configurations with optional name/pattern/group filters.
#[tracing::instrument(skip_all)]
pub async fn get_configurations(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<ConfigurationQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::configuration::GetConfigurationParams {
    name: q.name,
    pattern: q.pattern,
    hsm_group: q.hsm_group,
    settings_hsm_group_name: None,
    since: None,
    until: None,
    limit: q.limit,
  };

  let configs =
    service::configuration::get_configurations(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(configs))
}

// ---------------------------------------------------------------------------
// GET /api/v1/nodes
// ---------------------------------------------------------------------------

/// Query parameters for `GET /nodes`.
#[derive(Deserialize)]
pub struct NodesQuery {
  pub xname: String,
  /// Expand results to include nodes sharing the same power supply.
  pub include_siblings: Option<bool>,
  pub status: Option<String>,
}

/// GET /nodes — fetch node details for a given xname expression.
#[tracing::instrument(skip_all)]
pub async fn get_nodes(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<NodesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::node::GetNodesParams {
    xname: q.xname,
    include_siblings: q.include_siblings.unwrap_or(false),
    status_filter: q.status,
  };

  let nodes = service::node::get_nodes(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(nodes))
}

// ---------------------------------------------------------------------------
// GET /api/v1/groups
// ---------------------------------------------------------------------------

/// Query parameters for `GET /groups`.
#[derive(Deserialize)]
pub struct GroupQuery {
  pub name: Option<String>,
}

/// GET /groups — list HSM groups, optionally filtered by name.
#[tracing::instrument(skip_all)]
pub async fn get_groups(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<GroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::group::GetGroupParams {
    group_name: q.name,
    settings_hsm_group_name: None,
  };

  let groups = service::group::get_groups(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(groups))
}

// ---------------------------------------------------------------------------
// GET /api/v1/images
// ---------------------------------------------------------------------------

/// Query parameters for `GET /images`.
#[derive(Deserialize)]
pub struct ImageQuery {
  pub id: Option<String>,
  pub hsm_group: Option<String>,
  pub limit: Option<u8>,
}

/// Wrapper so the image tuple serializes to named fields.
#[derive(Serialize)]
pub struct ImageEntry {
  pub image: serde_json::Value,
  pub configuration_name: String,
  pub image_id: String,
  pub is_linked: bool,
}

/// GET /images — list IMS images with their associated CFS configuration names.
#[tracing::instrument(skip_all)]
pub async fn get_images(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<ImageQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::image::GetImagesParams {
    id: q.id,
    hsm_group: q.hsm_group,
    settings_hsm_group_name: None,
    limit: q.limit,
  };

  let images = service::image::get_images(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  let mut entries = Vec::with_capacity(images.len());
  for (img, config_name, image_id, linked) in images {
    let image = serialize_or_500(&img)?;
    entries.push(ImageEntry {
      image,
      configuration_name: config_name,
      image_id,
      is_linked: linked,
    });
  }

  Ok(Json(entries))
}

// ---------------------------------------------------------------------------
// GET /api/v1/templates
// ---------------------------------------------------------------------------

/// Query parameters for `GET /templates`.
#[derive(Deserialize)]
pub struct TemplateQuery {
  pub name: Option<String>,
  pub hsm_group: Option<String>,
  pub limit: Option<u8>,
}

/// GET /templates — list BOS session templates with optional filters.
#[tracing::instrument(skip_all)]
pub async fn get_templates(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<TemplateQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::template::GetTemplateParams {
    name: q.name,
    hsm_group: q.hsm_group,
    settings_hsm_group_name: None,
    limit: q.limit,
  };

  let templates = service::template::get_templates(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(templates))
}

// ---------------------------------------------------------------------------
// GET /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /boot-parameters`.
#[derive(Deserialize)]
pub struct BootParametersQuery {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
}

/// GET /boot-parameters — fetch BSS boot parameters for a group or node list.
#[tracing::instrument(skip_all)]
pub async fn get_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<BootParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::boot_parameters::GetBootParametersParams {
    hsm_group: q.hsm_group,
    nodes: q.nodes,
    settings_hsm_group_name: None,
  };

  let boot_params =
    service::boot_parameters::get_boot_parameters(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(boot_params))
}

// ---------------------------------------------------------------------------
// GET /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /kernel-parameters`.
#[derive(Deserialize)]
pub struct KernelParametersQuery {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
}

/// GET /kernel-parameters — fetch BSS kernel parameters for a group or node list.
#[tracing::instrument(skip_all)]
pub async fn get_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<KernelParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::kernel_parameters::GetKernelParametersParams {
    hsm_group: q.hsm_group,
    nodes: q.nodes,
    settings_hsm_group_name: None,
  };

  let kernel_params =
    service::kernel_parameters::get_kernel_parameters(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(kernel_params))
}

// ---------------------------------------------------------------------------
// GET /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

/// Query parameters for `GET /redfish-endpoints`.
#[derive(Deserialize)]
pub struct RedfishEndpointsQuery {
  pub id: Option<String>,
  pub fqdn: Option<String>,
  pub uuid: Option<String>,
  pub macaddr: Option<String>,
  pub ipaddress: Option<String>,
}

/// GET /redfish-endpoints — list HSM Redfish endpoints with optional filters.
#[tracing::instrument(skip_all)]
pub async fn get_redfish_endpoints(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<RedfishEndpointsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::redfish_endpoints::GetRedfishEndpointsParams {
    id: q.id,
    fqdn: q.fqdn,
    uuid: q.uuid,
    macaddr: q.macaddr,
    ipaddress: q.ipaddress,
  };

  let endpoints =
    service::redfish_endpoints::get_redfish_endpoints(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(endpoints))
}

// ---------------------------------------------------------------------------
// GET /api/v1/clusters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /clusters`.
#[derive(Deserialize)]
pub struct ClusterQuery {
  pub hsm_group: Option<String>,
  pub status: Option<String>,
}

/// GET /clusters — list cluster nodes with optional group/status filters.
#[tracing::instrument(skip_all)]
pub async fn get_clusters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<ClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::cluster::GetClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
    status_filter: q.status,
  };

  let nodes = service::cluster::get_cluster_nodes(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(nodes))
}

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-clusters
// ---------------------------------------------------------------------------

/// Query parameters for `GET /hardware-clusters`.
#[derive(Deserialize)]
pub struct HardwareClusterQuery {
  pub hsm_group: Option<String>,
}

/// GET /hardware-clusters — summarize hardware components per node for a cluster.
#[tracing::instrument(skip_all)]
pub async fn get_hardware_clusters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<HardwareClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::hardware::GetHardwareClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
  };

  let result = service::hardware::get_hardware_cluster(&infra, &token, &params)
    .await
    .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "hsm_group_name": result.hsm_group_name,
    "node_summaries": result.node_summaries,
  })))
}

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-nodes-list
// ---------------------------------------------------------------------------

/// Query parameters for `GET /hardware-nodes-list`.
#[derive(Deserialize)]
pub struct HardwareNodesListQuery {
  /// Comma-separated xnames.
  pub xnames: String,
}

/// GET /hardware-nodes-list — hardware details for an explicit list of xnames.
#[tracing::instrument(skip_all)]
pub async fn get_hardware_nodes_list(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<HardwareNodesListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params =
    service::hardware::GetHardwareNodesListParams { xnames: q.xnames };

  let result =
    service::hardware::get_hardware_nodes_list(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(serde_json::json!({
    "node_summaries": result.node_summaries,
  })))
}

// ===========================================================================
// WRITE ENDPOINTS
// ===========================================================================

// ---------------------------------------------------------------------------
// DELETE /api/v1/nodes/{id}
// ---------------------------------------------------------------------------

/// DELETE /nodes/{id} — remove a node from HSM by xname or NID.
#[tracing::instrument(skip_all)]
pub async fn delete_node(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_node id={}", id);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::node::delete_node(&infra, &token, &id)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/nodes
// ---------------------------------------------------------------------------

/// Body for `POST /nodes`.
#[derive(Deserialize)]
pub struct AddNodeRequest {
  pub id: String,
  pub group: String,
  #[serde(default)]
  pub enabled: bool,
  pub arch: Option<String>,
}

/// POST /nodes — register a new node in HSM and add it to a group.
#[tracing::instrument(skip_all)]
pub async fn add_node(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<AddNodeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_node id={} group={}", body.id, body.group);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::node::add_node(
    &infra,
    &token,
    &body.id,
    &body.group,
    body.enabled,
    body.arch,
    None, // hardware_file_path not applicable via HTTP
  )
  .await
  .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": body.id }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{label}
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /groups/{label}`.
#[derive(Deserialize)]
pub struct DeleteGroupQuery {
  /// Delete even if the group still has members (default: false).
  #[serde(default)]
  pub force: bool,
}

/// DELETE /groups/{label} — remove an HSM group.
#[tracing::instrument(skip_all)]
pub async fn delete_group(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(label): Path<String>,
  Query(q): Query<DeleteGroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_group label={} force={}", label, q.force);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::group::delete_group(&infra, &token, &label, q.force)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups
// ---------------------------------------------------------------------------

/// POST /groups — create a new HSM group.
#[tracing::instrument(skip_all)]
pub async fn create_group(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(group): Json<::manta_backend_dispatcher::types::Group>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("create_group");
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::group::create_group(&infra, &token, group)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups/{name}/members
// ---------------------------------------------------------------------------

/// Body for `POST /groups/{name}/members`.
#[derive(Deserialize)]
pub struct AddNodesToGroupRequest {
  pub hosts_expression: String,
}

/// Response for `POST /groups/{name}/members`.
#[derive(Serialize)]
pub struct AddNodesToGroupResponse {
  pub added: Vec<String>,
  pub removed: Vec<String>,
}

/// POST /groups/{name}/members — replace a group's member list from a host expression.
#[tracing::instrument(skip_all)]
pub async fn add_nodes_to_group(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(name): Path<String>,
  Json(body): Json<AddNodesToGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "add_nodes_to_group group={} hosts={}",
    name,
    body.hosts_expression
  );
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let (added, removed) =
    service::group::add_nodes_to_group(&infra, &token, &name, &body.hosts_expression)
      .await
      .map_err(to_handler_error)?;

  Ok(Json(AddNodesToGroupResponse { added, removed }))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// Body for `DELETE /boot-parameters`.
#[derive(Deserialize)]
pub struct DeleteBootParametersRequest {
  pub hosts: Vec<String>,
}

/// DELETE /boot-parameters — remove BSS boot parameter entries for specified hosts.
#[tracing::instrument(skip_all)]
pub async fn delete_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<DeleteBootParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  if body.hosts.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "hosts list must not be empty".to_string(),
      }),
    ));
  }
  tracing::info!("delete_boot_parameters hosts={:?}", body.hosts);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::boot_parameters::delete_boot_parameters(&infra, &token, body.hosts)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// POST /boot-parameters — create a new BSS boot parameters entry.
#[tracing::instrument(skip_all)]
pub async fn add_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(boot_params): Json<::manta_backend_dispatcher::types::bss::BootParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_boot_parameters");
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::boot_parameters::add_boot_parameters(&infra, &token, &boot_params)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/boot-parameters
// ---------------------------------------------------------------------------

/// PUT /boot-parameters — update boot image, kernel params, or runtime config for nodes.
#[tracing::instrument(skip_all)]
pub async fn update_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(params): Json<service::boot_parameters::UpdateBootParametersParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("update_boot_parameters");
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::boot_parameters::update_boot_parameters(&infra, &token, params)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/redfish-endpoints/{id}
// ---------------------------------------------------------------------------

/// DELETE /redfish-endpoints/{id} — remove a Redfish endpoint from HSM.
#[tracing::instrument(skip_all)]
pub async fn delete_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_redfish_endpoint id={}", id);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::redfish_endpoints::delete_redfish_endpoint(&infra, &token, &id)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

/// POST /redfish-endpoints — register a new Redfish endpoint in HSM.
#[tracing::instrument(skip_all)]
pub async fn add_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(params): Json<service::redfish_endpoints::UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_redfish_endpoint");
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::redfish_endpoints::add_redfish_endpoint(&infra, &token, params)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

/// PUT /redfish-endpoints — update an existing Redfish endpoint's properties.
#[tracing::instrument(skip_all)]
pub async fn update_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(params): Json<service::redfish_endpoints::UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("update_redfish_endpoint");
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::redfish_endpoints::update_redfish_endpoint(&infra, &token, params)
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/sessions/{name} — with ?dry_run=true support
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /sessions/{name}`.
#[derive(Deserialize)]
pub struct DeleteSessionQuery {
  /// When true, return deletion context without actually deleting (default: false).
  #[serde(default)]
  pub dry_run: bool,
}

/// DELETE /sessions/{name} — cancel and delete a CFS session; `?dry_run=true` previews.
#[tracing::instrument(skip_all)]
pub async fn delete_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(name): Path<String>,
  Query(q): Query<DeleteSessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_session name={} dry_run={}", name, q.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let deletion_ctx =
    service::session::prepare_session_deletion(&infra, &token, &name, None)
      .await
      .map_err(to_handler_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&deletion_ctx)?)));
  }

  service::session::execute_session_deletion(&infra, &token, &deletion_ctx, false)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": name }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/images — with ?ids=id1,id2&dry_run=true
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /images`.
#[derive(Deserialize)]
pub struct DeleteImagesQuery {
  /// Comma-separated list of IMS image IDs to delete.
  pub ids: String,
  /// When true, validates deletion eligibility without removing anything.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/images` — delete IMS images by ID; validates only when `dry_run=true`.
#[tracing::instrument(skip_all)]
pub async fn delete_images(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<DeleteImagesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_images ids={} dry_run={}", q.ids, q.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let id_strings: Vec<String> = q.ids.split(',').map(|s| s.trim().to_string()).collect();
  let id_refs: Vec<&str> = id_strings.iter().map(|s| s.as_str()).collect();

  if q.dry_run {
    service::image::validate_image_deletion(&infra, &token, &id_refs, None)
      .await
      .map_err(to_handler_error)?;
    return Ok((StatusCode::OK, Json(serde_json::json!({ "validated_ids": id_strings }))));
  }

  let deleted = service::image::delete_images(&infra, &token, &id_refs, None)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": deleted }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/configurations — with ?pattern=...&since=...&until=...&dry_run=true
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /configurations`.
#[derive(Deserialize)]
pub struct DeleteConfigurationsQuery {
  /// Glob pattern to match configuration names.
  pub pattern: Option<String>,
  /// ISO-8601 lower bound — only delete configurations created after this date.
  pub since: Option<String>,
  /// ISO-8601 upper bound — only delete configurations created before this date.
  pub until: Option<String>,
  /// When true, returns deletion candidates without removing anything.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/configurations` — delete CFS configurations and all derived artifacts.
#[tracing::instrument(skip_all)]
pub async fn delete_configurations(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Query(q): Query<DeleteConfigurationsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_configurations dry_run={}", q.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let since = q.since.as_deref().map(|s| parse_iso_datetime("since", s)).transpose()?;
  let until = q.until.as_deref().map(|s| parse_iso_datetime("until", s)).transpose()?;

  let candidates = service::configuration::get_deletion_candidates(
    &infra,
    &token,
    None,
    q.pattern.as_deref(),
    since,
    until,
  )
  .await
  .map_err(to_handler_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&candidates)?)));
  }

  service::configuration::delete_configurations_and_derivatives(&infra, &token, &candidates)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "deleted_configurations": candidates.configuration_names,
    "deleted_images": candidates.image_ids,
  }))))
}

// ===========================================================================
// BATCH A — MEDIUM-COMPLEXITY WRITE ENDPOINTS
// ===========================================================================

// ---------------------------------------------------------------------------
// POST /api/v1/sessions — Create CFS session
// ---------------------------------------------------------------------------

/// Request body for `POST /sessions`.
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn create_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  validate_repo_list_lengths(&body.repo_names, &body.repo_last_commit_ids)?;
  tracing::info!("create_session repos={:?}", body.repo_names);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let vault_base_url = require_vault(infra.vault_base_url)?;

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(&token, vault_base_url, infra.site_name)
      .await
      .map_err(to_handler_error)?;

  let repo_name_refs: Vec<&str> = body.repo_names.iter().map(|s| s.as_str()).collect();
  let repo_commit_refs: Vec<&str> = body.repo_last_commit_ids.iter().map(|s| s.as_str()).collect();

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
// POST /api/v1/boot-config — Apply boot configuration (with ?dry_run=true)
// ---------------------------------------------------------------------------

/// Request body for `POST /boot-config`.
#[derive(Deserialize)]
pub struct ApplyBootConfigRequest {
  /// Node-set expression (xnames, HSM group, or nodeset) identifying the target nodes.
  pub hosts_expression: String,
  /// IMS image ID to set as the boot image.
  pub boot_image_id: Option<String>,
  /// CFS configuration name associated with the boot image.
  pub boot_image_configuration: Option<String>,
  /// Kernel command-line parameters to apply.
  pub kernel_parameters: Option<String>,
  /// CFS configuration to assign as the runtime desired-config.
  pub runtime_configuration: Option<String>,
  /// When true, returns the computed changeset without persisting it.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/boot-config` — apply BSS boot configuration to a set of nodes.
#[tracing::instrument(skip_all)]
pub async fn apply_boot_config(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<ApplyBootConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "apply_boot_config hosts={} dry_run={}",
    body.hosts_expression,
    body.dry_run
  );
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let changeset = service::boot_parameters::prepare_boot_config(
    &infra,
    &token,
    &body.hosts_expression,
    body.boot_image_id.as_deref(),
    body.boot_image_configuration.as_deref(),
    body.kernel_parameters.as_deref(),
  )
  .await
  .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  service::boot_parameters::persist_boot_config(
    &infra,
    &token,
    &changeset,
    body.runtime_configuration.as_deref(),
  )
  .await
  .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "nodes": changeset.xname_vec,
    "need_restart": changeset.need_restart,
  }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/kernel-parameters/apply — Apply kernel parameter changes
// ---------------------------------------------------------------------------

/// Which kernel-parameter mutation to perform (`add`, `apply`, or `delete`).
#[derive(Debug, Deserialize)]
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
#[derive(Deserialize)]
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

fn default_true() -> bool {
  true
}

/// `POST /api/v1/kernel-parameters/apply` — add, replace, or delete kernel parameters on nodes.
#[tracing::instrument(skip_all)]
pub async fn apply_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<ApplyKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let xnames = resolve_xnames_from_request(
    infra.backend,
    &token,
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
    KernelParamOp::Add => service::kernel_parameters::KernelParamOperation::Add {
      params: &body.params,
      overwrite: body.overwrite,
    },
    KernelParamOp::Apply => service::kernel_parameters::KernelParamOperation::Apply {
      params: &body.params,
    },
    KernelParamOp::Delete => service::kernel_parameters::KernelParamOperation::Delete {
      params: &body.params,
    },
  };

  let changeset =
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &xnames, &operation)
      .await
      .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  let images_to_project =
    service::kernel_parameters::build_images_to_project(&changeset, body.project_sbps);

  service::kernel_parameters::apply_kernel_params_changes(&infra, &token, &changeset, &images_to_project)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "has_changes": changeset.has_changes,
    "xnames_to_reboot": changeset.xnames_to_reboot,
  }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/nodes — Migrate nodes between HSM groups
// ---------------------------------------------------------------------------

/// Request body for `POST /migrate/nodes`.
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn migrate_nodes(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<MigrateNodesRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_nodes dry_run={}", body.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let (xnames, results) = service::migrate::migrate_nodes(
    &infra,
    &token,
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
#[derive(Deserialize)]
pub struct MigrateBackupRequest {
  /// BOS session template name (or filter) to back up.
  pub bos: Option<String>,
  /// Filesystem path where backup files will be written.
  pub destination: Option<String>,
}

/// `POST /api/v1/migrate/backup` — export BOS session templates to backup files.
#[tracing::instrument(skip_all)]
pub async fn migrate_backup(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<MigrateBackupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_backup");
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::migrate::migrate_backup(
    &infra,
    &token,
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
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn migrate_restore(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<MigrateRestoreRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_restore overwrite={}", body.overwrite);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  service::migrate::migrate_restore(
    &infra,
    &token,
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

// ---------------------------------------------------------------------------
// POST /api/v1/ephemeral-env — Create ephemeral CFS environment
// ---------------------------------------------------------------------------

/// Request body for `POST /ephemeral-env`.
#[derive(Deserialize)]
pub struct CreateEphemeralEnvRequest {
  /// IMS image ID to boot the ephemeral environment from.
  pub image_id: String,
}

/// `POST /api/v1/ephemeral-env` — launch an ephemeral CFS environment from an IMS image.
#[tracing::instrument(skip_all)]
pub async fn create_ephemeral_env(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<CreateEphemeralEnvRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("create_ephemeral_env image_id={}", body.image_id);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let hostname =
    crate::service::ephemeral_env::exec(&infra, &token, &body.image_id)
      .await
      .map_err(to_handler_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::json!({ "hostname": hostname })),
  ))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{name}/members — Remove nodes from HSM group
// ---------------------------------------------------------------------------

/// Request body for `DELETE /groups/{name}/members`.
#[derive(Deserialize)]
pub struct DeleteGroupMembersRequest {
  /// Hosts expression (xnames, nids, or hostlist notation) identifying nodes to remove.
  pub xnames_expression: String,
  /// When true, validates the request without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/groups/{name}/members` — remove nodes from an HSM group.
#[tracing::instrument(skip_all)]
pub async fn delete_group_members(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(name): Path<String>,
  Json(body): Json<DeleteGroupMembersRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "delete_group_members group={} xnames_expression={} dry_run={}",
    name,
    body.xnames_expression,
    body.dry_run
  );
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let xnames = crate::common::node_ops::resolve_hosts_expression(
    infra.backend,
    &token,
    &body.xnames_expression,
    false,
  )
  .await
  .map_err(to_handler_error)?;

  if !body.dry_run {
    for xname in &xnames {
      infra
        .backend
        .delete_member_from_group(&token, &name, xname)
        .await
        .map_err(to_handler_error)?;
    }
  }

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/power — Power on/off/reset nodes or cluster
// ---------------------------------------------------------------------------

/// Power action to apply to the target nodes or cluster.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerTargetType {
  /// `targets` is a list of xnames.
  Nodes,
  /// `targets` contains a single HSM group name whose members will be targeted.
  Cluster,
}

/// Request body for `POST /power`.
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn post_power(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<PowerRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_power action={:?} target_type={:?}",
    body.action,
    body.target_type
  );
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let xnames: Vec<String> = match body.target_type {
    PowerTargetType::Cluster => infra
      .backend
      .get_member_vec_from_group_name_vec(&token, &[body.targets_expression.clone()])
      .await
      .map_err(to_handler_error)?,
    PowerTargetType::Nodes => crate::common::node_ops::resolve_hosts_expression(
      infra.backend,
      &token,
      &body.targets_expression,
      false,
    )
    .await
    .map_err(to_handler_error)?,
  };

  if xnames.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "No nodes to operate on".into(),
      }),
    ));
  }

  let result = match body.action {
    PowerAction::On => infra.backend.power_on_sync(&token, &xnames).await,
    PowerAction::Off => infra.backend.power_off_sync(&token, &xnames, body.force).await,
    PowerAction::Reset => infra.backend.power_reset_sync(&token, &xnames, body.force).await,
  }
  .map_err(to_handler_error)?;

  Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/v1/templates/{name}/sessions — Create BOS session from template
// ---------------------------------------------------------------------------

/// BOS session operation to run against the template's node list.
#[derive(Debug, Deserialize)]
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
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn post_template_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(name): Path<String>,
  Json(body): Json<PostTemplateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_template_session template={} op={:?} dry_run={}",
    name,
    body.operation,
    body.dry_run
  );
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let params = service::template::ApplyTemplateParams {
    bos_session_name: body.session_name,
    bos_sessiontemplate_name: name,
    bos_session_operation: body.operation.as_str().to_string(),
    limit: body.limit,
    include_disabled: body.include_disabled,
  };

  let (bos_session, _) =
    service::template::validate_and_prepare_template_session(&infra, &token, &params)
      .await
      .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&bos_session)?)));
  }

  let created =
    service::template::create_bos_session(&infra, &token, bos_session)
      .await
      .map_err(to_handler_error)?;

  Ok((StatusCode::CREATED, Json(serialize_or_500(&created)?)))
}

// ---------------------------------------------------------------------------
// GET /api/v1/sessions/{name}/logs — Stream CFS session logs via SSE
// ---------------------------------------------------------------------------

/// Query parameters for `GET /sessions/{name}/logs`.
#[derive(Deserialize)]
pub struct SessionLogsQuery {
  /// When true, prefix each log line with its timestamp.
  #[serde(default)]
  pub timestamps: bool,
}

/// `GET /api/v1/sessions/{name}/logs` — stream CFS session pod logs via Server-Sent Events.
#[tracing::instrument(skip_all)]
pub async fn get_session_logs(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(name): Path<String>,
  Query(q): Query<SessionLogsQuery>,
) -> Result<
  Sse<impl futures::Stream<Item = Result<Event, Infallible>>>,
  (StatusCode, Json<ErrorResponse>),
> {
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
    Ok::<Event, Infallible>(Event::default().data(
      result.unwrap_or_else(|e| format!("error: {}", e)),
    ))
  });

  Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sat-file — Apply a SAT file
// ---------------------------------------------------------------------------

/// Request body for `POST /sat-file`.
#[derive(Deserialize)]
pub struct PostSatFileRequest {
  /// Raw YAML content of the SAT file to apply.
  pub sat_file_content: String,
  /// Inline Jinja2 variable overrides (merged with `values_file_content`).
  pub values: Option<serde_json::Value>,
  /// Raw YAML content of a values file supplying Jinja2 variable overrides.
  pub values_file_content: Option<String>,
  /// Ansible verbosity level passed to any CFS sessions created.
  pub ansible_verbosity: Option<u8>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
  /// Reboot nodes after applying the SAT file.
  #[serde(default)]
  pub reboot: bool,
  /// Stream CFS session logs after creation.
  #[serde(default)]
  pub watch_logs: bool,
  /// Prefix log lines with timestamps when streaming logs.
  #[serde(default)]
  pub timestamps: bool,
  /// Only process image sections; skip session templates.
  #[serde(default)]
  pub image_only: bool,
  /// Only process session template sections; skip images.
  #[serde(default)]
  pub session_template_only: bool,
  /// Overwrite existing IMS images or BOS session templates.
  #[serde(default)]
  pub overwrite: bool,
  /// When true, validates the SAT file without creating any resources.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/sat-file` — apply a SAT file (images, session templates, and CFS sessions).
#[tracing::instrument(skip_all)]
pub async fn post_sat_file(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<PostSatFileRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_file dry_run={}", body.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(
      &token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(display_error)?;

  service::sat_file::apply_sat_file(
    &infra,
    &token,
    &gitea_token,
    vault_base_url,
    k8s_api_url,
    service::sat_file::ApplySatFileParams {
      sat_file_content: &body.sat_file_content,
      values: body.values.as_ref(),
      values_file_content: body.values_file_content.as_deref(),
      ansible_verbosity: body.ansible_verbosity,
      ansible_passthrough: body.ansible_passthrough.as_deref(),
      reboot: body.reboot,
      watch_logs: body.watch_logs,
      timestamps: body.timestamps,
      image_only: body.image_only,
      session_template_only: body.session_template_only,
      overwrite: body.overwrite,
      dry_run: body.dry_run,
    },
  )
  .await
  .map_err(display_error)?;

  Ok(Json(serde_json::json!({ "applied": true })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/kernel-parameters/add
// ---------------------------------------------------------------------------

/// Request body for `POST /kernel-parameters/add`.
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn add_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<AddKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
  let xnames = resolve_xnames_from_request(
    infra.backend,
    &token,
    body.xnames_expression.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!("add_kernel_parameters xnames={:?} dry_run={}", xnames, body.dry_run);

  let operation = service::kernel_parameters::KernelParamOperation::Add {
    params: &body.params,
    overwrite: body.overwrite,
  };

  let changeset =
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &xnames, &operation)
      .await
      .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  let images_to_project =
    service::kernel_parameters::build_images_to_project(&changeset, body.project_sbps);

  service::kernel_parameters::apply_kernel_params_changes(&infra, &token, &changeset, &images_to_project)
    .await
    .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "has_changes": changeset.has_changes,
    "xnames_to_reboot": changeset.xnames_to_reboot,
  }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

/// Request body for `DELETE /kernel-parameters`.
#[derive(Deserialize)]
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
#[tracing::instrument(skip_all)]
pub async fn delete_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<DeleteKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
  let xnames = resolve_xnames_from_request(
    infra.backend,
    &token,
    body.xnames_expression.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!("delete_kernel_parameters xnames={:?} dry_run={}", xnames, body.dry_run);

  let operation = service::kernel_parameters::KernelParamOperation::Delete {
    params: &body.params,
  };

  let changeset =
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &xnames, &operation)
      .await
      .map_err(to_handler_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  service::kernel_parameters::apply_kernel_params_changes(
    &infra,
    &token,
    &changeset,
    &std::collections::HashMap::new(),
  )
  .await
  .map_err(to_handler_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "has_changes": changeset.has_changes,
    "xnames_to_reboot": changeset.xnames_to_reboot,
  }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/hardware-clusters/{target}/members
// ---------------------------------------------------------------------------

/// Request body for `POST /hardware-clusters/{target}/members`.
#[derive(Deserialize)]
pub struct AddHwComponentRequest {
  /// Source HSM group that donates nodes matching `pattern`.
  pub parent_cluster: String,
  /// Hardware component pattern used to select which nodes to move.
  pub pattern: String,
  /// Create the target HSM group if it does not already exist.
  #[serde(default)]
  pub create_hsm_group: bool,
  /// When true, returns the planned changes without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/hardware-clusters/{target}/members` — move nodes matching a hardware pattern into a cluster.
#[tracing::instrument(skip_all)]
pub async fn add_hw_component(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(target): Path<String>,
  Json(body): Json<AddHwComponentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_hw_component target={} parent={} dry_run={}", target, body.parent_cluster, body.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let result =
    crate::service::hw_cluster::add_hw_component(
      infra.backend,
      &token,
      &target,
      &body.parent_cluster,
      &body.pattern,
      body.dry_run,
      body.create_hsm_group,
    )
    .await
    .map_err(display_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "nodes_moved": result.nodes_moved,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/hardware-clusters/{target}/members
// ---------------------------------------------------------------------------

/// Request body for `DELETE /hardware-clusters/{target}/members`.
#[derive(Deserialize)]
pub struct DeleteHwComponentRequest {
  /// Destination HSM group that receives nodes moved out of the target cluster.
  pub parent_cluster: String,
  /// Hardware component pattern used to select which nodes to move back.
  pub pattern: String,
  /// Delete the target HSM group if it becomes empty after the operation.
  #[serde(default)]
  pub delete_hsm_group: bool,
  /// When true, returns the planned changes without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `DELETE /api/v1/hardware-clusters/{target}/members` — move nodes back to parent cluster by hardware pattern.
#[tracing::instrument(skip_all)]
pub async fn delete_hw_component(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(target): Path<String>,
  Json(body): Json<DeleteHwComponentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_hw_component target={} parent={} dry_run={}", target, body.parent_cluster, body.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let result =
    crate::service::hw_cluster::delete_hw_component(
      infra.backend,
      &token,
      &target,
      &body.parent_cluster,
      &body.pattern,
      body.dry_run,
      body.delete_hsm_group,
    )
    .await
    .map_err(display_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "nodes_moved": result.nodes_moved,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/hardware-clusters/{target}/configuration
// ---------------------------------------------------------------------------

/// Whether to pin nodes to the target cluster or unpin them back to the parent.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HwClusterMode {
  /// Move nodes from the parent cluster into the target cluster.
  #[default]
  Pin,
  /// Move nodes back from the target cluster to the parent cluster.
  Unpin,
}

/// Request body for `POST /hardware-clusters/{target}/configuration`.
#[derive(Deserialize)]
pub struct ApplyHwConfigurationRequest {
  /// Source (parent) HSM group supplying nodes.
  pub parent_cluster: String,
  /// Hardware component pattern selecting which nodes to pin/unpin.
  pub pattern: String,
  /// Whether to pin nodes into the target cluster or unpin back to parent (default: pin).
  #[serde(default)]
  pub mode: HwClusterMode,
  /// Create the target HSM group if absent (default true).
  #[serde(default = "default_true")]
  pub create_target_hsm_group: bool,
  /// Delete the parent HSM group if it becomes empty (default true).
  #[serde(default = "default_true")]
  pub delete_empty_parent_hsm_group: bool,
  /// When true, returns the planned changes without modifying group membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// `POST /api/v1/hardware-clusters/{target}/configuration` — pin or unpin nodes between clusters by hardware pattern.
#[tracing::instrument(skip_all)]
pub async fn apply_hw_configuration(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Path(target): Path<String>,
  Json(body): Json<ApplyHwConfigurationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("apply_hw_configuration target={} parent={} dry_run={}", target, body.parent_cluster, body.dry_run);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;

  let mode = match body.mode {
    HwClusterMode::Pin => crate::service::hw_cluster::HwClusterMode::Pin,
    HwClusterMode::Unpin => crate::service::hw_cluster::HwClusterMode::Unpin,
  };

  let result =
    crate::service::hw_cluster::apply_hw_configuration(
      infra.backend,
      mode,
      &token,
      &target,
      &body.parent_cluster,
      &body.pattern,
      body.dry_run,
      body.create_target_hsm_group,
      body.delete_empty_parent_hsm_group,
    )
    .await
    .map_err(display_error)?;

  Ok(Json(serde_json::json!({
    "dry_run": body.dry_run,
    "target_cluster": target,
    "target_nodes": result.target_nodes,
    "parent_cluster": body.parent_cluster,
    "parent_nodes": result.parent_nodes,
  })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/sessions/apply
// ---------------------------------------------------------------------------

/// Request body for `POST /sessions/apply`.
#[derive(Deserialize)]
pub struct ApplySessionRequest {
  /// Git repository names (parallel-indexed with `repo_last_commit_ids`).
  pub repo_names: Vec<String>,
  /// Git commit SHAs matching each entry in `repo_names`.
  pub repo_last_commit_ids: Vec<String>,
  /// Explicit name for the CFS session and configuration; auto-generated when absent.
  pub cfs_conf_sess_name: Option<String>,
  /// Ansible playbook filename inside the repository.
  pub playbook_yaml_file_name: Option<String>,
  /// Target HSM group name.
  pub hsm_group: Option<String>,
  /// Ansible `--limit` expression to restrict which hosts are targeted.
  pub ansible_limit: Option<String>,
  /// Ansible verbosity level (e.g. `"-v"`, `"-vvv"`).
  pub ansible_verbosity: Option<String>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
}

/// `POST /api/v1/sessions/apply` — create a CFS configuration and session from git repositories.
#[tracing::instrument(skip_all)]
pub async fn apply_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  SiteName(site_name): SiteName,
  Json(body): Json<ApplySessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  validate_repo_list_lengths(&body.repo_names, &body.repo_last_commit_ids)?;

  tracing::info!("apply_session repos={:?}", body.repo_names);
  let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
  let vault_base_url = require_vault(infra.vault_base_url)?;

  if let Some(ref ansible_limit) = body.ansible_limit {
    let xnames: Vec<String> = ansible_limit
      .split(',')
      .map(|s| s.trim().to_string())
      .collect();
    crate::common::authorization::validate_target_hsm_members(infra.backend, &token, &xnames)
      .await
      .map_err(display_error)?;
  }

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(&token, vault_base_url, infra.site_name)
      .await
      .map_err(to_handler_error)?;

  let repo_name_refs: Vec<&str> = body.repo_names.iter().map(|s| s.as_str()).collect();
  let repo_commit_refs: Vec<&str> = body.repo_last_commit_ids.iter().map(|s| s.as_str()).collect();

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
// WS /api/v1/nodes/{xname}/console — Interactive node console
// ---------------------------------------------------------------------------

/// Query parameters for WebSocket console endpoints (initial terminal size).
#[derive(Deserialize)]
pub struct ConsoleQuery {
  /// Initial terminal width in columns (default 80).
  #[serde(default = "default_cols")]
  pub cols: u16,
  /// Initial terminal height in rows (default 24).
  #[serde(default = "default_rows")]
  pub rows: u16,
}

fn default_cols() -> u16 { 80 }
fn default_rows() -> u16 { 24 }

/// `WS /api/v1/nodes/{xname}/console` — attach an interactive PTY console to a node via WebSocket.
#[tracing::instrument(skip_all, fields(xname = %xname))]
pub async fn console_node_ws(
  BearerToken(token): BearerToken,
  State(state): State<Arc<ServerState>>,
  SiteName(site_name): SiteName,
  Path(xname): Path<String>,
  Query(q): Query<ConsoleQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (k8s_api_url, vault_base_url) = {
    let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
    let k = require_k8s_url(infra.k8s_api_url)?.to_string();
    let v = require_vault(infra.vault_base_url)?.to_string();
    (k, v)
  };

  let k8s = K8sDetails {
    api_url: k8s_api_url,
    authentication: K8sAuth::Vault { base_url: vault_base_url },
  };

  let timeout = state.console_inactivity_timeout;
  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for node {xname}");
    if let Some(site) = state.sites.get(&site_name) {
      match site.backend
        .attach_to_node_console(&token, &site_name, &xname, q.cols, q.rows, &k8s)
        .await
      {
        Ok((console_in, console_out)) => {
          run_console_bridge(socket, console_in, console_out, timeout).await;
          tracing::info!("WebSocket console closed for node {xname}");
        }
        Err(e) => {
          tracing::error!("Failed to attach to node console {xname}: {e:#}");
        }
      }
    }
  }))
}

// ---------------------------------------------------------------------------
// WS /api/v1/sessions/{name}/console — Interactive CFS session console
// ---------------------------------------------------------------------------

/// `WS /api/v1/sessions/{name}/console` — attach an interactive PTY console to a CFS session pod via WebSocket.
#[tracing::instrument(skip_all, fields(session = %name))]
pub async fn console_session_ws(
  BearerToken(token): BearerToken,
  State(state): State<Arc<ServerState>>,
  SiteName(site_name): SiteName,
  Path(name): Path<String>,
  Query(q): Query<ConsoleQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (k8s_api_url, vault_base_url) = {
    let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
    let k = require_k8s_url(infra.k8s_api_url)?.to_string();
    let v = require_vault(infra.vault_base_url)?.to_string();
    service::session::validate_console_session(&infra, &token, &name)
      .await
      .map_err(to_handler_error)?;
    (k, v)
  };

  let k8s = K8sDetails {
    api_url: k8s_api_url,
    authentication: K8sAuth::Vault { base_url: vault_base_url },
  };

  let timeout = state.console_inactivity_timeout;
  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for session {name}");
    if let Some(site) = state.sites.get(&site_name) {
      match site.backend
        .attach_to_session_console(&token, &site_name, &name, q.cols, q.rows, &k8s)
        .await
      {
        Ok((console_in, console_out)) => {
          run_console_bridge(socket, console_in, console_out, timeout).await;
          tracing::info!("WebSocket console closed for session {name}");
        }
        Err(e) => {
          tracing::error!("Failed to attach to session console {name}: {e:#}");
        }
      }
    }
  }))
}

/// Bridge a WebSocket connection to a console's stdin/stdout streams.
///
/// - Binary and text WS frames are forwarded as raw bytes to console stdin.
/// - Text frames matching `{"type":"resize","cols":N,"rows":N}` are silently
///   consumed (dynamic resize is not yet supported by the ConsoleTrait).
/// - Console stdout is forwarded as Binary WS frames.
/// - Either side closing or erroring terminates the bridge.
/// - The bridge closes automatically after `inactivity_timeout` of silence
///   from the client, releasing the Kubernetes pod attachment.
async fn run_console_bridge(
  mut socket: WebSocket,
  mut console_in: Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
  console_out: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  inactivity_timeout: std::time::Duration,
) {
  let mut out_stream = tokio_util::io::ReaderStream::new(console_out);
  let mut deadline = tokio::time::Instant::now() + inactivity_timeout;

  loop {
    tokio::select! {
      msg = socket.recv() => {
        match msg {
          Some(Ok(Message::Binary(data))) => {
            deadline = tokio::time::Instant::now() + inactivity_timeout;
            if console_in.write_all(&data).await.is_err() { break; }
          }
          Some(Ok(Message::Text(text))) => {
            deadline = tokio::time::Instant::now() + inactivity_timeout;
            // Consume resize control messages silently; forward everything else.
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
              if v.get("type").and_then(|t| t.as_str()) == Some("resize") {
                continue;
              }
            }
            if console_in.write_all(text.as_bytes()).await.is_err() { break; }
          }
          Some(Ok(Message::Close(_))) | None => break,
          Some(Ok(_)) => {} // Ping/Pong handled by axum automatically
          Some(Err(_)) => break,
        }
      }
      chunk = out_stream.next() => {
        match chunk {
          Some(Ok(data)) => {
            if socket.send(Message::Binary(data)).await.is_err() { break; }
          }
          Some(Err(_)) | None => break,
        }
      }
      _ = tokio::time::sleep_until(deadline) => {
        tracing::warn!("Console session idle for {:?}, closing", inactivity_timeout);
        break;
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Resolve target xnames from an explicit list or an HSM group name.
/// Returns 400 if neither is provided.
async fn resolve_xnames_from_request(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  token: &str,
  xnames_expression: Option<&str>,
  hsm_group: Option<&str>,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
  if let Some(expr) = xnames_expression {
    if !expr.is_empty() {
      return crate::common::node_ops::resolve_hosts_expression(backend, token, expr, false)
        .await
        .map_err(display_error);
    }
  }
  if let Some(group) = hsm_group {
    return crate::common::node_ops::resolve_target_nodes(
      backend,
      token,
      None,
      Some(group),
      None,
    )
    .await
    .map_err(display_error);
  }
  Err((
    StatusCode::BAD_REQUEST,
    Json(ErrorResponse {
      error: "At least one of 'xnames' or 'hsm_group' must be provided".to_string(),
    }),
  ))
}
