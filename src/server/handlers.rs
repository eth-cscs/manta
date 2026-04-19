use std::sync::Arc;

use axum::{
  Json,
  extract::{Query, State},
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
