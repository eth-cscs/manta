//! Smoke tests for the HTTP server layer.
//!
//! These tests build the Axum router with a stub backend (real
//! CSM/OCHAMI struct, dummy URLs) and call it via
//! `tower::ServiceExt::oneshot` without starting a TCP listener
//! or doing TLS negotiation.
//!
//! What is verified:
//!   - Health endpoint is reachable and unauthenticated
//!   - Every other route enforces Bearer-token authentication
//!   - Unknown routes return 404, wrong methods return 405
//!   - Request-body validation fires before backend calls
//!   - Vault/K8s-dependent routes return 501 when not configured

use std::sync::Arc;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt as _;
use tower::ServiceExt as _;

use crate::{
  manta_backend_dispatcher::StaticBackendDispatcher,
  server::{ServerState, routes::build_router},
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a router backed by a stub CSM backend with no real URLs.
/// Backend methods are never called because all non-health tests
/// either fail at auth or at Axum's request extraction layer first.
fn router() -> axum::Router {
  let backend =
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"").unwrap();
  let state = Arc::new(ServerState {
    backend,
    site_name: "test".to_string(),
    shasta_base_url: "http://stub.invalid".to_string(),
    shasta_root_cert: vec![],
    vault_base_url: None,
    gitea_base_url: "http://stub.invalid".to_string(),
    k8s_api_url: None,
  });
  build_router(state)
}

/// Same as `router()` but with vault and k8s URLs set, used to
/// reach the "requires vault/k8s" code paths.
fn router_with_vault() -> axum::Router {
  let backend =
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"").unwrap();
  let state = Arc::new(ServerState {
    backend,
    site_name: "test".to_string(),
    shasta_base_url: "http://stub.invalid".to_string(),
    shasta_root_cert: vec![],
    vault_base_url: Some("http://vault.stub.invalid".to_string()),
    gitea_base_url: "http://stub.invalid".to_string(),
    k8s_api_url: Some("http://k8s.stub.invalid".to_string()),
  });
  build_router(state)
}

async fn body_string(body: Body) -> String {
  let bytes = body.collect().await.unwrap().to_bytes();
  String::from_utf8_lossy(&bytes).into_owned()
}

fn get(uri: &str) -> Request<Body> {
  Request::builder()
    .method(Method::GET)
    .uri(uri)
    .body(Body::empty())
    .unwrap()
}

fn get_auth(uri: &str) -> Request<Body> {
  Request::builder()
    .method(Method::GET)
    .uri(uri)
    .header(header::AUTHORIZATION, "Bearer test-token")
    .body(Body::empty())
    .unwrap()
}

fn delete_auth(uri: &str) -> Request<Body> {
  Request::builder()
    .method(Method::DELETE)
    .uri(uri)
    .header(header::AUTHORIZATION, "Bearer test-token")
    .body(Body::empty())
    .unwrap()
}

fn post_json(uri: &str, body: &str) -> Request<Body> {
  Request::builder()
    .method(Method::POST)
    .uri(uri)
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::AUTHORIZATION, "Bearer test-token")
    .body(Body::from(body.to_string()))
    .unwrap()
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_returns_200_without_auth() {
  let resp = router().oneshot(get("/api/v1/health")).await.unwrap();
  assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_body_contains_status_ok() {
  let resp = router().oneshot(get("/api/v1/health")).await.unwrap();
  let body = body_string(resp.into_body()).await;
  assert!(body.contains("\"ok\""), "body was: {}", body);
}

// ---------------------------------------------------------------------------
// Unknown routes return 404
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unknown_route_returns_404() {
  let resp = router()
    .oneshot(get("/api/v1/does-not-exist"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn wrong_api_version_returns_404() {
  let resp = router()
    .oneshot(get("/api/v2/sessions"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Wrong HTTP method returns 405
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_on_post_only_route_returns_405() {
  let resp = router()
    .oneshot(get("/api/v1/power"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn post_on_get_only_route_returns_405() {
  // POST /clusters is not a registered route; only GET is allowed.
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/clusters")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn delete_health_returns_405() {
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// ---------------------------------------------------------------------------
// Authentication enforcement — GET endpoints
//
// GET endpoints have no body extractors, so the handler always runs
// and the first thing it does is check the Authorization header.
// ---------------------------------------------------------------------------

async fn assert_401_without_auth(uri: &str) {
  let resp = router().oneshot(get(uri)).await.unwrap();
  assert_eq!(
    resp.status(),
    StatusCode::UNAUTHORIZED,
    "expected 401 for GET {}",
    uri
  );
}

#[tokio::test]
async fn get_sessions_requires_auth() {
  assert_401_without_auth("/api/v1/sessions").await;
}

#[tokio::test]
async fn get_configurations_requires_auth() {
  assert_401_without_auth("/api/v1/configurations").await;
}

#[tokio::test]
async fn get_groups_requires_auth() {
  assert_401_without_auth("/api/v1/groups").await;
}

#[tokio::test]
async fn get_images_requires_auth() {
  assert_401_without_auth("/api/v1/images").await;
}

#[tokio::test]
async fn get_templates_requires_auth() {
  assert_401_without_auth("/api/v1/templates").await;
}

#[tokio::test]
async fn get_boot_parameters_requires_auth() {
  assert_401_without_auth("/api/v1/boot-parameters").await;
}

#[tokio::test]
async fn get_kernel_parameters_requires_auth() {
  assert_401_without_auth("/api/v1/kernel-parameters").await;
}

#[tokio::test]
async fn get_redfish_endpoints_requires_auth() {
  assert_401_without_auth("/api/v1/redfish-endpoints").await;
}

#[tokio::test]
async fn get_clusters_requires_auth() {
  assert_401_without_auth("/api/v1/clusters").await;
}

#[tokio::test]
async fn get_hardware_clusters_requires_auth() {
  assert_401_without_auth("/api/v1/hardware-clusters").await;
}

// ---------------------------------------------------------------------------
// Authentication enforcement — DELETE endpoints without body
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_node_requires_auth() {
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/nodes/x3000c0s1b0n0")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_group_requires_auth() {
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/groups/my-group")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_session_requires_auth() {
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/sessions/my-session")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_redfish_endpoint_requires_auth() {
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/redfish-endpoints/x3000c0s1b0")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Request-body validation
//
// These requests include a valid Bearer token. The Axum JSON
// extractor fires before the handler body, so missing required
// fields return 422 Unprocessable Entity before the backend is
// touched.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_nodes_missing_required_fields_returns_422() {
  let resp = router()
    .oneshot(post_json("/api/v1/nodes", "{}"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_ephemeral_env_missing_image_id_returns_422() {
  let resp = router()
    .oneshot(post_json("/api/v1/ephemeral-env", "{}"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_power_unknown_action_returns_400() {
  let resp = router()
    .oneshot(post_json(
      "/api/v1/power",
      r#"{"action":"fly","targets":["x3000c0s1b0n0"],"target_type":"nodes"}"#,
    ))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_power_missing_action_returns_422() {
  let resp = router()
    .oneshot(post_json(
      "/api/v1/power",
      r#"{"targets":["x3000c0s1b0n0"],"target_type":"nodes"}"#,
    ))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_template_session_missing_required_fields_returns_422() {
  let resp = router()
    .oneshot(post_json(
      "/api/v1/templates/my-template/sessions",
      r#"{"dry_run":false}"#,
    ))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_sat_file_missing_content_returns_422() {
  let resp = router()
    .oneshot(post_json("/api/v1/sat-file", "{}"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ---------------------------------------------------------------------------
// Missing required query parameters
//
// GET /nodes and GET /hardware-nodes have a required `xname`/`xnames`
// query parameter. Axum's Query extractor rejects the request with
// 400 Bad Request before auth is checked.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_nodes_without_xname_returns_400() {
  let resp = router()
    .oneshot(get("/api/v1/nodes"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_hardware_nodes_without_xnames_returns_400() {
  let resp = router()
    .oneshot(get("/api/v1/hardware-nodes"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Optional-config endpoints return 501 when vault/k8s not configured
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_session_logs_without_k8s_config_returns_501() {
  let resp = router()
    .oneshot(get_auth("/api/v1/sessions/my-session/logs"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn post_sat_file_without_vault_config_returns_501() {
  let resp = router()
    .oneshot(post_json(
      "/api/v1/sat-file",
      r#"{"sat_file_content":"schema: 1.0\n"}"#,
    ))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}

// When vault IS configured but k8s is not, sat-file still returns 501.
#[tokio::test]
async fn post_sat_file_without_k8s_config_returns_501() {
  let backend =
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"").unwrap();
  let state = Arc::new(ServerState {
    backend,
    site_name: "test".to_string(),
    shasta_base_url: "http://stub.invalid".to_string(),
    shasta_root_cert: vec![],
    vault_base_url: Some("http://vault.stub.invalid".to_string()),
    gitea_base_url: "http://stub.invalid".to_string(),
    k8s_api_url: None, // k8s not set
  });
  let resp = build_router(state)
    .oneshot(post_json(
      "/api/v1/sat-file",
      r#"{"sat_file_content":"schema: 1.0\n"}"#,
    ))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}

// ---------------------------------------------------------------------------
// Route existence — authenticated requests must not return 404 or 405
// ---------------------------------------------------------------------------

async fn assert_route_exists(method: Method, uri: &str) {
  let req = Request::builder()
    .method(method)
    .uri(uri)
    .header(header::AUTHORIZATION, "Bearer test-token")
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::empty())
    .unwrap();
  let resp = router().oneshot(req).await.unwrap();
  assert_ne!(
    resp.status(),
    StatusCode::NOT_FOUND,
    "route not found: {}",
    uri
  );
  assert_ne!(
    resp.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "method not allowed: {}",
    uri
  );
}

#[tokio::test]
async fn all_get_routes_are_registered() {
  for uri in &[
    "/api/v1/sessions",
    "/api/v1/configurations",
    "/api/v1/groups",
    "/api/v1/images",
    "/api/v1/templates",
    "/api/v1/boot-parameters",
    "/api/v1/kernel-parameters",
    "/api/v1/redfish-endpoints",
    "/api/v1/clusters",
    "/api/v1/hardware-clusters",
    "/api/v1/health",
  ] {
    assert_route_exists(Method::GET, uri).await;
  }
}

#[tokio::test]
async fn all_post_routes_are_registered() {
  for uri in &[
    "/api/v1/sessions",
    "/api/v1/nodes",
    "/api/v1/groups",
    "/api/v1/groups/test/members",
    "/api/v1/boot-parameters",
    "/api/v1/redfish-endpoints",
    "/api/v1/boot-config",
    "/api/v1/kernel-parameters/apply",
    "/api/v1/migrate/nodes",
    "/api/v1/migrate/backup",
    "/api/v1/migrate/restore",
    "/api/v1/ephemeral-env",
    "/api/v1/power",
    "/api/v1/templates/my-template/sessions",
    "/api/v1/sat-file",
  ] {
    assert_route_exists(Method::POST, uri).await;
  }
}

#[tokio::test]
async fn all_delete_routes_are_registered() {
  for uri in &[
    "/api/v1/nodes/x3000c0s1b0n0",
    "/api/v1/groups/my-group",
    "/api/v1/groups/my-group/members",
    "/api/v1/boot-parameters",
    "/api/v1/redfish-endpoints/x3000c0s1b0",
    "/api/v1/sessions/my-session",
    "/api/v1/images",
    "/api/v1/configurations",
  ] {
    assert_route_exists(Method::DELETE, uri).await;
  }
}
