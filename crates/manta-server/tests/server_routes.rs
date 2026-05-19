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

use manta_server::manta_backend_dispatcher::StaticBackendDispatcher;
use manta_server::server::{ServerState, SiteBackend, routes::build_router};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a router backed by a stub CSM backend with no real URLs.
/// Backend methods are never called because all non-health tests
/// either fail at auth or at Axum's request extraction layer first.
fn router() -> axum::Router {
  let backend =
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"", None)
      .unwrap();
  let mut sites = std::collections::HashMap::new();
  sites.insert(
    "test".to_string(),
    SiteBackend {
      backend,
      shasta_base_url: "http://stub.invalid".to_string(),
      shasta_root_cert: vec![],
      socks5_proxy: None,
      vault_base_url: None,
      gitea_base_url: "http://stub.invalid".to_string(),
      k8s_api_url: None,
    },
  );
  let state = Arc::new(ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(1800),
    auditor: None,
    auth_rate_limit_per_minute: None,
  });
  build_router(state)
}

/// Same as `router()` but with vault and k8s URLs set, used to
/// reach the "requires vault/k8s" code paths.
fn router_with_vault() -> axum::Router {
  let backend =
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"", None)
      .unwrap();
  let mut sites = std::collections::HashMap::new();
  sites.insert(
    "test".to_string(),
    SiteBackend {
      backend,
      shasta_base_url: "http://stub.invalid".to_string(),
      shasta_root_cert: vec![],
      socks5_proxy: None,
      vault_base_url: Some("http://vault.stub.invalid".to_string()),
      gitea_base_url: "http://stub.invalid".to_string(),
      k8s_api_url: Some("http://k8s.stub.invalid".to_string()),
    },
  );
  let state = Arc::new(ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(1800),
    auditor: None,
    auth_rate_limit_per_minute: None,
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
    .header("X-Manta-Site", "test")
    .body(Body::empty())
    .unwrap()
}

fn post_json(uri: &str, body: &str) -> Request<Body> {
  Request::builder()
    .method(Method::POST)
    .uri(uri)
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::AUTHORIZATION, "Bearer test-token")
    .header("X-Manta-Site", "test")
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
  let resp = router().oneshot(get("/api/v2/sessions")).await.unwrap();
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Wrong HTTP method returns 405
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_on_post_only_route_returns_405() {
  let resp = router().oneshot(get("/api/v1/power")).await.unwrap();
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
// Authentication enforcement — table-driven coverage for every privileged
// endpoint. The handler always runs as far as the bearer-token check; we
// just iterate over the route list and assert 401.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_routes_reject_missing_bearer_token() {
  let routes = [
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
  ];
  for uri in routes {
    let resp = router().oneshot(get(uri)).await.unwrap();
    assert_eq!(
      resp.status(),
      StatusCode::UNAUTHORIZED,
      "expected 401 for GET {}",
      uri
    );
  }
}

#[tokio::test]
async fn delete_routes_without_body_reject_missing_bearer_token() {
  let routes = [
    "/api/v1/nodes/x3000c0s1b0n0",
    "/api/v1/groups/my-group",
    "/api/v1/sessions/my-session",
    "/api/v1/redfish-endpoints/x3000c0s1b0",
  ];
  for uri in routes {
    let resp = router()
      .oneshot(
        Request::builder()
          .method(Method::DELETE)
          .uri(uri)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(
      resp.status(),
      StatusCode::UNAUTHORIZED,
      "expected 401 for DELETE {}",
      uri
    );
  }
}

#[tokio::test]
async fn body_routes_reject_missing_bearer_token() {
  // POST/DELETE routes that take a JSON body. The Authorization
  // header check must still trip BEFORE the body deserializer.
  let cases: &[(Method, &str, &str)] = &[
    (Method::POST, "/api/v1/kernel-parameters/add", r#"{"params":"quiet"}"#),
    (Method::DELETE, "/api/v1/kernel-parameters", r#"{"params":"quiet"}"#),
    (
      Method::POST,
      "/api/v1/hardware-clusters/my-cluster/members",
      r#"{"parent_cluster":"p","pattern":"a100:2"}"#,
    ),
    (
      Method::DELETE,
      "/api/v1/hardware-clusters/my-cluster/members",
      r#"{"parent_cluster":"p","pattern":"a100:2"}"#,
    ),
    (
      Method::POST,
      "/api/v1/hardware-clusters/my-cluster/configuration",
      r#"{"parent_cluster":"p","pattern":"a100:2"}"#,
    ),
  ];
  for (method, uri, body) in cases {
    let resp = router()
      .oneshot(
        Request::builder()
          .method(method.clone())
          .uri(*uri)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(body.to_string()))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(
      resp.status(),
      StatusCode::UNAUTHORIZED,
      "expected 401 for {} {}",
      method,
      uri
    );
  }
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
async fn post_routes_reject_invalid_bodies() {
  // Authenticated requests whose JSON body fails to deserialise into
  // the handler's expected shape — serde rejects them as 422 before
  // the handler runs. Comments inline justify each case beyond the
  // generic "missing required field".
  let cases: &[(&str, &str)] = &[
    ("/api/v1/nodes", "{}"),
    ("/api/v1/ephemeral-env", "{}"),
    // "fly" is not a valid PowerAction enum variant.
    (
      "/api/v1/power",
      r#"{"action":"fly","targets":["x3000c0s1b0n0"],"target_type":"nodes"}"#,
    ),
    (
      "/api/v1/power",
      r#"{"targets":["x3000c0s1b0n0"],"target_type":"nodes"}"#,
    ),
    (
      "/api/v1/templates/my-template/sessions",
      r#"{"dry_run":false}"#,
    ),
    ("/api/v1/sat-file", "{}"),
    ("/api/v1/kernel-parameters/add", "{}"),
    ("/api/v1/hardware-clusters/my-cluster/members", "{}"),
    ("/api/v1/hardware-clusters/my-cluster/configuration", "{}"),
  ];
  for (uri, body) in cases {
    let resp = router().oneshot(post_json(uri, body)).await.unwrap();
    assert_eq!(
      resp.status(),
      StatusCode::UNPROCESSABLE_ENTITY,
      "expected 422 for POST {} with body {}",
      uri,
      body
    );
  }
}

// ---------------------------------------------------------------------------
// Missing required query parameters
//
// GET /nodes and GET /hardware-nodes-list have a required `xname`/`xnames`
// query parameter. Authenticated requests that omit it get 400 Bad Request.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_nodes_without_xname_returns_400() {
  let resp = router().oneshot(get_auth("/api/v1/nodes")).await.unwrap();
  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_hardware_nodes_list_without_xnames_returns_400() {
  let resp = router()
    .oneshot(get_auth("/api/v1/hardware-nodes-list"))
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
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"", None)
      .unwrap();
  let mut sites = std::collections::HashMap::new();
  sites.insert(
    "test".to_string(),
    SiteBackend {
      backend,
      shasta_base_url: "http://stub.invalid".to_string(),
      shasta_root_cert: vec![],
      socks5_proxy: None,
      vault_base_url: Some("http://vault.stub.invalid".to_string()),
      gitea_base_url: "http://stub.invalid".to_string(),
      k8s_api_url: None, // k8s not set
    },
  );
  let state = Arc::new(ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(1800),
    auditor: None,
    auth_rate_limit_per_minute: None,
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
    .header("X-Manta-Site", "test")
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
    "/api/v1/hardware-nodes-list",
    "/api/v1/health",
    "/api/v1/nodes/x3000c0s1b0n0/console",
    "/api/v1/sessions/my-session/console",
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
    "/api/v1/kernel-parameters/add",
    "/api/v1/migrate/nodes",
    "/api/v1/migrate/backup",
    "/api/v1/migrate/restore",
    "/api/v1/ephemeral-env",
    "/api/v1/power",
    "/api/v1/templates/my-template/sessions",
    "/api/v1/sat-file",
    "/api/v1/hardware-clusters/my-cluster/members",
    "/api/v1/hardware-clusters/my-cluster/configuration",
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
    "/api/v1/kernel-parameters",
    "/api/v1/redfish-endpoints/x3000c0s1b0",
    "/api/v1/sessions/my-session",
    "/api/v1/images",
    "/api/v1/configurations",
    "/api/v1/hardware-clusters/my-cluster/members",
  ] {
    assert_route_exists(Method::DELETE, uri).await;
  }
}

// ---------------------------------------------------------------------------
// Body validation (422) for body-carrying endpoints not covered by
// `post_routes_reject_invalid_bodies` above (DELETE method).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_kernel_parameters_missing_params_returns_422() {
  let resp = router()
    .oneshot(
      Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/kernel-parameters")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer test-token")
        .header("X-Manta-Site", "test")
        .body(Body::from("{}"))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ---------------------------------------------------------------------------
// 501-complement tests — vault/k8s configured → guard does NOT fire
//
// When vault and k8s ARE configured the relevant 501 guard passes and the
// handler attempts a real backend call.  The stub URL fails with a network
// error (5xx), but crucially that is NOT 501 Not Implemented.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_session_with_vault_does_not_return_501() {
  let resp = router_with_vault()
    .oneshot(post_json(
      "/api/v1/sessions",
      r#"{"repo_names":["cray/foo"],"repo_last_commit_ids":["abc123"]}"#,
    ))
    .await
    .unwrap();
  assert_ne!(
    resp.status(),
    StatusCode::NOT_IMPLEMENTED,
    "create_session should pass the vault 501 guard when vault is configured"
  );
}

#[tokio::test]
async fn get_session_logs_with_vault_and_k8s_does_not_return_501() {
  let resp = router_with_vault()
    .oneshot(get_auth("/api/v1/sessions/my-session/logs"))
    .await
    .unwrap();
  assert_ne!(
    resp.status(),
    StatusCode::NOT_IMPLEMENTED,
    "get_session_logs should pass the k8s 501 guard when k8s is configured"
  );
}

// ---------------------------------------------------------------------------
// WebSocket console auth tests
// ---------------------------------------------------------------------------

/// Build a minimal WebSocket upgrade GET request (no actual TCP connection).
/// `WebSocketUpgrade` requires these headers to extract successfully, which
/// lets handler-body checks (auth, 501 guards) run and return HTTP errors.
fn ws_upgrade(uri: &str) -> Request<Body> {
  Request::builder()
    .method(Method::GET)
    .uri(uri)
    .header(header::CONNECTION, "Upgrade")
    .header(header::UPGRADE, "websocket")
    .header("Sec-WebSocket-Version", "13")
    .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
    .body(Body::empty())
    .unwrap()
}

#[tokio::test]
async fn console_node_without_auth_returns_401() {
  let resp = router()
    .oneshot(ws_upgrade("/api/v1/nodes/x3000c0s1b0n0/console"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn console_session_without_auth_returns_401() {
  let resp = router()
    .oneshot(ws_upgrade("/api/v1/sessions/my-session/console"))
    .await
    .unwrap();
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// OpenAPI spec tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn openapi_json_returns_200() {
  let resp = router().oneshot(get("/openapi.json")).await.unwrap();
  assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn openapi_spec_contains_expected_paths_and_schemas() {
  let resp = router().oneshot(get("/openapi.json")).await.unwrap();
  assert_eq!(resp.status(), StatusCode::OK);
  let body = body_string(resp.into_body()).await;
  let spec: serde_json::Value =
    serde_json::from_str(&body).expect("OpenAPI spec must be valid JSON");
  let paths = spec["paths"]
    .as_object()
    .expect("spec must have a paths object");
  assert!(
    paths.contains_key("/sessions"),
    "spec must document /sessions"
  );
  assert!(paths.contains_key("/health"), "spec must document /health");
  assert!(
    paths.contains_key("/boot-parameters"),
    "spec must document /boot-parameters"
  );
  assert!(
    spec["components"]["schemas"].is_object(),
    "spec must have a schemas object"
  );
}

// ---------------------------------------------------------------------------
// Error mapping unit tests
// ---------------------------------------------------------------------------

#[test]
fn to_handler_error_not_found_variants() {
  use axum::http::StatusCode;
  use manta_backend_dispatcher::error::Error;
  use manta_server::server::handlers::to_handler_error;

  assert_eq!(
    to_handler_error(Error::NotFound("session foo".into())).0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    to_handler_error(Error::SessionNotFound).0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    to_handler_error(Error::ConfigurationNotFound).0,
    StatusCode::NOT_FOUND
  );
}

#[test]
fn to_handler_error_conflict_variants() {
  use axum::http::StatusCode;
  use manta_backend_dispatcher::error::Error;
  use manta_server::server::handlers::to_handler_error;

  assert_eq!(
    to_handler_error(Error::Conflict("group foo".into())).0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    to_handler_error(Error::ConfigurationAlreadyExistsError("cfg".into())).0,
    StatusCode::CONFLICT
  );
}

#[test]
fn to_handler_error_bad_request_variants() {
  use axum::http::StatusCode;
  use manta_backend_dispatcher::error::Error;
  use manta_server::server::handlers::to_handler_error;

  for err in [
    Error::BadRequest("bad input".into()),
    Error::InvalidPattern("not-a-pattern".into()),
    Error::UnsupportedBackend("unknown".into()),
    Error::InvalidNodeId("x9999".into()),
  ] {
    let label = format!("{:?}", err);
    assert_eq!(
      to_handler_error(err).0,
      StatusCode::BAD_REQUEST,
      "expected 400 for {}",
      label
    );
  }
}

#[test]
fn to_handler_error_unauthorized_variants() {
  use axum::http::StatusCode;
  use manta_backend_dispatcher::error::Error;
  use manta_server::server::handlers::to_handler_error;

  for err in [
    Error::AuthenticationTokenNotFound("no header".into()),
    Error::JwtMalformed("bad claims".into()),
  ] {
    let label = format!("{:?}", err);
    assert_eq!(
      to_handler_error(err).0,
      StatusCode::UNAUTHORIZED,
      "expected 401 for {}",
      label
    );
  }
}

#[test]
fn to_handler_error_unprocessable_variants() {
  use axum::http::StatusCode;
  use manta_backend_dispatcher::error::Error;
  use manta_server::server::handlers::to_handler_error;

  assert_eq!(
    to_handler_error(Error::InsufficientResources("no nodes free".into())).0,
    StatusCode::UNPROCESSABLE_ENTITY
  );
}

#[test]
fn to_handler_error_unmapped_variant_defaults_to_500() {
  use axum::http::StatusCode;
  use manta_backend_dispatcher::error::Error;
  use manta_server::server::handlers::to_handler_error;

  // `Message` is the catch-all for anything not explicitly mapped;
  // anything else not in the match arms above falls through here too.
  // Pinning this so future variant additions are caught by tests if
  // they get accidentally bucketed into the default 500.
  assert_eq!(
    to_handler_error(Error::Message("something broke".into())).0,
    StatusCode::INTERNAL_SERVER_ERROR
  );
}
