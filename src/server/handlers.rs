use std::sync::Arc;
use std::convert::Infallible;

use axum::{
  Json,
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
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
    hsm::group::GroupTrait,
    pcs::PCSTrait,
  },
  types::{K8sAuth, K8sDetails},
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

pub async fn create_session(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let vault_base_url = state.vault_base_url.as_deref().ok_or_else(|| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        error: "Vault URL not configured — cannot fetch gitea token".to_string(),
      }),
    )
  })?;

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

pub async fn apply_boot_config(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<ApplyBootConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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
    return Ok((StatusCode::OK, Json(serde_json::to_value(&changeset).unwrap_or_default())));
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

#[derive(Deserialize)]
pub struct ApplyKernelParametersRequest {
  pub xnames: Vec<String>,
  /// One of: "add", "apply", "delete"
  pub operation: String,
  pub params: String,
  /// Only relevant for "add" operation
  #[serde(default)]
  pub overwrite: bool,
  /// Whether to project SBPS images (default true)
  #[serde(default = "default_true")]
  pub project_sbps: bool,
  #[serde(default)]
  pub dry_run: bool,
}

fn default_true() -> bool {
  true
}

pub async fn apply_kernel_parameters(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<ApplyKernelParametersRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let operation = match body.operation.as_str() {
    "add" => service::kernel_parameters::KernelParamOperation::Add {
      params: &body.params,
      overwrite: body.overwrite,
    },
    "apply" => service::kernel_parameters::KernelParamOperation::Apply {
      params: &body.params,
    },
    "delete" => service::kernel_parameters::KernelParamOperation::Delete {
      params: &body.params,
    },
    other => {
      return Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!("Invalid operation '{}'. Must be one of: add, apply, delete", other),
        }),
      ));
    }
  };

  let changeset =
    service::kernel_parameters::prepare_kernel_params_changes(&infra, &token, &body.xnames, &operation)
      .await
      .map_err(internal_error)?;

  if body.dry_run {
    return Ok((StatusCode::OK, Json(serde_json::to_value(&changeset).unwrap_or_default())));
  }

  // Build images_to_project from SBPS candidates
  let mut images_to_project = std::collections::HashMap::new();
  if body.project_sbps {
    for (image_id, mut image) in changeset.sbps_candidates.clone() {
      image.set_boot_image_iscsi_ready();
      images_to_project.insert(image_id, image);
    }
  }

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

pub async fn migrate_nodes(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<MigrateNodesRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

pub async fn migrate_backup(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<MigrateBackupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  service::migrate::migrate_backup(
    &infra,
    &token,
    body.bos.as_deref(),
    body.destination.as_deref(),
  )
  .await
  .map_err(internal_error)?;

  Ok(Json(serde_json::json!({ "status": "backup completed" })))
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

pub async fn migrate_restore(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<MigrateRestoreRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

  Ok(Json(serde_json::json!({ "status": "restore completed" })))
}

// ---------------------------------------------------------------------------
// POST /api/v1/ephemeral-env — Create ephemeral CFS environment
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateEphemeralEnvRequest {
  pub image_id: String,
}

pub async fn create_ephemeral_env(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<CreateEphemeralEnvRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;

  crate::cli::commands::apply_ephemeral_env::exec(
    &state.shasta_base_url,
    &state.shasta_root_cert,
    &token,
    &body.image_id,
  )
  .await
  .map_err(internal_error)?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "status": "ephemeral environment created" }))))
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

pub async fn delete_group_members(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(name): Path<String>,
  Json(body): Json<DeleteGroupMembersRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
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

#[derive(Deserialize)]
pub struct PowerRequest {
  pub action: String,
  pub targets: Vec<String>,
  pub target_type: String,
  #[serde(default)]
  pub force: bool,
}

pub async fn post_power(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<PowerRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let xnames: Vec<String> = if body.target_type == "cluster" {
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
  } else {
    body.targets.clone()
  };

  if xnames.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: "No nodes to operate on".into(),
      }),
    ));
  }

  let result = match body.action.as_str() {
    "on" => infra.backend.power_on_sync(&token, &xnames).await,
    "off" => {
      infra
        .backend
        .power_off_sync(&token, &xnames, body.force)
        .await
    }
    "reset" => {
      infra
        .backend
        .power_reset_sync(&token, &xnames, body.force)
        .await
    }
    other => {
      return Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
          error: format!(
            "Unknown action '{}': must be 'on', 'off', or 'reset'",
            other
          ),
        }),
      ))
    }
  }
  .map_err(|e| internal_error(anyhow::anyhow!("{:#}", e)))?;

  Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/v1/templates/{name}/sessions — Create BOS session from template
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PostTemplateSessionRequest {
  pub operation: String,
  pub limit: String,
  pub session_name: Option<String>,
  #[serde(default)]
  pub include_disabled: bool,
  #[serde(default)]
  pub dry_run: bool,
}

pub async fn post_template_session(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(name): Path<String>,
  Json(body): Json<PostTemplateSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let params = service::template::ApplyTemplateParams {
    bos_session_name: body.session_name,
    bos_sessiontemplate_name: name,
    bos_session_operation: body.operation,
    limit: body.limit,
    include_disabled: body.include_disabled,
  };

  let (bos_session, _) =
    service::template::validate_and_prepare_template_session(
      &infra, &token, &params,
    )
    .await
    .map_err(internal_error)?;

  if body.dry_run {
    return Ok((
      StatusCode::OK,
      Json(
        serde_json::to_value(&bos_session)
          .unwrap_or(serde_json::json!({})),
      ),
    ));
  }

  let created =
    service::template::create_bos_session(&infra, &token, bos_session)
      .await
      .map_err(internal_error)?;

  Ok((
    StatusCode::CREATED,
    Json(serde_json::to_value(&created).unwrap_or(serde_json::json!({}))),
  ))
}

// ---------------------------------------------------------------------------
// GET /api/v1/sessions/{name}/logs — Stream CFS session logs via SSE
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SessionLogsQuery {
  #[serde(default)]
  pub timestamps: bool,
}

pub async fn get_session_logs(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Path(name): Path<String>,
  Query(q): Query<SessionLogsQuery>,
) -> Result<
  Sse<impl futures::Stream<Item = Result<Event, Infallible>>>,
  (StatusCode, Json<ErrorResponse>),
> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let k8s_api_url = infra.k8s_api_url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "k8s_api_url not configured on this server".into(),
      }),
    )
  })?;

  let vault_base_url = infra.vault_base_url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "vault_base_url not configured on this server".into(),
      }),
    )
  })?;

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

pub async fn post_sat_file(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(body): Json<PostSatFileRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let token = extract_bearer_token(&headers)?;
  let infra = state.infra_context();

  let vault_base_url = infra.vault_base_url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "vault_base_url not configured on this server".into(),
      }),
    )
  })?;

  let k8s_api_url = infra.k8s_api_url.ok_or_else(|| {
    (
      StatusCode::NOT_IMPLEMENTED,
      Json(ErrorResponse {
        error: "k8s_api_url not configured on this server".into(),
      }),
    )
  })?;

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

  let sat_template_string =
    serde_yaml::to_string(&sat_template_yaml).map_err(|e| {
      internal_error(anyhow::anyhow!(
        "Failed to serialize SAT template: {}",
        e
      ))
    })?;

  let mut sat_file: crate::cli::commands::apply_sat_file::utils::SatFile =
    serde_yaml::from_str(&sat_template_string).map_err(|e| {
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

  Ok(Json(serde_json::json!({ "status": "SAT file applied successfully" })))
}
