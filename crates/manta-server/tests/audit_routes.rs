//! Router-level tests for the audit trail: which requests produce an
//! event, and what the event carries.
//!
//! The coverage test walks the OpenAPI spec rather than a hand-kept
//! list, so a new mutating route that somehow bypasses the audit layer
//! fails here without anyone remembering to add it.

use std::sync::{Arc, Mutex};

use axum::{
  body::Body,
  http::{Method, Request, StatusCode, header},
};
use base64::prelude::*;
use tower::ServiceExt as _;
use utoipa::OpenApi as _;

use manta_server::dispatcher::StaticBackendDispatcher;
use manta_server::server::{
  ServerState, SiteBackend,
  api_doc::ApiDoc,
  audit::{
    AuditLog, AuditSink,
    event::{AuditEvent, EventKind, Verdict},
  },
  routes::build_router,
};

#[derive(Default)]
struct Capture(Mutex<Vec<AuditEvent>>);

impl AuditSink for Capture {
  fn emit(&self, event: &AuditEvent) {
    self.0.lock().unwrap().push(event.clone());
  }
}

impl Capture {
  fn take(&self) -> Vec<AuditEvent> {
    std::mem::take(&mut self.0.lock().unwrap())
  }
}

fn router(sink: Arc<Capture>) -> axum::Router {
  router_with_timeout(sink, std::time::Duration::from_secs(60))
}

fn router_with_timeout(
  sink: Arc<Capture>,
  request_timeout: std::time::Duration,
) -> axum::Router {
  let backend =
    StaticBackendDispatcher::new("csm", "http://stub.invalid", b"").unwrap();
  let mut sites = std::collections::HashMap::new();
  sites.insert(
    "test".to_string(),
    SiteBackend {
      backend,
      shasta_base_url: "http://stub.invalid".to_string(),
      shasta_root_cert: vec![],
      vault_base_url: None,
      gitea_base_url: "http://stub.invalid".to_string(),
      k8s_api_url: None,
    },
  );
  build_router(Arc::new(ServerState {
    sites,
    console_inactivity_timeout: std::time::Duration::from_secs(1800),
    audit: AuditLog::with_sinks(vec![sink]),
    auth_rate_limit_per_minute: None,
    request_timeout,
    shutdown_grace_period: std::time::Duration::from_secs(30),
    migrate_backup_root: None,
  }))
}

/// An unsigned JWT carrying the identity claims the audit layer reads.
fn jwt() -> String {
  let claims = serde_json::json!({
    "preferred_username": "alice",
    "name": "Alice Admin",
    "sub": "0b5e-subject",
    // Admin, so handlers skip the group-claim checks and reach the
    // (unreachable) backend.
    "realm_access": { "roles": ["pa_admin"] },
  });
  format!(
    "{}.{}.sig",
    BASE64_URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#),
    BASE64_URL_SAFE_NO_PAD.encode(claims.to_string())
  )
}

fn request(method: Method, uri: &str) -> Request<Body> {
  Request::builder()
    .method(method)
    .uri(uri)
    .body(Body::empty())
    .unwrap()
}

/// Every operation in the spec (outside `/auth/*`, which audits
/// itself) produces an event iff it mutates or attaches a console.
#[tokio::test]
async fn exactly_mutating_operations_and_consoles_are_audited() {
  let spec = serde_json::to_value(ApiDoc::openapi()).unwrap();
  let paths = spec["paths"].as_object().unwrap();
  let mut checked = 0;
  for (template, item) in paths {
    if template.starts_with("/auth/") {
      continue;
    }
    // `{param}` → a concrete segment.
    let concrete = template
      .split('/')
      .map(|s| if s.starts_with('{') { "p" } else { s })
      .collect::<Vec<_>>()
      .join("/");
    for method in ["get", "post", "put", "patch", "delete"] {
      if item.get(method).is_none() {
        continue;
      }
      let m = Method::from_bytes(method.to_uppercase().as_bytes()).unwrap();
      let sink = Arc::new(Capture::default());
      let response = router(sink.clone())
        .oneshot(request(m.clone(), &format!("/v2{concrete}")))
        .await
        .unwrap();
      let events = sink.take();
      let expected = m != Method::GET || template.ends_with("/console");
      assert_eq!(
        events.len(),
        usize::from(expected),
        "{m} {template} (status {}): expected {} event(s)",
        response.status(),
        usize::from(expected),
      );
      if let Some(e) = events.first() {
        assert_eq!(e.request.route, format!("/v2{template}"));
        assert_eq!(e.request.method, m.as_str());
        assert_eq!(e.outcome.status, response.status().as_u16());
      }
      checked += 1;
    }
  }
  assert!(checked > 40, "spec walk found only {checked} operations");
}

#[tokio::test]
async fn event_carries_identity_site_client_and_outcome() {
  let sink = Arc::new(Capture::default());
  let mut req = Request::builder()
    .method(Method::DELETE)
    .uri("/v2/images?ids=abc&dry_run=true")
    .header(header::AUTHORIZATION, format!("Bearer {}", jwt()))
    .header("X-Manta-Site", "nosuchsite")
    .header("X-Forwarded-For", "192.0.2.7")
    .body(Body::empty())
    .unwrap();
  req.extensions_mut().insert(axum::extract::ConnectInfo(
    "10.1.2.3:40000".parse::<std::net::SocketAddr>().unwrap(),
  ));
  let response = router(sink.clone()).oneshot(req).await.unwrap();

  let events = sink.take();
  let e = &events[0];
  assert_eq!(e.kind, EventKind::ApiRequest);
  assert_eq!(e.actor.username.as_deref(), Some("alice"));
  assert_eq!(e.actor.name.as_deref(), Some("Alice Admin"));
  assert_eq!(e.actor.subject.as_deref(), Some("0b5e-subject"));
  assert_eq!(e.site.as_deref(), Some("nosuchsite"));
  assert_eq!(e.client.ip.as_deref(), Some("10.1.2.3"));
  assert_eq!(e.client.forwarded_for.as_deref(), Some("192.0.2.7"));
  assert_eq!(e.request.path, "/v2/images");
  assert_eq!(e.request.query.as_deref(), Some("ids=abc&dry_run=true"));
  // Unknown site → 404 from extraction, before the handler ran: the
  // action falls back to the route.
  assert_eq!(response.status(), StatusCode::NOT_FOUND);
  assert_eq!(e.outcome.result, Verdict::Rejected);
  assert_eq!(e.action, "DELETE /v2/images");

  let json = serde_json::to_string(e).unwrap();
  assert!(!json.contains(&jwt()), "token leaked into event: {json}");
}

#[tokio::test]
async fn unauthenticated_mutation_is_recorded_as_denied() {
  let sink = Arc::new(Capture::default());
  let response = router(sink.clone())
    .oneshot(request(Method::DELETE, "/v2/groups/g1"))
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  let e = &sink.take()[0];
  assert_eq!(e.outcome.result, Verdict::Denied);
  assert_eq!(e.actor.username, None);
}

#[tokio::test]
async fn reads_unknown_routes_and_wrong_methods_are_not_audited() {
  let sink = Arc::new(Capture::default());
  for (m, uri) in [
    (Method::GET, "/v2/groups"),
    (Method::GET, "/v2/health"),
    (Method::POST, "/v2/no-such-route"),
    (Method::DELETE, "/v2/health"),
  ] {
    router(sink.clone()).oneshot(request(m, uri)).await.unwrap();
  }
  assert!(sink.take().is_empty());
}

/// The audit layer sits outside the request timeout, so a request the
/// server gave up on is still recorded. A zero timeout fires as soon
/// as the handler first waits on the (unreachable) backend.
#[tokio::test]
async fn timed_out_mutation_is_still_audited() {
  let sink = Arc::new(Capture::default());
  let req = Request::builder()
    .method(Method::DELETE)
    .uri("/v2/groups/g1")
    .header(header::AUTHORIZATION, format!("Bearer {}", jwt()))
    .header("X-Manta-Site", "test")
    .body(Body::empty())
    .unwrap();
  let response = router_with_timeout(sink.clone(), std::time::Duration::ZERO)
    .oneshot(req)
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
  let e = &sink.take()[0];
  assert_eq!(e.outcome.result, Verdict::TimedOut);
  assert_eq!(e.outcome.status, 408);
}
