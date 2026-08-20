//! Offline tests of the [`refresh`] HTTP path against a local wiremock
//! server standing in for manta-server.
//!
//! These cover what the unit tests cannot — the wire behaviour: request
//! headers (`X-Manta-Site`, per-site bearer token), the two-call
//! fan-out, tolerance of unknown JSON fields, and the mapping of HTTP
//! failures into [`RefreshOutcome::failures`].

use manta_cache::{CacheError, RefreshOutcome, SiteDescriptor, refresh};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn descriptor(server: &MockServer, site: &str, token: &str) -> SiteDescriptor {
  SiteDescriptor {
    name: site.to_owned(),
    manta_server_url: server.uri(),
    token: token.to_owned(),
  }
}

/// Mount the two group endpoints for one site, matching on its
/// `X-Manta-Site` and bearer headers so a request with the wrong
/// headers falls through to wiremock's 404.
async fn mount_site(
  server: &MockServer,
  site: &str,
  token: &str,
  labels: serde_json::Value,
  nodes: serde_json::Value,
) {
  for (endpoint, body) in [("available", labels), ("nodes", nodes)] {
    Mock::given(method("GET"))
      .and(path(format!("/v2/groups/{endpoint}")))
      .and(header("X-Manta-Site", site))
      .and(header("Authorization", format!("Bearer {token}")))
      .respond_with(ResponseTemplate::new(200).set_body_json(body))
      .expect(1)
      .mount(server)
      .await;
  }
}

#[tokio::test]
async fn refresh_fans_out_with_per_site_headers() {
  let server = MockServer::start().await;

  // Two sites behind one manta-server URL, exactly as SiteDescriptor
  // documents; only the headers tell them apart. The node objects carry
  // extra fields to prove the deserialiser ignores them.
  mount_site(
    &server,
    "alps",
    "tok-alps",
    json!(["compute"]),
    json!([{"xname": "x1000c0s0b0n0", "hsm": "compute", "power_status": "ON"}]),
  )
  .await;
  mount_site(
    &server,
    "daint",
    "tok-daint",
    json!(["compute_d"]),
    json!([{"xname": "x2000c0s0b0n0", "hsm": "compute_d, extra"}]),
  )
  .await;

  let outcome: RefreshOutcome = refresh(&[
    descriptor(&server, "alps", "tok-alps"),
    descriptor(&server, "daint", "tok-daint"),
  ])
  .await
  .expect("client build");

  assert!(outcome.is_complete(), "failures: {:?}", outcome.failures);
  let index = outcome.index;
  assert_eq!(index.sites().collect::<Vec<_>>(), vec!["alps", "daint"]);
  assert_eq!(index.group_to_site("compute"), Some("alps"));
  assert_eq!(index.group_to_site("compute_d"), Some("daint"));
  // "extra" exists only as an hsm mention, split and trimmed.
  assert_eq!(index.group_to_site("extra"), Some("daint"));
  assert_eq!(index.xname_to_site("x1000c0s0b0n0"), Some("alps"));
  assert_eq!(index.xname_to_site("x2000c0s0b0n0"), Some("daint"));
  // MockServer verifies each `.expect(1)` on drop: two calls per site,
  // each carrying that site's own headers.
}

#[tokio::test]
async fn refresh_collects_failures_and_keeps_healthy_sites() {
  let server = MockServer::start().await;

  mount_site(
    &server,
    "good",
    "tok-good",
    json!(["compute"]),
    json!([{"xname": "x1000c0s0b0n0", "hsm": "compute"}]),
  )
  .await;
  Mock::given(method("GET"))
    .and(path("/v2/groups/available"))
    .and(header("X-Manta-Site", "bad"))
    .respond_with(ResponseTemplate::new(500).set_body_json(json!({
      "error": "backend exploded"
    })))
    .mount(&server)
    .await;

  let outcome = refresh(&[
    descriptor(&server, "good", "tok-good"),
    descriptor(&server, "bad", "tok-bad"),
  ])
  .await
  .expect("client build");

  // The healthy site is fully indexed; the failed one is absent.
  assert_eq!(outcome.index.sites().collect::<Vec<_>>(), vec!["good"]);
  assert_eq!(outcome.index.group_to_site("compute"), Some("good"));

  // The failure names the site and carries the body snippet.
  assert_eq!(outcome.failures.len(), 1);
  match &outcome.failures[0] {
    CacheError::Status { site, status, body } => {
      assert_eq!(site, "bad");
      assert_eq!(*status, 500);
      assert!(body.contains("backend exploded"), "body: {body}");
    }
    other => panic!("expected Status error, got {other:?}"),
  }
}

#[tokio::test]
async fn refresh_maps_undecodable_bodies_to_request_errors() {
  let server = MockServer::start().await;

  Mock::given(method("GET"))
    .and(path("/v2/groups/available"))
    .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
    .mount(&server)
    .await;

  let outcome = refresh(&[descriptor(&server, "alps", "tok")])
    .await
    .expect("client build");

  assert_eq!(outcome.index.sites().count(), 0);
  assert_eq!(outcome.failures.len(), 1);
  match &outcome.failures[0] {
    CacheError::Request { site, source } => {
      assert_eq!(site, "alps");
      assert!(source.is_decode(), "expected decode error, got {source}");
    }
    other => panic!("expected Request error, got {other:?}"),
  }
}

#[tokio::test]
async fn refresh_marks_empty_error_bodies() {
  let server = MockServer::start().await;

  Mock::given(method("GET"))
    .and(path("/v2/groups/available"))
    .respond_with(ResponseTemplate::new(401))
    .mount(&server)
    .await;

  let outcome = refresh(&[descriptor(&server, "alps", "tok")])
    .await
    .expect("client build");

  match &outcome.failures[0] {
    CacheError::Status { status, body, .. } => {
      assert_eq!(*status, 401);
      assert_eq!(body, "<empty body>");
    }
    other => panic!("expected Status error, got {other:?}"),
  }
}
