//! Offline tests of the [`refresh`] HTTP path against a local wiremock
//! server standing in for manta-server.
//!
//! These cover what the unit tests cannot — the wire behaviour: request
//! headers (`X-Manta-Site`, a new bearer token per request), the
//! one-call-per-site fan-out, tolerance of unknown JSON fields, and the
//! mapping of token and HTTP failures into [`RefreshOutcome::failures`].

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use manta_cache::{
  CacheError, RefreshOutcome, SiteDescriptor, TokenError, TokenSource, refresh,
};
use serde_json::json;
use wiremock::matchers::{header, header_regex, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Hands out `tok-1`, `tok-2`, … — a new token per call, like the
/// service account's client credentials grant.
#[derive(Default)]
struct Counter(AtomicUsize);

impl TokenSource for Counter {
  async fn token(&self) -> Result<String, TokenError> {
    Ok(format!("tok-{}", self.0.fetch_add(1, Ordering::SeqCst) + 1))
  }
}

/// Never supplies a token.
struct Refused;

impl TokenSource for Refused {
  async fn token(&self) -> Result<String, TokenError> {
    Err("service-account login rejected: Invalid client secret".into())
  }
}

fn descriptor(server: &MockServer, site: &str) -> SiteDescriptor {
  SiteDescriptor {
    name: site.to_owned(),
    manta_server_url: server.uri(),
  }
}

/// Mount `GET /v2/groups` for one site, answering `groups`, matching
/// on its `X-Manta-Site` header and a [`Counter`] bearer token so a
/// request with the wrong headers falls through to wiremock's 404.
async fn mount_site(
  server: &MockServer,
  site: &str,
  groups: serde_json::Value,
) {
  Mock::given(method("GET"))
    .and(path("/v2/groups"))
    .and(header("X-Manta-Site", site))
    .and(header_regex("Authorization", r"^Bearer tok-\d+$"))
    .respond_with(ResponseTemplate::new(200).set_body_json(groups))
    .expect(1)
    .mount(server)
    .await;
}

#[tokio::test]
async fn refresh_fans_out_with_per_site_headers() {
  let server = MockServer::start().await;

  // Two sites behind one manta-server URL, exactly as SiteDescriptor
  // documents; only the headers tell them apart. The group objects
  // carry extra fields to prove the deserialiser ignores them.
  mount_site(
    &server,
    "alps",
    json!([{
      "label": "compute",
      "description": "Compute nodes",
      "members": {"ids": ["x1000c0s0b0n0"]},
    }]),
  )
  .await;
  mount_site(
    &server,
    "daint",
    json!([
      {"label": "compute_d", "members": {"ids": ["x2000c0s0b0n0"]}},
      {"label": "extra", "tags": ["t"], "members": {"ids": ["x2000c0s0b0n0"]}},
    ]),
  )
  .await;

  let outcome: RefreshOutcome = refresh(
    &[descriptor(&server, "alps"), descriptor(&server, "daint")],
    &Counter::default(),
  )
  .await
  .expect("client build");

  assert!(outcome.is_complete(), "failures: {:?}", outcome.failures);
  let index = outcome.index;
  assert_eq!(index.sites().collect::<Vec<_>>(), vec!["alps", "daint"]);
  assert_eq!(index.group_to_site("compute"), Some("alps"));
  assert_eq!(index.group_to_site("compute_d"), Some("daint"));
  assert_eq!(index.group_to_site("extra"), Some("daint"));
  assert_eq!(
    index.group_members("extra"),
    Some(&["x2000c0s0b0n0".to_owned()][..])
  );
  assert_eq!(index.xname_to_site("x1000c0s0b0n0"), Some("alps"));
  assert_eq!(index.xname_to_site("x2000c0s0b0n0"), Some("daint"));
  // MockServer verifies each `.expect(1)` on drop: one call per site,
  // carrying that site's own headers.
}

/// No token is sent twice: OpenCHAMI's Hydra would reject the reuse.
#[tokio::test]
async fn every_request_carries_a_new_token() {
  let server = MockServer::start().await;
  for site in ["alps", "daint"] {
    mount_site(&server, site, json!([])).await;
  }
  let tokens = Counter::default();

  let outcome = refresh(
    &[descriptor(&server, "alps"), descriptor(&server, "daint")],
    &tokens,
  )
  .await
  .expect("client build");

  assert!(outcome.is_complete(), "failures: {:?}", outcome.failures);
  let sent: Vec<String> = server
    .received_requests()
    .await
    .unwrap()
    .iter()
    .map(|r| r.headers["authorization"].to_str().unwrap().to_owned())
    .collect();
  assert_eq!(sent.len(), 2, "{sent:?}");
  assert_eq!(sent.iter().collect::<HashSet<_>>().len(), 2, "{sent:?}");
  assert_eq!(tokens.0.load(Ordering::SeqCst), 2);
}

/// A token the source cannot supply fails each site with
/// [`CacheError::Token`], and no request goes out without one.
#[tokio::test]
async fn a_token_failure_fails_the_site_without_sending() {
  let server = MockServer::start().await;

  let outcome = refresh(
    &[descriptor(&server, "alps"), descriptor(&server, "daint")],
    &Refused,
  )
  .await
  .expect("client build");

  assert_eq!(outcome.index.sites().count(), 0);
  let failed: Vec<_> = outcome.failures.iter().map(CacheError::site).collect();
  assert_eq!(failed, vec![Some("alps"), Some("daint")]);
  for failure in &outcome.failures {
    assert!(matches!(failure, CacheError::Token { .. }), "{failure:?}");
    assert!(
      failure.to_string().contains("Invalid client secret"),
      "{failure}"
    );
  }
  assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn refresh_collects_failures_and_keeps_healthy_sites() {
  let server = MockServer::start().await;

  mount_site(
    &server,
    "good",
    json!([{"label": "compute", "members": {"ids": ["x1000c0s0b0n0"]}}]),
  )
  .await;
  Mock::given(method("GET"))
    .and(path("/v2/groups"))
    .and(header("X-Manta-Site", "bad"))
    .respond_with(ResponseTemplate::new(500).set_body_json(json!({
      "error": "backend exploded"
    })))
    .mount(&server)
    .await;

  let outcome = refresh(
    &[descriptor(&server, "good"), descriptor(&server, "bad")],
    &Counter::default(),
  )
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
    .and(path("/v2/groups"))
    .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
    .mount(&server)
    .await;

  let outcome = refresh(&[descriptor(&server, "alps")], &Counter::default())
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
    .and(path("/v2/groups"))
    .respond_with(ResponseTemplate::new(401))
    .mount(&server)
    .await;

  let outcome = refresh(&[descriptor(&server, "alps")], &Counter::default())
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
