//! Router-level tests for server-side site resolution: requests
//! without `X-Manta-Site` on the endpoints that extract
//! `SiteResolvingCtx`, against a wiremock `manta-cache-server`.
//!
//! The stub backend is the OpenCHAMI dispatcher (pass-through token
//! exchange) with unreachable URLs, so a request that gets past
//! resolution fails later in the backend call. These tests therefore
//! assert on what resolution decides — the status of a resolution
//! failure, the echoed `X-Manta-Site` header, the audit event — never
//! on a successful backend answer.

use std::sync::{Arc, Mutex};

use axum::{
  body::Body,
  http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt as _;
use tower::ServiceExt as _;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use manta_server::config::SiteResolutionConfig;
use manta_server::dispatcher::StaticBackendDispatcher;
use manta_server::server::{
  ServerState, SiteBackend,
  audit::{AuditLog, AuditSink, event::AuditEvent},
  routes::build_router,
  site_resolution::SiteResolver,
};

#[derive(Default)]
struct Capture(Mutex<Vec<AuditEvent>>);

impl AuditSink for Capture {
  fn emit(&self, event: &AuditEvent) {
    self.0.lock().unwrap().push(event.clone());
  }
}

/// Router hosting site `alps`, resolving through `cache` when given.
fn router(cache: Option<&MockServer>, sink: Arc<Capture>) -> axum::Router {
  let backend =
    StaticBackendDispatcher::new("ochami", "http://stub.invalid", b"", None)
      .unwrap();
  let mut sites = std::collections::HashMap::new();
  sites.insert(
    "alps".to_string(),
    SiteBackend {
      backend,
      shasta_base_url: "http://stub.invalid".to_string(),
      shasta_root_cert: vec![],
      vault_base_url: None,
      gitea_base_url: "http://stub.invalid".to_string(),
      k8s_api_url: None,
    },
  );
  let site_resolver = cache.map(|server| {
    SiteResolver::new(&SiteResolutionConfig {
      url: server.uri(),
      api_token_file: None,
      root_ca_cert_file: None,
    })
    .unwrap()
  });
  build_router(Arc::new(ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(1800),
    audit: AuditLog::with_sinks(vec![sink]),
    request_timeout: std::time::Duration::from_secs(60),
    shutdown_grace_period: std::time::Duration::from_secs(30),
    migrate_backup_root: None,
    site_resolver,
  }))
}

fn get(uri: &str) -> Request<Body> {
  Request::builder()
    .method(Method::GET)
    .uri(uri)
    .header(header::AUTHORIZATION, "Bearer test-token")
    .body(Body::empty())
    .unwrap()
}

fn post_json(uri: &str, body: serde_json::Value) -> Request<Body> {
  Request::builder()
    .method(Method::POST)
    .uri(uri)
    .header(header::AUTHORIZATION, "Bearer test-token")
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(body.to_string()))
    .unwrap()
}

async fn body_string(response: axum::response::Response) -> String {
  let bytes = response.into_body().collect().await.unwrap().to_bytes();
  String::from_utf8_lossy(&bytes).into_owned()
}

fn echoed_site(response: &axum::response::Response) -> Option<&str> {
  response
    .headers()
    .get("X-Manta-Site")
    .and_then(|v| v.to_str().ok())
}

async fn mount_group(
  server: &MockServer,
  label: &str,
  status: u16,
  body: serde_json::Value,
) {
  Mock::given(method("GET"))
    .and(path(format!("/api/v1/lookup/group/{label}")))
    .respond_with(ResponseTemplate::new(status).set_body_json(body))
    .mount(server)
    .await;
}

#[tokio::test]
async fn resolved_site_is_echoed_and_audited() {
  let cache = MockServer::start().await;
  mount_group(&cache, "zinal", 200, serde_json::json!({ "site": "alps" }))
    .await;
  let sink = Arc::new(Capture::default());

  let response = router(Some(&cache), sink.clone())
    .oneshot(post_json(
      "/v2/power",
      serde_json::json!({
        "action": "off", "target_type": "cluster", "host_expression": "zinal"
      }),
    ))
    .await
    .unwrap();

  assert_eq!(echoed_site(&response), Some("alps"));
  let events = std::mem::take(&mut *sink.0.lock().unwrap());
  assert_eq!(events.len(), 1);
  assert_eq!(events[0].site.as_deref(), Some("alps"));
  assert_eq!(
    events[0].details.get("site_resolved_from"),
    Some(&serde_json::json!("group 'zinal'"))
  );
}

#[tokio::test]
async fn xname_list_resolves_through_the_nodes_lookup() {
  let cache = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/api/v1/lookup/nodes"))
    .and(query_param("xnames", "x1000c0s0b0n0,x1000c0s0b0n1"))
    .respond_with(ResponseTemplate::new(200).set_body_json(
      serde_json::json!({ "site": "alps", "resolutions": {}, "unknown": [] }),
    ))
    .expect(1)
    .mount(&cache)
    .await;

  let response = router(Some(&cache), Arc::default())
    .oneshot(get("/v2/nodes?xname=x1000c0s0b0n0,x1000c0s0b0n1"))
    .await
    .unwrap();
  assert_eq!(echoed_site(&response), Some("alps"));
}

#[tokio::test]
async fn explicit_site_header_bypasses_the_cache() {
  let cache = MockServer::start().await;
  Mock::given(method("GET"))
    .respond_with(ResponseTemplate::new(500))
    .expect(0)
    .mount(&cache)
    .await;

  let mut request = get("/v2/groups/nodes?hsm_group=zinal");
  request
    .headers_mut()
    .insert("X-Manta-Site", "alps".parse().unwrap());
  let response = router(Some(&cache), Arc::default())
    .oneshot(request)
    .await
    .unwrap();
  // The caller named the site; nothing to echo.
  assert_eq!(echoed_site(&response), None);
}

#[tokio::test]
async fn unknown_group_is_404() {
  let cache = MockServer::start().await;
  mount_group(&cache, "nope", 404, serde_json::json!({ "error": "x" })).await;

  let response = router(Some(&cache), Arc::default())
    .oneshot(get("/v2/groups/nodes?hsm_group=nope"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::NOT_FOUND);
  assert!(
    body_string(response)
      .await
      .contains("knows no group 'nope'")
  );
}

#[tokio::test]
async fn split_xname_list_is_409() {
  let cache = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/api/v1/lookup/nodes"))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "site": null,
      "resolutions": { "x1000c0s0b0n0": "alps", "x3000c0s0b0n0": "daint" },
      "unknown": []
    })))
    .mount(&cache)
    .await;

  let response = router(Some(&cache), Arc::default())
    .oneshot(get("/v2/nodes?xname=x1000c0s0b0n0,x3000c0s0b0n0"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::CONFLICT);
  let body = body_string(response).await;
  assert!(body.contains("x3000c0s0b0n0 → daint"), "{body}");
}

#[tokio::test]
async fn cache_outage_is_503() {
  let cache = MockServer::start().await;
  mount_group(&cache, "zinal", 502, serde_json::json!({})).await;

  let response = router(Some(&cache), Arc::default())
    .oneshot(get("/v2/groups?name=zinal"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
  assert!(body_string(response).await.contains("--site"));
}

#[tokio::test]
async fn resolved_site_must_still_be_configured() {
  let cache = MockServer::start().await;
  mount_group(&cache, "zinal", 200, serde_json::json!({ "site": "daint" }))
    .await;

  let response = router(Some(&cache), Arc::default())
    .oneshot(get("/v2/groups/nodes?hsm_group=zinal"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::NOT_FOUND);
  assert!(
    body_string(response)
      .await
      .contains("site 'daint' not found")
  );
}

#[tokio::test]
async fn request_without_a_resolvable_target_is_400() {
  let cache = MockServer::start().await;
  Mock::given(method("GET"))
    .respond_with(ResponseTemplate::new(500))
    .expect(0)
    .mount(&cache)
    .await;
  let app = router(Some(&cache), Arc::default());

  // No filter at all.
  let response = app.clone().oneshot(get("/v2/sessions")).await.unwrap();
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  assert!(body_string(response).await.contains("names no group"));

  // A hostlist expression needs a site to expand.
  let response = app
    .oneshot(get("/v2/nodes?xname=x1000c0s%5B0-3%5Db0n0"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_site_without_resolver_is_400() {
  let response = router(None, Arc::default())
    .oneshot(get("/v2/groups/nodes?hsm_group=zinal"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  assert!(body_string(response).await.contains("not configured"));
}

#[tokio::test]
async fn endpoints_without_resolution_still_require_the_header() {
  let cache = MockServer::start().await;
  let response = router(Some(&cache), Arc::default())
    .oneshot(get("/v2/images"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  assert!(
    body_string(response)
      .await
      .contains("Missing X-Manta-Site header")
  );
}
