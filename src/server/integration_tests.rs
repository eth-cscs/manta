//! Phase 6 — Happy-path HTTP integration tests.
//!
//! Each test spins up a real wiremock server, builds an Axum router whose
//! backend points at it, and drives requests through
//! `tower::ServiceExt::oneshot` without opening a TCP listener.
//!
//! Covered endpoints:
//!   GET /api/v1/groups
//!   GET /api/v1/configurations
//!   GET /api/v1/sessions
//!   GET /api/v1/templates
//!   GET /api/v1/images
//!   GET /api/v1/boot-parameters

use std::sync::Arc;
use std::time::Duration;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode, header},
  response::Response,
};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{method, path},
};

use crate::{
  manta_backend_dispatcher::StaticBackendDispatcher,
  server::{ServerState, routes::build_router},
};

// ---------------------------------------------------------------------------
// Embedded credentials (CI-safe — no file-system dependency)
//
// TEST_ROOT_CERT: the real ALPS Platform CA certificate.
//   Satisfies reqwest::Certificate::from_pem(), which is called lazily at
//   first backend HTTP call. The wiremock server speaks plain HTTP so TLS
//   is never negotiated and this cert is never verified.
//
// TEST_TOKEN: a real Keycloak JWT for the ALPS system.
//   csm-rs reads only realm_access.roles (no sig/expiry validation), so
//   this token works permanently regardless of its exp date.
//   The payload contains ["pa_admin", ...] which takes the admin path in
//   get_group_name_available, returning all groups without per-group
//   filtering.
// ---------------------------------------------------------------------------

const TEST_ROOT_CERT: &[u8] = b"-----BEGIN CERTIFICATE-----\n\
MIIEuDCCAyCgAwIBAgIUUuuQOz5Bu78gF1uz8lsDCWDRR4cwDQYJKoZIhvcNAQEL\n\
BQAwYTEPMA0GA1UECgwGU2hhc3RhMREwDwYDVQQLDAhQbGF0Zm9ybTE7MDkGA1UE\n\
AwwyUGxhdGZvcm0gQ0EgKGE2NjkzY2ExLTNmNWMtNDY1Zi04ZTAxLWQ4MDFkNGEw\n\
OWE0ZikwHhcNMjIwNTMxMDcxMTI1WhcNMzIwNTI4MDcxMTI1WjBmMQ8wDQYDVQQK\n\
DAZTaGFzdGExETAPBgNVBAsMCFBsYXRmb3JtMUAwPgYDVQQDDDdQbGF0Zm9ybSBD\n\
QSAtIEwxIChhNjY5M2NhMS0zZjVjLTQ2NWYtOGUwMS1kODAxZDRhMDlhNGYpMIIB\n\
ojANBgkqhkiG9w0BAQEFAAOCAY8AMIIBigKCAYEAriBXAeZVnRvUtNAe0V8BUqbn\n\
Ij6gQ8mgBP7c9BLbz3N4ALDswzHyQVIAuKJ7D3VsHVRjkKWqzOVAiP14sLJ8ko/o\n\
Fqc3HyS4L7PC6y9BY3eH2XJ3oKc6EmmlUTGEf4ZdZIvg59Tr9aYSIAqvlS/FtNCy\n\
Ch6jkCltLtHpXlSDEjuWMK8YQZCj2V0cvGoZuW4GhiNWU1amwKvNnsJIRt9uKd2O\n\
Y1GYwV9QcJOjpvWtKerNz00QJ0DNJdCBSJcp2X6sa+uEpJUK7SMM59ZKmrAVdFYo\n\
ROWnlJplEexOULCpbUwqQHsVe0ybOaI/P+Gsa9VB8qn3K+CBF+CKVqh9g2dG3AEC\n\
O04CANmQj4dmeqUFLHzMKOZFbIyShvyzNrIQwzDPKMPeeK9kO5r9Xb76LySv3a0F\n\
KuWB/57ync9RvCEa3WErIetZEG2kxyo/lIH6GaG7/TgYM/y5roBr1bTPAuKPgehj\n\
DMfhWSUoHlOkbNK985fWnsizbCR496AilzY/n1r1AgMBAAGjYzBhMA8GA1UdEwEB\n\
/wQFMAMBAf8wDgYDVR0PAQH/BAQDAgEGMB0GA1UdDgQWBBT+yXIKK9rNq1X2Dg6c\n\
w/VNMBIpujAfBgNVHSMEGDAWgBTtk3qrK5zSqc2ecEVVsIGayNMfPTANBgkqhkiG\n\
9w0BAQsFAAOCAYEAWB+dnLKeDiQC91X845lVGcOV1y1ZXTOEulYM+BLWSoVRBW/h\n\
FyIG1Qeho+Yzxx++vitocKiih6z9wRjKrnTCAehr44vXikTB12MbrmrLROs2zKl3\n\
fU+CUThK6vBhx+kqVU1Xcxf+PSgmgsOhOhZmW0fVwPKFJtxAR6KBOvCN0mBj82xd\n\
yFkNG7+FUxPnvwHWT8NWukfFopFYMw5bWD8B7rpGt1fa41CMBH/8WXdh4QTAdvQu\n\
YeSQqdnEqRhDni1AMIgm8ubh1A89jEMYj2oGJt6WuUMDmMIXJNYbiTeN/3M3rDBV\n\
3WE0Rm+rM1nsVtXH+0ibFgicE6BH67wasXYyiNDin+dKY6bk/nIw2aZCVjQvVPuX\n\
tfnAjIbZCtohCCdOU1eQ/fJwlqz51WJz3Ti846zkjaXjnLeGiW51XWZpVcz1y06j\n\
R7v/4/prpr1EL7t4ZwBOZhqkvJ4IL/Nv/SLiCjgg5b6b8WtVIHLosEd7ca5lhJ+v\n\
ce8Dbp369gQPR3Eu\n\
-----END CERTIFICATE-----\n";

const TEST_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJNSW5BOEFfUUd4RTJ3REI5RlNkTzRKelVYSE9wVWFqZXVVb3JXemx1QlQwIn0.eyJleHAiOjE3NzcyMzM2NTcsImlhdCI6MTc3NjYyODg1NywianRpIjoiYTVjNmIxNTgtZmZlNy00Y2UzLTljZWYtYjJkNWY1NjA4MTk1IiwiaXNzIjoiaHR0cHM6Ly9hcGkuY21uLmFscHMuY3Njcy5jaC9rZXljbG9hay9yZWFsbXMvc2hhc3RhIiwiYXVkIjpbInNoYXN0YSIsInN5c3RlbS1uZXh1cy1jbGllbnQiLCJhY2NvdW50Il0sInN1YiI6IjYxMTgwOGY4LWZlMTAtNDRkNi04Nzc4LWJhZTBiODVmZDk5MiIsInR5cCI6IkJlYXJlciIsImF6cCI6InNoYXN0YSIsInNlc3Npb25fc3RhdGUiOiIzZmI3YjcwMC0xMjZjLTQwNjktYWJjOC0yMDY5YzRhNjcyY2YiLCJyZWFsbV9hY2Nlc3MiOnsicm9sZXMiOlsicGFfYWRtaW4iLCJhbHBzIiwidGFwbXNfd2lsZGhvcm4iLCJkZWZhdWx0LXJvbGVzLXNoYXN0YSIsInRlbmFudC1hZG1pbiIsIm9mZmxpbmVfYWNjZXNzIiwiZm9yYSIsInVtYV9hdXRob3JpemF0aW9uIl19LCJyZXNvdXJjZV9hY2Nlc3MiOnsic2hhc3RhIjp7InJvbGVzIjpbImFkbWluIiwidXNlciJdfSwic3lzdGVtLW5leHVzLWNsaWVudCI6eyJyb2xlcyI6WyJueC1hZG1pbiJdfSwiYWNjb3VudCI6eyJyb2xlcyI6WyJtYW5hZ2UtYWNjb3VudCIsIm1hbmFnZS1hY2NvdW50LWxpbmtzIiwidmlldy1wcm9maWxlIl19fSwic2NvcGUiOiJvcGVuaWQgcHJvZmlsZSBlbWFpbCIsInNpZCI6IjNmYjdiNzAwLTEyNmMtNDA2OS1hYmM4LTIwNjljNGE2NzJjZiIsImVtYWlsX3ZlcmlmaWVkIjpmYWxzZSwibmFtZSI6Ik1hbnVlbCBTb3BlbmEiLCJncm91cHMiOltdLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJtc29wZW5hIiwiZ2l2ZW5fbmFtZSI6Ik1hbnVlbCIsImZhbWlseV9uYW1lIjoiU29wZW5hIiwiZW1haWwiOiJtYW51ZWwuc29wZW5hQGNzY3MuY2gifQ.n4TrFTbL0XZuwIS69uLqbtqDJalWa_6UXek1mcJkKe1rQZBw3tnpek7yVIVJf5yFvlMi7SeQpsQorzXqFvwc0YAYODJNvAibuTZVVIrxbUdWfH9QR92xhp3BLkwMQtLT_4VK_1vXo8rsmtvDYKvQOqhO6PCqD0s1h4gBXQRssRG5Doo451KNTkGSoiZRATMo7KlQIGGLAXv7M2avrXhxOuE_ERlFMDWaXfL2aPjdO-e3xF0ZoFUte-L0r91QQUyRauQ3Ce_Abo5k1RYB744zdCQDDvP9qZL6SgkXEcXfF5GMCqNpA4aM7rVhl6hK_Jin13HCgi2pB3RU06J3y3zNBQ";

// ---------------------------------------------------------------------------
// TestFixture
// ---------------------------------------------------------------------------

struct TestFixture {
  mock_server: MockServer,
  router: axum::Router,
}

impl TestFixture {
  async fn setup() -> Self {
    let mock_server = MockServer::start().await;

    let backend = StaticBackendDispatcher::new(
      "csm",
      &mock_server.uri(),
      TEST_ROOT_CERT,
    )
    .unwrap();

    let state = Arc::new(ServerState {
      backend,
      site_name: "test".to_string(),
      shasta_base_url: mock_server.uri(),
      shasta_root_cert: TEST_ROOT_CERT.to_vec(),
      vault_base_url: None,
      gitea_base_url: "http://stub.invalid".to_string(),
      k8s_api_url: None,
      console_inactivity_timeout: Duration::from_secs(1800),
    });

    let router = build_router(state);

    TestFixture { mock_server, router }
  }

  fn auth_get(&self, uri: &str) -> Request<Body> {
    Request::builder()
      .method(Method::GET)
      .uri(uri)
      .header(header::AUTHORIZATION, format!("Bearer {TEST_TOKEN}"))
      .body(Body::empty())
      .unwrap()
  }

  async fn send(&self, req: Request<Body>) -> Response {
    self.router.clone().oneshot(req).await.unwrap()
  }

  async fn body_json(resp: Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).expect("response body is not valid JSON")
  }

  fn auth_post_json(&self, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
      .method(Method::POST)
      .uri(uri)
      .header(header::AUTHORIZATION, format!("Bearer {TEST_TOKEN}"))
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_string(&body).unwrap()))
      .unwrap()
  }

  fn auth_delete(&self, uri: &str) -> Request<Body> {
    Request::builder()
      .method(Method::DELETE)
      .uri(uri)
      .header(header::AUTHORIZATION, format!("Bearer {TEST_TOKEN}"))
      .body(Body::empty())
      .unwrap()
  }

  fn auth_delete_json(&self, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
      .method(Method::DELETE)
      .uri(uri)
      .header(header::AUTHORIZATION, format!("Bearer {TEST_TOKEN}"))
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_string(&body).unwrap()))
      .unwrap()
  }
}

// ---------------------------------------------------------------------------
// Wiremock stub helpers
//
// wiremock::matchers::path() matches the path component only — query strings
// are ignored. A single stub for /smd/hsm/v2/groups therefore handles both
// the admin "get all" call (no query params) and any filtered call
// (?group=compute).
//
// Version notes:
//   - csm-rs uses CFS v2 for both sessions AND configurations inside
//     get_and_filter_sessions / get_and_filter_configuration. Only
//     get_images_and_details also calls /cfs/v2/sessions internally.
//   - BOS templates and BSS boot parameters are always v2.
//   - CFS v2 responses are flat JSON arrays (no pagination envelope).
// ---------------------------------------------------------------------------

async fn mock_hsm_groups(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/groups"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {
        "label": "compute",
        "members": { "ids": ["x3000c0s1b0n0"] }
      }
    ])))
    .mount(srv)
    .await;
}

// CFS v2 configurations — flat array, camelCase field names.
// Config name contains "compute" so it passes the name-contains-hsm-group
// filter inside cfs::configuration::utils::filter.
async fn mock_cfs_v2_configurations(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/cfs/v2/configurations"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {
        "name": "compute-config",
        "lastUpdated": "2024-01-01T00:00:00",
        "layers": [
          {
            "name": "layer1",
            "cloneUrl": "https://vcs.example.com/vcs/cray/cfg.git",
            "playbook": "site.yml"
          }
        ]
      }
    ])))
    .mount(srv)
    .await;
}

// CFS v2 sessions — flat array.
// target.groups[0].name = "compute" so the session survives
// cfs::session::utils::filter (which checks target HSM membership).
// Required for get_sessions (filter errors on empty result) and
// used as a side input by get_configurations and get_images.
async fn mock_cfs_v2_sessions(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/cfs/v2/sessions"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {
        "name": "my-session",
        "debug_on_failure": false,
        "target": {
          "definition": "dynamic",
          "groups": [{ "name": "compute", "members": [] }],
          "image_map": []
        }
      }
    ])))
    .mount(srv)
    .await;
}

async fn mock_bos_v2_templates(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/bos/v2/sessiontemplates"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {
        "name": "my-template",
        "enable_cfs": true,
        "boot_sets": {
          "compute": {
            "node_groups": ["compute"],
            "path": "s3://boot-images/abc123/manifest.json"
          }
        }
      }
    ])))
    .mount(srv)
    .await;
}

// CFS v2 components — flat array; empty is valid since it only contributes
// desired_config names to the configuration filter.
async fn mock_cfs_v2_components(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/cfs/v2/components"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .mount(srv)
    .await;
}

async fn mock_bss_bootparameters(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/bss/boot/v1/bootparameters"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {
        "hosts": ["x3000c0s1b0n0"],
        "params": "quiet",
        "kernel": "s3://boot-images/abc123/kernel",
        "initrd": "s3://boot-images/abc123/initrd"
      }
    ])))
    .mount(srv)
    .await;
}

async fn mock_ims_images(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/ims/v3/images"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {
        "id": "abc123",
        "name": "compute-my-image",
        "created": "2024-01-01T00:00:00"
      }
    ])))
    .mount(srv)
    .await;
}

// Used by get_boot_parameters via resolve_hosts_expression →
// get_node_metadata_available → get_all_nodes.
// Component ID must match the HSM group member so the xname passes
// the availability filter inside get_node_metadata_available.
async fn mock_hsm_components(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/State/Components"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "Components": [
        {
          "ID": "x3000c0s1b0n0",
          "Type": "Node",
          "State": "Ready",
          "Enabled": true,
          "Role": "Compute",
          "NID": 1,
          "NetType": "Sling",
          "Arch": "X86",
          "Class": "Mountain"
        }
      ]
    })))
    .mount(srv)
    .await;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// GET /api/v1/groups
//
// Call chain:
//   service::group::get_groups
//     → get_groups_names_available (pa_admin → GET /smd/hsm/v2/groups)
//     → backend.get_groups          (GET /smd/hsm/v2/groups?group=compute)
#[tokio::test]
async fn get_groups_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;

  let resp = fx.send(fx.auth_get("/api/v1/groups")).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["label"], "compute");
  assert_eq!(arr[0]["members"]["ids"][0], "x3000c0s1b0n0");
}

// GET /api/v1/configurations
//
// Call chain (csm-rs uses CFS v2 for configurations):
//   service::configuration::get_configurations
//     → get_groups_names_available           (GET /smd/hsm/v2/groups)
//     → backend.get_and_filter_configuration
//         → get_member_vec_from_hsm_name_vec  (GET /smd/hsm/v2/groups?group=compute)
//         → try_join!:
//             GET /cfs/v2/configurations
//             GET /cfs/v2/sessions
//             GET /bos/v2/sessiontemplates
//             GET /cfs/v2/components
//
// The configuration name must contain the HSM group name ("compute") to
// survive cfs::configuration::utils::filter.
// CFS v2 config response is a flat array with camelCase field names.
#[tokio::test]
async fn get_configurations_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_cfs_v2_configurations(&fx.mock_server).await;
  mock_cfs_v2_sessions(&fx.mock_server).await;
  mock_bos_v2_templates(&fx.mock_server).await;
  mock_cfs_v2_components(&fx.mock_server).await;

  let resp = fx.send(fx.auth_get("/api/v1/configurations")).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["name"], "compute-config");
  assert!(arr[0]["layers"].is_array());
}

// GET /api/v1/sessions
//
// Call chain (csm-rs uses CFS v2 for sessions):
//   service::session::get_sessions
//     → backend.get_and_filter_sessions(hsm_group_name_vec=[], xname_vec=[])
//         → hsm::group::utils::get_group_available (GET /smd/hsm/v2/groups)
//         → cfs::session::get_and_sort             (GET /cfs/v2/sessions)
//         → cfs::session::utils::filter
//
// The session mock includes target.groups[0].name = "compute" so it
// survives the filter (which retains only sessions targeting available
// groups). An empty result after filtering returns Err("No CFS session found").
// Neither process::exit branch fires because both filter vecs are empty.
#[tokio::test]
async fn get_sessions_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_cfs_v2_sessions(&fx.mock_server).await;

  let resp = fx.send(fx.auth_get("/api/v1/sessions")).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["name"], "my-session");
}

// GET /api/v1/templates
//
// Call chain:
//   service::template::get_templates
//     → get_groups_names_available (GET /smd/hsm/v2/groups)
//     → backend.get_member_vec_from_group_name_vec
//         → GET /smd/hsm/v2/groups?group=compute  (same stub)
//     → backend.get_and_filter_templates
//         → GET /bos/v2/sessiontemplates
#[tokio::test]
async fn get_templates_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_bos_v2_templates(&fx.mock_server).await;

  let resp = fx.send(fx.auth_get("/api/v1/templates")).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert!(!arr.is_empty());
  assert_eq!(arr[0]["name"], "my-template");
}

// GET /api/v1/images
//
// Handler returns Vec<ImageEntry> — a mapped struct (not raw Image):
//   { image: Value, configuration_name: String, image_id: String, is_linked: bool }
//
// Call chain (all internal CFS calls also use v2):
//   service::image::get_images
//     → get_groups_names_available (GET /smd/hsm/v2/groups)
//     → backend.get_images_and_details
//         → GET /ims/v3/images
//         → get_member_vec_from_hsm_name_vec (GET /smd/hsm/v2/groups?group=compute, x2)
//         → GET /bos/v2/sessiontemplates
//         → cfs::session::get_and_sort       (GET /cfs/v2/sessions, await? — must succeed)
//         → GET /bss/boot/v1/bootparameters  (unwrap_or_default — safe to leave empty)
#[tokio::test]
async fn get_images_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_ims_images(&fx.mock_server).await;
  mock_bos_v2_templates(&fx.mock_server).await;
  mock_cfs_v2_sessions(&fx.mock_server).await;
  mock_bss_bootparameters(&fx.mock_server).await;

  let resp = fx.send(fx.auth_get("/api/v1/images")).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert!(!arr.is_empty());
  assert!(arr[0]["image_id"].is_string());
  assert!(arr[0]["configuration_name"].is_string());
  assert!(arr[0]["is_linked"].is_boolean());
}

// GET /api/v1/boot-parameters?hsm_group=compute
//
// Call chain:
//   service::boot_parameters::get_boot_parameters
//     → common::node_ops::resolve_target_nodes(hsm_group=Some("compute"))
//         → get_groups_names_available            (GET /smd/hsm/v2/groups)
//         → backend.get_member_vec_from_group_name_vec
//                                                 (GET /smd/hsm/v2/groups?group=compute)
//         → resolve_hosts_expression("x3000c0s1b0n0")
//             → backend.get_node_metadata_available
//                 → backend.get_group_available    (GET /smd/hsm/v2/groups)
//                 → backend.get_all_nodes          (GET /smd/hsm/v2/State/Components)
//     → backend.get_bootparameters(xnames)        (GET /bss/boot/v1/bootparameters)
//
// The Component ID must match "x3000c0s1b0n0" so the xname survives
// the availability filter inside get_node_metadata_available.
#[tokio::test]
async fn get_boot_parameters_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_hsm_components(&fx.mock_server).await;
  mock_bss_bootparameters(&fx.mock_server).await;

  let resp = fx
    .send(fx.auth_get("/api/v1/boot-parameters?hsm_group=compute"))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["hosts"][0], "x3000c0s1b0n0");
  assert_eq!(arr[0]["kernel"], "s3://boot-images/abc123/kernel");
}

// ---------------------------------------------------------------------------
// Additional stub helpers
// ---------------------------------------------------------------------------

async fn mock_redfish_endpoints(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/Inventory/RedfishEndpoints"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "RedfishEndpoints": [
        {
          "ID": "x3000c0s1b0",
          "Type": "NodeBMC",
          "Hostname": "x3000c0s1b0",
          "Domain": "",
          "FQDN": "x3000c0s1b0",
          "Enabled": true,
          "UUID": "abc-123",
          "User": "root",
          "Password": "***",
          "UseSSDP": false,
          "MACRequired": false,
          "RediscoverOnUpdate": true
        }
      ]
    })))
    .mount(srv)
    .await;
}

// CFS v3 components — used by prepare_session_deletion via get_cfs_components.
// Response envelope: {"components": [...]}.
async fn mock_cfs_v3_components(srv: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/cfs/v3/components"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({"components": []})))
    .mount(srv)
    .await;
}

// ---------------------------------------------------------------------------
// Phase A — Remaining GET happy paths
// ---------------------------------------------------------------------------

// GET /api/v1/kernel-parameters?hsm_group=compute
//
// Call chain mirrors get_boot_parameters: resolve_target_nodes
// (HSM groups + components) → get_bootparameters.
#[tokio::test]
async fn get_kernel_parameters_happy_path() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_hsm_components(&fx.mock_server).await;
  mock_bss_bootparameters(&fx.mock_server).await;

  let resp = fx
    .send(fx.auth_get("/api/v1/kernel-parameters?hsm_group=compute"))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let body = TestFixture::body_json(resp).await;
  let arr = body.as_array().expect("expected JSON array");
  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["hosts"][0], "x3000c0s1b0n0");
}

// GET /api/v1/redfish-endpoints
//
// Call chain:
//   service::redfish_endpoints::get_redfish_endpoints
//     → backend.get_redfish_endpoints
//         → GET /smd/hsm/v2/Inventory/RedfishEndpoints
//
// Response is RedfishEndpointArray serialised as {"RedfishEndpoints": [...]}.
#[tokio::test]
async fn get_redfish_endpoints_happy_path() {
  let fx = TestFixture::setup().await;
  mock_redfish_endpoints(&fx.mock_server).await;

  let resp = fx.send(fx.auth_get("/api/v1/redfish-endpoints")).await;
  let status = resp.status();
  if status != StatusCode::OK {
    let body = TestFixture::body_json(resp).await;
    panic!("Expected 200, got {}: {:?}", status, body);
  }

  let body = TestFixture::body_json(resp).await;
  let endpoints = body["RedfishEndpoints"]
    .as_array()
    .expect("expected RedfishEndpoints array");
  assert_eq!(endpoints.len(), 1);
  assert_eq!(endpoints[0]["ID"], "x3000c0s1b0");
}

// ---------------------------------------------------------------------------
// Phase B — Write endpoint happy paths
// ---------------------------------------------------------------------------

// POST /api/v1/groups
//
// Call chain:
//   service::group::create_group → backend.add_group
//     → POST /smd/hsm/v2/groups → 201
#[tokio::test]
async fn create_group_happy_path() {
  let fx = TestFixture::setup().await;
  Mock::given(method("POST"))
    .and(path("/smd/hsm/v2/groups"))
    .respond_with(
      ResponseTemplate::new(201)
        .set_body_json(json!({"label": "new-group"})),
    )
    .mount(&fx.mock_server)
    .await;

  let resp = fx
    .send(fx.auth_post_json(
      "/api/v1/groups",
      json!({"label": "new-group", "description": "integration test"}),
    ))
    .await;
  assert_eq!(resp.status(), StatusCode::CREATED);
}

// DELETE /api/v1/groups/compute?force=true
//
// force=true bypasses validate_group_deletion, going directly to
//   backend.delete_group → DELETE /smd/hsm/v2/groups/compute
#[tokio::test]
async fn delete_group_force_happy_path() {
  let fx = TestFixture::setup().await;
  Mock::given(method("DELETE"))
    .and(path("/smd/hsm/v2/groups/compute"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
    .mount(&fx.mock_server)
    .await;

  let resp = fx
    .send(fx.auth_delete("/api/v1/groups/compute?force=true"))
    .await;
  assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// DELETE /api/v1/nodes/x3000c0s1b0n0
//
// Call chain:
//   service::node::delete_node → backend.delete_node
//     → DELETE /hsm/v2/State/Components/x3000c0s1b0n0
//   NOTE: csm-rs omits the "smd/" prefix for this specific call.
#[tokio::test]
async fn delete_node_happy_path() {
  let fx = TestFixture::setup().await;
  Mock::given(method("DELETE"))
    .and(path("/hsm/v2/State/Components/x3000c0s1b0n0"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
    .mount(&fx.mock_server)
    .await;

  let resp = fx
    .send(fx.auth_delete("/api/v1/nodes/x3000c0s1b0n0"))
    .await;
  assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// DELETE /api/v1/groups/compute/members — dry_run=true
//
// When dry_run=true the handler returns 204 immediately without
// making any backend calls, so no mock stubs are needed.
#[tokio::test]
async fn delete_group_members_dry_run() {
  let fx = TestFixture::setup().await;

  let resp = fx
    .send(fx.auth_delete_json(
      "/api/v1/groups/compute/members",
      json!({"xnames": ["x3000c0s1b0n0"], "dry_run": true}),
    ))
    .await;
  assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// Phase C — Error paths
// ---------------------------------------------------------------------------

// GET /api/v1/groups — backend returns an error → handler returns 500.
//
// With no mock mounted wiremock returns 404 for the HSM groups call.
// csm-rs treats any non-2xx as Error::Message, which to_handler_error
// maps to 500.
#[tokio::test]
async fn get_groups_backend_error_returns_500() {
  let fx = TestFixture::setup().await;
  // No mock → wiremock default 404 → csm-rs Error::Message → 500.

  let resp = fx.send(fx.auth_get("/api/v1/groups")).await;
  assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// DELETE /api/v1/sessions/unknown-session — session not found → 404.
//
// Call chain:
//   service::session::prepare_session_deletion
//     → get_groups_names_available (GET /smd/hsm/v2/groups)
//     → try_join!:
//         get_group_available      (GET /smd/hsm/v2/groups)
//         get_and_filter_sessions  (GET /cfs/v2/sessions → "my-session")
//         get_cfs_components       (GET /cfs/v3/components → [])
//         get_all_bootparameters   (GET /bss/boot/v1/bootparameters)
//     → cfs_session_vec.find("unknown-session") → None
//       → Error::NotFound → 404
//
// The sessions mock returns "my-session" (not "unknown-session") so that
// get_and_filter_sessions succeeds with a non-empty list and the "not found"
// is detected at the find() step rather than the empty-list guard.
#[tokio::test]
async fn delete_session_unknown_session_returns_404() {
  let fx = TestFixture::setup().await;
  mock_hsm_groups(&fx.mock_server).await;
  mock_cfs_v2_sessions(&fx.mock_server).await;
  mock_cfs_v3_components(&fx.mock_server).await;
  mock_bss_bootparameters(&fx.mock_server).await;

  let resp = fx
    .send(fx.auth_delete("/api/v1/sessions/unknown-session"))
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
