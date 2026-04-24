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
  interfaces::{
    apply_sat_file::SatTrait,
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

/// Map an error message to the most appropriate HTTP status code.
///
/// Inspects well-known phrases from the service layer to avoid returning
/// 500 for conditions that are actually 404 or 409.
pub(crate) fn classify_status(msg: &str) -> StatusCode {
  let lower = msg.to_lowercase();
  if lower.contains("not found") || lower.contains("does not exist") {
    StatusCode::NOT_FOUND
  } else if lower.contains("already exists") {
    StatusCode::CONFLICT
  } else {
    StatusCode::INTERNAL_SERVER_ERROR
  }
}

/// Convert an `anyhow::Error` into the best-fitting HTTP error response.
///
/// Returns 404, 409, or 500 depending on `classify_status`. Only 500s
/// are logged as errors; lower-severity failures are logged at debug.
fn internal_error(e: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
  let msg = format!("{:#}", e);
  let status = classify_status(&msg);
  if status == StatusCode::INTERNAL_SERVER_ERROR {
    tracing::error!("Internal error: {}", msg);
  } else {
    tracing::debug!("Service error {}: {}", status, msg);
  }
  (status, Json(ErrorResponse { error: msg }))
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

#[derive(Serialize)]
pub struct ErrorResponse {
  pub error: String,
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn health() -> impl IntoResponse {
  Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/sessions
// ---------------------------------------------------------------------------

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

#[tracing::instrument(skip_all)]
pub async fn get_sessions(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::session::GetSessionParams {
    hsm_group: q.hsm_group,
    xnames: q
      .xnames
      .map(|x| x.split(',').map(String::from).collect())
      .unwrap_or_default(),
    min_age: q.min_age,
    max_age: q.max_age,
    session_type: q.session_type,
    status: q.status,
    name: q.name,
    limit: q.limit,
  };

  let sessions = service::session::get_sessions(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(sessions))
}

// ---------------------------------------------------------------------------
// GET /api/v1/configurations
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ConfigurationQuery {
  pub name: Option<String>,
  pub pattern: Option<String>,
  pub hsm_group: Option<String>,
  pub limit: Option<u8>,
}

#[tracing::instrument(skip_all)]
pub async fn get_configurations(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<ConfigurationQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

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
      .map_err(internal_error)?;

  Ok(Json(configs))
}

// ---------------------------------------------------------------------------
// GET /api/v1/nodes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct NodesQuery {
  pub xname: String,
  pub include_siblings: Option<bool>,
  pub status: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_nodes(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<NodesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::node::GetNodesParams {
    xname: q.xname,
    include_siblings: q.include_siblings.unwrap_or(false),
    status_filter: q.status,
  };

  let nodes = service::node::get_nodes(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(nodes))
}

// ---------------------------------------------------------------------------
// GET /api/v1/groups
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct GroupQuery {
  pub name: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_groups(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<GroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::group::GetGroupParams {
    group_name: q.name,
    settings_hsm_group_name: None,
  };

  let groups = service::group::get_groups(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(groups))
}

// ---------------------------------------------------------------------------
// GET /api/v1/images
// ---------------------------------------------------------------------------

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

#[tracing::instrument(skip_all)]
pub async fn get_images(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<ImageQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::image::GetImagesParams {
    id: q.id,
    hsm_group: q.hsm_group,
    settings_hsm_group_name: None,
    limit: q.limit,
  };

  let images = service::image::get_images(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

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

#[derive(Deserialize)]
pub struct TemplateQuery {
  pub name: Option<String>,
  pub hsm_group: Option<String>,
  pub limit: Option<u8>,
}

#[tracing::instrument(skip_all)]
pub async fn get_templates(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<TemplateQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::template::GetTemplateParams {
    name: q.name,
    hsm_group: q.hsm_group,
    settings_hsm_group_name: None,
    limit: q.limit,
  };

  let templates = service::template::get_templates(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(templates))
}

// ---------------------------------------------------------------------------
// GET /api/v1/boot-parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct BootParametersQuery {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<BootParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::boot_parameters::GetBootParametersParams {
    hsm_group: q.hsm_group,
    nodes: q.nodes,
    settings_hsm_group_name: None,
  };

  let boot_params =
    service::boot_parameters::get_boot_parameters(&infra, &token, &params)
      .await
      .map_err(internal_error)?;

  Ok(Json(boot_params))
}

// ---------------------------------------------------------------------------
// GET /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct KernelParametersQuery {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<KernelParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::kernel_parameters::GetKernelParametersParams {
    hsm_group: q.hsm_group,
    nodes: q.nodes,
    settings_hsm_group_name: None,
  };

  let kernel_params =
    service::kernel_parameters::get_kernel_parameters(&infra, &token, &params)
      .await
      .map_err(internal_error)?;

  Ok(Json(kernel_params))
}

// ---------------------------------------------------------------------------
// GET /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RedfishEndpointsQuery {
  pub id: Option<String>,
  pub fqdn: Option<String>,
  pub uuid: Option<String>,
  pub macaddr: Option<String>,
  pub ipaddress: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_redfish_endpoints(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<RedfishEndpointsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

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
      .map_err(internal_error)?;

  Ok(Json(endpoints))
}

// ---------------------------------------------------------------------------
// GET /api/v1/clusters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ClusterQuery {
  pub hsm_group: Option<String>,
  pub status: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_clusters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<ClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::cluster::GetClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
    status_filter: q.status,
  };

  let nodes = service::cluster::get_cluster_nodes(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(nodes))
}

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-clusters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct HardwareClusterQuery {
  pub hsm_group: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_hardware_clusters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<HardwareClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::hardware::GetHardwareClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
  };

  let result = service::hardware::get_hardware_cluster(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(serde_json::json!({
    "hsm_group_name": result.hsm_group_name,
    "node_summaries": result.node_summaries,
  })))
}

// ---------------------------------------------------------------------------
// GET /api/v1/hardware-nodes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct HardwareNodeQuery {
  pub xnames: String,
  pub type_artifact: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_hardware_nodes(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<HardwareNodeQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let infra = state.infra_context();

  let params = service::hardware::GetHardwareNodeParams {
    xnames: q.xnames,
    type_artifact: q.type_artifact,
  };

  let result = service::hardware::get_hardware_node(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  Ok(Json(serde_json::json!({
    "node_summary": result.node_summary,
  })))
}

// ===========================================================================
// WRITE ENDPOINTS
// ===========================================================================

// ---------------------------------------------------------------------------
// DELETE /api/v1/nodes/{id}
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn delete_node(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_node id={}", id);
  let infra = state.infra_context();

  service::node::delete_node(&infra, &token, &id)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/nodes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddNodeRequest {
  pub id: String,
  pub group: String,
  #[serde(default)]
  pub enabled: bool,
  pub arch: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn add_node(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<AddNodeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_node id={} group={}", body.id, body.group);
  let infra = state.infra_context();

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
  .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": body.id }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{label}
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteGroupQuery {
  #[serde(default)]
  pub force: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_group(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(label): Path<String>,
  Query(q): Query<DeleteGroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_group label={} force={}", label, q.force);
  let infra = state.infra_context();

  service::group::delete_group(&infra, &token, &label, q.force)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn create_group(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(group): Json<::manta_backend_dispatcher::types::Group>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("create_group");
  let infra = state.infra_context();

  service::group::create_group(&infra, &token, group)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups/{name}/members
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddNodesToGroupRequest {
  pub hosts_expression: String,
}

#[derive(Serialize)]
pub struct AddNodesToGroupResponse {
  pub added: Vec<String>,
  pub removed: Vec<String>,
}

#[tracing::instrument(skip_all)]
pub async fn add_nodes_to_group(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(name): Path<String>,
  Json(body): Json<AddNodesToGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "add_nodes_to_group group={} hosts={}",
    name,
    body.hosts_expression
  );
  let infra = state.infra_context();

  let (added, removed) =
    service::group::add_nodes_to_group(&infra, &token, &name, &body.hosts_expression)
      .await
      .map_err(internal_error)?;

  Ok(Json(AddNodesToGroupResponse { added, removed }))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/boot-parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteBootParametersRequest {
  pub hosts: Vec<String>,
}

#[tracing::instrument(skip_all)]
pub async fn delete_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
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
  let infra = state.infra_context();

  service::boot_parameters::delete_boot_parameters(&infra, &token, body.hosts)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/boot-parameters
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn add_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(boot_params): Json<::manta_backend_dispatcher::types::bss::BootParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_boot_parameters");
  let infra = state.infra_context();

  service::boot_parameters::add_boot_parameters(&infra, &token, &boot_params)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/boot-parameters
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn update_boot_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(params): Json<service::boot_parameters::UpdateBootParametersParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("update_boot_parameters");
  let infra = state.infra_context();

  service::boot_parameters::update_boot_parameters(&infra, &token, params)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/redfish-endpoints/{id}
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn delete_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_redfish_endpoint id={}", id);
  let infra = state.infra_context();

  service::redfish_endpoints::delete_redfish_endpoint(&infra, &token, &id)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn add_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(params): Json<service::redfish_endpoints::UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_redfish_endpoint");
  let infra = state.infra_context();

  service::redfish_endpoints::add_redfish_endpoint(&infra, &token, params)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all)]
pub async fn update_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(params): Json<service::redfish_endpoints::UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("update_redfish_endpoint");
  let infra = state.infra_context();

  service::redfish_endpoints::update_redfish_endpoint(&infra, &token, params)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/sessions/{name} — with ?dry_run=true support
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteSessionQuery {
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(name): Path<String>,
  Query(q): Query<DeleteSessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_session name={} dry_run={}", name, q.dry_run);
  let infra = state.infra_context();

  let deletion_ctx =
    service::session::prepare_session_deletion(&infra, &token, &name, None)
      .await
      .map_err(internal_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&deletion_ctx)?)));
  }

  service::session::execute_session_deletion(&infra, &token, &deletion_ctx, false)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": name }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/images — with ?ids=id1,id2&dry_run=true
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteImagesQuery {
  pub ids: String,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_images(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<DeleteImagesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_images ids={} dry_run={}", q.ids, q.dry_run);
  let infra = state.infra_context();

  let id_strings: Vec<String> = q.ids.split(',').map(|s| s.trim().to_string()).collect();
  let id_refs: Vec<&str> = id_strings.iter().map(|s| s.as_str()).collect();

  if q.dry_run {
    service::image::validate_image_deletion(&infra, &token, &id_refs, None)
      .await
      .map_err(internal_error)?;
    return Ok((StatusCode::OK, Json(serde_json::json!({ "validated_ids": id_strings }))));
  }

  let deleted = service::image::delete_images(&infra, &token, &id_refs, None)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": deleted }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/configurations — with ?pattern=...&since=...&until=...&dry_run=true
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteConfigurationsQuery {
  pub pattern: Option<String>,
  pub since: Option<String>,
  pub until: Option<String>,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_configurations(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Query(q): Query<DeleteConfigurationsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_configurations dry_run={}", q.dry_run);
  let infra = state.infra_context();

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
  .map_err(internal_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&candidates)?)));
  }

  service::configuration::delete_configurations_and_derivatives(&infra, &token, &candidates)
    .await
    .map_err(internal_error)?;

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

#[derive(Deserialize)]
pub struct CreateSessionRequest {
  pub cfs_conf_sess_name: Option<String>,
  pub playbook_yaml_file_name: Option<String>,
  pub hsm_group: Option<String>,
  pub repo_names: Vec<String>,
  pub repo_last_commit_ids: Vec<String>,
  pub ansible_limit: Option<String>,
  pub ansible_verbosity: Option<String>,
  pub ansible_passthrough: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn create_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  validate_repo_list_lengths(&body.repo_names, &body.repo_last_commit_ids)?;
  tracing::info!("create_session repos={:?}", body.repo_names);
  let infra = state.infra_context();

  let vault_base_url = require_vault(state.vault_base_url.as_deref())?;

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(&token, vault_base_url, &state.site_name)
      .await
      .map_err(|e| internal_error(e.into()))?;

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
  .map_err(internal_error)?;

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

#[derive(Deserialize)]
pub struct ApplyBootConfigRequest {
  pub hosts_expression: String,
  pub boot_image_id: Option<String>,
  pub boot_image_configuration: Option<String>,
  pub kernel_parameters: Option<String>,
  pub runtime_configuration: Option<String>,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn apply_boot_config(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<ApplyBootConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "apply_boot_config hosts={} dry_run={}",
    body.hosts_expression,
    body.dry_run
  );
  let infra = state.infra_context();

  let changeset = service::boot_parameters::prepare_boot_config(
    &infra,
    &token,
    &body.hosts_expression,
    body.boot_image_id.as_deref(),
    body.boot_image_configuration.as_deref(),
    body.kernel_parameters.as_deref(),
  )
  .await
  .map_err(internal_error)?;

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
  .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "nodes": changeset.xname_vec,
    "need_restart": changeset.need_restart,
  }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/kernel-parameters/apply — Apply kernel parameter changes
// ---------------------------------------------------------------------------

/// "add" | "apply" | "delete"
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KernelParamOp {
  Add,
  Apply,
  Delete,
}

#[derive(Deserialize)]
pub struct ApplyKernelParametersRequest {
  pub xnames: Vec<String>,
  pub operation: KernelParamOp,
  pub params: String,
  /// Only relevant for the `add` operation.
  #[serde(default)]
  pub overwrite: bool,
  /// Whether to project SBPS images (default true).
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  #[serde(default)]
  pub dry_run: bool,
}

fn default_true() -> bool {
  true
}

#[tracing::instrument(skip_all)]
pub async fn apply_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<ApplyKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  if body.xnames.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "xnames list must not be empty".to_string(),
      }),
    ));
  }
  tracing::info!(
    "apply_kernel_parameters xnames={:?} op={:?} dry_run={}",
    body.xnames,
    body.operation,
    body.dry_run
  );
  let infra = state.infra_context();

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
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &body.xnames, &operation)
      .await
      .map_err(internal_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  let images_to_project =
    service::kernel_parameters::build_images_to_project(&changeset, body.project_sbps);

  service::kernel_parameters::apply_kernel_params_changes(&infra, &token, &changeset, &images_to_project)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "has_changes": changeset.has_changes,
    "xnames_to_reboot": changeset.xnames_to_reboot,
  }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/nodes — Migrate nodes between HSM groups
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct MigrateNodesRequest {
  pub target_hsm_names: Vec<String>,
  pub parent_hsm_names: Vec<String>,
  pub hosts_expression: String,
  #[serde(default)]
  pub dry_run: bool,
  #[serde(default)]
  pub create_hsm_group: bool,
}

#[tracing::instrument(skip_all)]
pub async fn migrate_nodes(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<MigrateNodesRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_nodes dry_run={}", body.dry_run);
  let infra = state.infra_context();

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
  .map_err(internal_error)?;

  Ok(Json(serde_json::json!({
    "xnames": xnames,
    "results": results,
  })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/backup — Backup BOS session templates
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct MigrateBackupRequest {
  pub bos: Option<String>,
  pub destination: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn migrate_backup(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<MigrateBackupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_backup");
  let infra = state.infra_context();

  service::migrate::migrate_backup(
    &infra,
    &token,
    body.bos.as_deref(),
    body.destination.as_deref(),
  )
  .await
  .map_err(internal_error)?;

  Ok(Json(serde_json::json!({ "completed": true })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/migrate/restore — Restore from backup files
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct MigrateRestoreRequest {
  pub bos_file: Option<String>,
  pub cfs_file: Option<String>,
  pub hsm_file: Option<String>,
  pub ims_file: Option<String>,
  pub image_dir: Option<String>,
  #[serde(default)]
  pub overwrite: bool,
}

#[tracing::instrument(skip_all)]
pub async fn migrate_restore(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<MigrateRestoreRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("migrate_restore overwrite={}", body.overwrite);
  let infra = state.infra_context();

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
  .map_err(internal_error)?;

  Ok(Json(serde_json::json!({ "completed": true })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/ephemeral-env — Create ephemeral CFS environment
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateEphemeralEnvRequest {
  pub image_id: String,
}

#[tracing::instrument(skip_all)]
pub async fn create_ephemeral_env(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<CreateEphemeralEnvRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("create_ephemeral_env image_id={}", body.image_id);

  crate::cli::commands::apply_ephemeral_env::exec(
    &state.shasta_base_url,
    &state.shasta_root_cert,
    &token,
    &body.image_id,
  )
  .await
  .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "created": true }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/groups/{name}/members — Remove nodes from HSM group
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteGroupMembersRequest {
  pub xnames: Vec<String>,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_group_members(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(name): Path<String>,
  Json(body): Json<DeleteGroupMembersRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "delete_group_members group={} xnames={:?} dry_run={}",
    name,
    body.xnames,
    body.dry_run
  );
  let infra = state.infra_context();

  if !body.dry_run {
    for xname in &body.xnames {
      infra
        .backend
        .delete_member_from_group(&token, &name, xname)
        .await
        .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;
    }
  }

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/power — Power on/off/reset nodes or cluster
// ---------------------------------------------------------------------------

/// "on" | "off" | "reset"
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerAction {
  On,
  Off,
  Reset,
}

/// "nodes" | "cluster"
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerTargetType {
  Nodes,
  Cluster,
}

#[derive(Deserialize)]
pub struct PowerRequest {
  pub action: PowerAction,
  pub targets: Vec<String>,
  pub target_type: PowerTargetType,
  #[serde(default)]
  pub force: bool,
}

#[tracing::instrument(skip_all)]
pub async fn post_power(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<PowerRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_power action={:?} target_type={:?}",
    body.action,
    body.target_type
  );
  let infra = state.infra_context();

  let xnames: Vec<String> = match body.target_type {
    PowerTargetType::Cluster => {
      let group_name = body.targets.first().ok_or_else(|| {
        (
          StatusCode::BAD_REQUEST,
          Json(ErrorResponse {
            error: "targets must contain at least one cluster name".into(),
          }),
        )
      })?;
      infra
        .backend
        .get_member_vec_from_group_name_vec(&token, &[group_name.clone()])
        .await
        .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?
    }
    PowerTargetType::Nodes => body.targets.clone(),
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
  .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;

  Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/v1/templates/{name}/sessions — Create BOS session from template
// ---------------------------------------------------------------------------

/// "boot" | "reboot" | "shutdown"
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BosOperation {
  Boot,
  Reboot,
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

#[derive(Deserialize)]
pub struct PostTemplateSessionRequest {
  pub operation: BosOperation,
  pub limit: String,
  pub session_name: Option<String>,
  #[serde(default)]
  pub include_disabled: bool,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn post_template_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(name): Path<String>,
  Json(body): Json<PostTemplateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(
    "post_template_session template={} op={:?} dry_run={}",
    name,
    body.operation,
    body.dry_run
  );
  let infra = state.infra_context();

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
      .map_err(internal_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&bos_session)?)));
  }

  let created =
    service::template::create_bos_session(&infra, &token, bos_session)
      .await
      .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serialize_or_500(&created)?)))
}

// ---------------------------------------------------------------------------
// GET /api/v1/sessions/{name}/logs — Stream CFS session logs via SSE
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SessionLogsQuery {
  #[serde(default)]
  pub timestamps: bool,
}

#[tracing::instrument(skip_all)]
pub async fn get_session_logs(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(name): Path<String>,
  Query(q): Query<SessionLogsQuery>,
) -> Result<
  Sse<impl futures::Stream<Item = Result<Event, Infallible>>>,
  (StatusCode, Json<ErrorResponse>),
> {
  let infra = state.infra_context();

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
    .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;

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

#[derive(Deserialize)]
pub struct PostSatFileRequest {
  pub sat_file_content: String,
  pub values: Option<serde_json::Value>,
  pub values_file_content: Option<String>,
  pub ansible_verbosity: Option<u8>,
  pub ansible_passthrough: Option<String>,
  #[serde(default)]
  pub reboot: bool,
  #[serde(default)]
  pub watch_logs: bool,
  #[serde(default)]
  pub timestamps: bool,
  #[serde(default)]
  pub image_only: bool,
  #[serde(default)]
  pub session_template_only: bool,
  #[serde(default)]
  pub overwrite: bool,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn post_sat_file(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<PostSatFileRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_file dry_run={}", body.dry_run);
  let infra = state.infra_context();

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(
      &token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .map_err(internal_error)?;

  let hsm_group_available_vec = infra
    .backend
    .get_group_name_available(&token)
    .await
    .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;

  let values_cli_vec: Vec<String> = body
    .values
    .as_ref()
    .and_then(|v| v.as_object())
    .map(|map| {
      map
        .iter()
        .map(|(k, v)| {
          format!("{}={}", k, v.as_str().unwrap_or(&v.to_string()))
        })
        .collect()
    })
    .unwrap_or_default();

  let sat_template_yaml =
    crate::cli::commands::apply_sat_file::utils::render_jinja2_sat_file_yaml(
      &body.sat_file_content,
      body.values_file_content.as_deref(),
      if values_cli_vec.is_empty() {
        None
      } else {
        Some(&values_cli_vec)
      },
    )
    .map_err(internal_error)?;

  let mut sat_file: crate::cli::commands::apply_sat_file::utils::SatFile =
    serde_yaml::from_value(sat_template_yaml).map_err(|e| {
      internal_error(anyhow::anyhow!("Failed to parse SAT file: {}", e))
    })?;

  sat_file
    .filter(body.image_only, body.session_template_only)
    .map_err(internal_error)?;

  let sat_file_yaml = serde_yaml::to_value(sat_file).map_err(|e| {
    internal_error(anyhow::anyhow!(
      "Failed to convert SAT file to YAML: {}",
      e
    ))
  })?;

  let shasta_k8s_secrets =
    crate::common::vault::http_client::fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      infra.site_name,
      &token,
    )
    .await
    .map_err(internal_error)?;

  infra
    .backend
    .apply_sat_file(
      &token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      vault_base_url,
      infra.site_name,
      k8s_api_url,
      shasta_k8s_secrets,
      sat_file_yaml,
      &hsm_group_available_vec,
      body.ansible_verbosity,
      body.ansible_passthrough.as_deref(),
      infra.gitea_base_url,
      &gitea_token,
      body.reboot,
      body.watch_logs,
      body.timestamps,
      true,
      body.overwrite,
      body.dry_run,
    )
    .await
    .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;

  Ok(Json(serde_json::json!({ "applied": true })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/kernel-parameters/add
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddKernelParametersRequest {
  pub params: String,
  pub xnames: Option<Vec<String>>,
  pub hsm_group: Option<String>,
  #[serde(default)]
  pub overwrite: bool,
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn add_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<AddKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let xnames = resolve_xnames_from_request(
    &state,
    &token,
    body.xnames.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!("add_kernel_parameters xnames={:?} dry_run={}", xnames, body.dry_run);
  let infra = state.infra_context();

  let operation = service::kernel_parameters::KernelParamOperation::Add {
    params: &body.params,
    overwrite: body.overwrite,
  };

  let changeset =
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &xnames, &operation)
      .await
      .map_err(internal_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serialize_or_500(&changeset)?)));
  }

  let images_to_project =
    service::kernel_parameters::build_images_to_project(&changeset, body.project_sbps);

  service::kernel_parameters::apply_kernel_params_changes(&infra, &token, &changeset, &images_to_project)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "has_changes": changeset.has_changes,
    "xnames_to_reboot": changeset.xnames_to_reboot,
  }))))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/kernel-parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeleteKernelParametersRequest {
  pub params: String,
  pub xnames: Option<Vec<String>>,
  pub hsm_group: Option<String>,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<DeleteKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let xnames = resolve_xnames_from_request(
    &state,
    &token,
    body.xnames.as_deref(),
    body.hsm_group.as_deref(),
  )
  .await?;

  tracing::info!("delete_kernel_parameters xnames={:?} dry_run={}", xnames, body.dry_run);
  let infra = state.infra_context();

  let operation = service::kernel_parameters::KernelParamOperation::Delete {
    params: &body.params,
  };

  let changeset =
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &xnames, &operation)
      .await
      .map_err(internal_error)?;

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
  .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "applied": true,
    "has_changes": changeset.has_changes,
    "xnames_to_reboot": changeset.xnames_to_reboot,
  }))))
}

// ---------------------------------------------------------------------------
// POST /api/v1/hardware-clusters/{target}/members
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddHwComponentRequest {
  pub parent_cluster: String,
  pub pattern: String,
  #[serde(default)]
  pub create_hsm_group: bool,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn add_hw_component(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(target): Path<String>,
  Json(body): Json<AddHwComponentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("add_hw_component target={} parent={} dry_run={}", target, body.parent_cluster, body.dry_run);

  let result =
    crate::cli::commands::add_hw_component_cluster::run(
      &state.backend,
      &token,
      &target,
      &body.parent_cluster,
      &body.pattern,
      body.dry_run,
      body.create_hsm_group,
    )
    .await
    .map_err(internal_error)?;

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

#[derive(Deserialize)]
pub struct DeleteHwComponentRequest {
  pub parent_cluster: String,
  pub pattern: String,
  #[serde(default)]
  pub delete_hsm_group: bool,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn delete_hw_component(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(target): Path<String>,
  Json(body): Json<DeleteHwComponentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("delete_hw_component target={} parent={} dry_run={}", target, body.parent_cluster, body.dry_run);

  let result =
    crate::cli::commands::delete_hw_component_cluster::run(
      &state.backend,
      &token,
      &target,
      &body.parent_cluster,
      &body.pattern,
      body.dry_run,
      body.delete_hsm_group,
    )
    .await
    .map_err(internal_error)?;

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

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HwClusterMode {
  #[default]
  Pin,
  Unpin,
}

#[derive(Deserialize)]
pub struct ApplyHwConfigurationRequest {
  pub parent_cluster: String,
  pub pattern: String,
  #[serde(default)]
  pub mode: HwClusterMode,
  #[serde(default = "default_true")]
  pub create_target_hsm_group: bool,
  #[serde(default = "default_true")]
  pub delete_empty_parent_hsm_group: bool,
  #[serde(default)]
  pub dry_run: bool,
}

#[tracing::instrument(skip_all)]
pub async fn apply_hw_configuration(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Path(target): Path<String>,
  Json(body): Json<ApplyHwConfigurationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("apply_hw_configuration target={} parent={} dry_run={}", target, body.parent_cluster, body.dry_run);

  let mode = match body.mode {
    HwClusterMode::Pin => crate::cli::commands::hw_cluster_common::command::HwClusterMode::Pin,
    HwClusterMode::Unpin => crate::cli::commands::hw_cluster_common::command::HwClusterMode::Unpin,
  };

  let result =
    crate::cli::commands::hw_cluster_common::command::exec_with_backend(
      &state.backend,
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
    .map_err(internal_error)?;

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

#[derive(Deserialize)]
pub struct ApplySessionRequest {
  pub repo_names: Vec<String>,
  pub repo_last_commit_ids: Vec<String>,
  pub cfs_conf_sess_name: Option<String>,
  pub playbook_yaml_file_name: Option<String>,
  pub hsm_group: Option<String>,
  pub ansible_limit: Option<String>,
  pub ansible_verbosity: Option<String>,
  pub ansible_passthrough: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn apply_session(
  State(state): State<Arc<ServerState>>,
  BearerToken(token): BearerToken,
  Json(body): Json<ApplySessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  validate_repo_list_lengths(&body.repo_names, &body.repo_last_commit_ids)?;

  tracing::info!("apply_session repos={:?}", body.repo_names);
  let infra = state.infra_context();
  let vault_base_url = require_vault(state.vault_base_url.as_deref())?;

  if let Some(ref ansible_limit) = body.ansible_limit {
    let xnames: Vec<String> = ansible_limit
      .split(',')
      .map(|s| s.trim().to_string())
      .collect();
    crate::common::authorization::validate_target_hsm_members(infra.backend, &token, &xnames)
      .await
      .map_err(internal_error)?;
  }

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(&token, vault_base_url, &state.site_name)
      .await
      .map_err(|e| internal_error(e.into()))?;

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
  .map_err(internal_error)?;

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

#[derive(Deserialize)]
pub struct ConsoleQuery {
  #[serde(default = "default_cols")]
  pub cols: u16,
  #[serde(default = "default_rows")]
  pub rows: u16,
}

fn default_cols() -> u16 { 80 }
fn default_rows() -> u16 { 24 }

#[tracing::instrument(skip_all, fields(xname = %xname))]
pub async fn console_node_ws(
  BearerToken(token): BearerToken,
  State(state): State<Arc<ServerState>>,
  Path(xname): Path<String>,
  Query(q): Query<ConsoleQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let k8s_api_url = require_k8s_url(state.k8s_api_url.as_deref())?;
  let vault_base_url = require_vault(state.vault_base_url.as_deref())?;

  let k8s = K8sDetails {
    api_url: k8s_api_url.to_string(),
    authentication: K8sAuth::Vault {
      base_url: vault_base_url.to_string(),
    },
  };

  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for node {xname}");
    match state.backend
      .attach_to_node_console(&token, &state.site_name, &xname, q.cols, q.rows, &k8s)
      .await
    {
      Ok((console_in, console_out)) => {
        run_console_bridge(socket, console_in, console_out).await;
        tracing::info!("WebSocket console closed for node {xname}");
      }
      Err(e) => {
        tracing::error!("Failed to attach to node console {xname}: {e:#}");
      }
    }
  }))
}

// ---------------------------------------------------------------------------
// WS /api/v1/sessions/{name}/console — Interactive CFS session console
// ---------------------------------------------------------------------------

#[tracing::instrument(skip_all, fields(session = %name))]
pub async fn console_session_ws(
  BearerToken(token): BearerToken,
  State(state): State<Arc<ServerState>>,
  Path(name): Path<String>,
  Query(q): Query<ConsoleQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let k8s_api_url = require_k8s_url(state.k8s_api_url.as_deref())?;
  let vault_base_url = require_vault(state.vault_base_url.as_deref())?;

  let infra = state.infra_context();

  let sessions = infra.backend
    .get_and_filter_sessions(
      &token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      Vec::new(),
      Vec::new(),
      None, None, None, None,
      Some(&name),
      None, None,
    )
    .await
    .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;

  let session = sessions.first().ok_or_else(|| {
    (
      StatusCode::NOT_FOUND,
      Json(ErrorResponse { error: format!("CFS session '{name}' not found") }),
    )
  })?;

  let target_def = session
    .target.as_ref().and_then(|t| t.definition.as_ref())
    .ok_or_else(|| {
      (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "CFS session has no target definition".into() }))
    })?;
  if target_def != "image" {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse { error: format!("CFS session '{name}' is not an image-type session (got '{target_def}')") }),
    ));
  }

  let status = session
    .status.as_ref()
    .and_then(|s| s.session.as_ref())
    .and_then(|s| s.status.as_ref())
    .ok_or_else(|| {
      (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "CFS session has no status".into() }))
    })?;
  if status != "running" {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse { error: format!("CFS session '{name}' is not running (status: '{status}')") }),
    ));
  }

  let k8s = K8sDetails {
    api_url: k8s_api_url.to_string(),
    authentication: K8sAuth::Vault {
      base_url: vault_base_url.to_string(),
    },
  };

  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for session {name}");
    match state.backend
      .attach_to_session_console(&token, &state.site_name, &name, q.cols, q.rows, &k8s)
      .await
    {
      Ok((console_in, console_out)) => {
        run_console_bridge(socket, console_in, console_out).await;
        tracing::info!("WebSocket console closed for session {name}");
      }
      Err(e) => {
        tracing::error!("Failed to attach to session console {name}: {e:#}");
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
async fn run_console_bridge(
  mut socket: WebSocket,
  mut console_in: Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
  console_out: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
) {
  let mut out_stream = tokio_util::io::ReaderStream::new(console_out);

  loop {
    tokio::select! {
      msg = socket.recv() => {
        match msg {
          Some(Ok(Message::Binary(data))) => {
            if console_in.write_all(&data).await.is_err() { break; }
          }
          Some(Ok(Message::Text(text))) => {
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
    }
  }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Resolve target xnames from an explicit list or an HSM group name.
/// Returns 400 if neither is provided.
async fn resolve_xnames_from_request(
  state: &ServerState,
  token: &str,
  xnames: Option<&[String]>,
  hsm_group: Option<&str>,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
  if let Some(xnames) = xnames {
    if !xnames.is_empty() {
      return Ok(xnames.to_vec());
    }
  }
  if let Some(group) = hsm_group {
    return crate::common::node_ops::resolve_target_nodes(
      &state.backend,
      token,
      None,
      Some(group),
      None,
    )
    .await
    .map_err(internal_error);
  }
  Err((
    StatusCode::BAD_REQUEST,
    Json(ErrorResponse {
      error: "At least one of 'xnames' or 'hsm_group' must be provided".to_string(),
    }),
  ))
}

