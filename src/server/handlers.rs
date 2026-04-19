use std::sync::Arc;

use axum::{
  Json,
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::ServerState;
use crate::service;

// ---------------------------------------------------------------------------
// Helper: extract bearer token from Authorization header
// ---------------------------------------------------------------------------

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
  let auth_header = headers
    .get("authorization")
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

  Ok(token.to_string())
}

/// Convert an `anyhow::Error` into a 500 response.
fn internal_error(e: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
  log::error!("Request failed: {:#}", e);
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ErrorResponse {
      error: format!("{:#}", e),
    }),
  )
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

pub async fn get_sessions(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<SessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_configurations(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<ConfigurationQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_nodes(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<NodesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_groups(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<GroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_images(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<ImageQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

  // Convert tuples to named struct for clean JSON
  let entries: Vec<ImageEntry> = images
    .into_iter()
    .map(|(img, config_name, image_id, linked)| ImageEntry {
      image: serde_json::to_value(img).unwrap_or_default(),
      configuration_name: config_name,
      image_id,
      is_linked: linked,
    })
    .collect();

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

pub async fn get_templates(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<TemplateQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_boot_parameters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<BootParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<KernelParametersQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_redfish_endpoints(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<RedfishEndpointsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_clusters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<ClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn get_hardware_clusters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<HardwareClusterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let params = service::hardware::GetHardwareClusterParams {
    hsm_group_name: q.hsm_group,
    settings_hsm_group_name: None,
  };

  let result = service::hardware::get_hardware_cluster(&infra, &token, &params)
    .await
    .map_err(internal_error)?;

  // Serialize the result into a clean JSON structure
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

pub async fn get_hardware_nodes(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<HardwareNodeQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn delete_node(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn add_node(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<AddNodeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn delete_group(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(label): Path<String>,
  Query(q): Query<DeleteGroupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::group::delete_group(&infra, &token, &label, q.force)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/groups
// ---------------------------------------------------------------------------

pub async fn create_group(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(group): Json<::manta_backend_dispatcher::types::Group>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::group::create_group(&infra, &token, group)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::CREATED)
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

pub async fn add_nodes_to_group(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(name): Path<String>,
  Json(body): Json<AddNodesToGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn delete_boot_parameters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<DeleteBootParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::boot_parameters::delete_boot_parameters(&infra, &token, body.hosts)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/boot-parameters
// ---------------------------------------------------------------------------

pub async fn add_boot_parameters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(boot_params): Json<::manta_backend_dispatcher::types::bss::BootParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::boot_parameters::add_boot_parameters(&infra, &token, &boot_params)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::CREATED)
}

// ---------------------------------------------------------------------------
// PUT /api/v1/boot-parameters
// ---------------------------------------------------------------------------

pub async fn update_boot_parameters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(params): Json<service::boot_parameters::UpdateBootParametersParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::boot_parameters::update_boot_parameters(&infra, &token, params)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/redfish-endpoints/{id}
// ---------------------------------------------------------------------------

pub async fn delete_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::redfish_endpoints::delete_redfish_endpoint(&infra, &token, &id)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

pub async fn add_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(params): Json<service::redfish_endpoints::UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::redfish_endpoints::add_redfish_endpoint(&infra, &token, params)
    .await
    .map_err(internal_error)?;

  Ok(StatusCode::CREATED)
}

// ---------------------------------------------------------------------------
// PUT /api/v1/redfish-endpoints
// ---------------------------------------------------------------------------

pub async fn update_redfish_endpoint(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(params): Json<service::redfish_endpoints::UpdateRedfishEndpointParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn delete_session(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(name): Path<String>,
  Query(q): Query<DeleteSessionQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let deletion_ctx =
    service::session::prepare_session_deletion(&infra, &token, &name, None)
      .await
      .map_err(internal_error)?;

  if q.dry_run {
    return Ok((StatusCode::OK, Json(serde_json::to_value(&deletion_ctx).unwrap_or_default())));
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

pub async fn delete_images_handler(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<DeleteImagesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let id_strings: Vec<String> = q.ids.split(',').map(|s| s.trim().to_string()).collect();
  let id_refs: Vec<&str> = id_strings.iter().map(|s| s.as_str()).collect();

  // Validate first
  service::image::validate_image_deletion(&infra, &token, &id_refs, None)
    .await
    .map_err(internal_error)?;

  if q.dry_run {
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

pub async fn delete_configurations_handler(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Query(q): Query<DeleteConfigurationsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let since = q
    .since
    .as_deref()
    .map(|s| {
      chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| {
          (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
              error: format!("Invalid 'since' datetime: {}", e),
            }),
          )
        })
    })
    .transpose()?;

  let until = q
    .until
    .as_deref()
    .map(|s| {
      chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| {
          (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
              error: format!("Invalid 'until' datetime: {}", e),
            }),
          )
        })
    })
    .transpose()?;

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
    return Ok((StatusCode::OK, Json(serde_json::to_value(&candidates).unwrap_or_default())));
  }

  service::configuration::delete_configurations_and_derivatives(&infra, &token, &candidates)
    .await
    .map_err(internal_error)?;

  Ok((StatusCode::OK, Json(serde_json::json!({
    "deleted_configurations": candidates.configuration_names,
    "deleted_images": candidates.image_ids,
  }))))
}
