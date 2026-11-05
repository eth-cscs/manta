//! Fixtures shared by the router-level tests.

use manta_server::config::TokenExchangeConfig;
use manta_server::dispatcher::StaticBackendDispatcher;
use wiremock::{
  Mock, MockServer, Request, ResponseTemplate,
  matchers::{any, method, path},
};

/// Any well-formed PEM certificate; the mock Hydra speaks plain HTTP.
const TEST_PEM: &[u8] = b"-----BEGIN CERTIFICATE-----\n\
MIIBhTCCASugAwIBAgIQIRi6zePL6mKjOipn+dNuaTAKBggqhkjOPQQDAjASMRAw\n\
DgYDVQQKEwdBY21lIENvMB4XDTE3MTAyMDE5NDMwNloXDTE4MTAyMDE5NDMwNlow\n\
EjEQMA4GA1UEChMHQWNtZSBDbzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABD0d\n\
7VNhbWvZLWPuj/RtHFjvtJBEwOkhbN/BnnE8rnZR8+sbwnc/KhCk3FhnpHZnQz7B\n\
5aETbbIgmuvewdjvSBSjYzBhMA4GA1UdDwEB/wQEAwICpDATBgNVHSUEDDAKBggr\n\
BgEFBQcDATAPBgNVHRMBAf8EBTADAQH/MCkGA1UdEQQiMCCCDmxvY2FsaG9zdDo1\n\
NDUzgg4xMjcuMC4wLjE6NTQ1MzAKBggqhkjOPQQDAgNIADBFAiEA2zpJEPQyz6/l\n\
Wf86aX6PepsntZv2GYlA5UpabfT2EZICICpJ5h/iI+i341gBmLiAFQOyTDT+/wQc\n\
6MF9+Yw1Yy0t\n\
-----END CERTIFICATE-----\n";

/// Path of Hydra's token endpoint, as ochami-rs derives it.
pub const HYDRA_TOKEN_PATH: &str = "/hydra/oauth2/token";

/// An OpenCHAMI dispatcher whose API and Hydra are one mock server. Hydra
/// returns the caller's token as the exchanged token, so requests get past
/// `RequestCtx`. The API refuses everything with `418`: not `404`, which
/// route tests read as a missing route, and not a 5xx, which ochami-rs retries.
pub async fn stub_ochami_backend() -> StaticBackendDispatcher {
  let server = MockServer::start().await;
  mount_echoing_hydra(&server).await;
  Mock::given(any())
    .respond_with(ResponseTemplate::new(418))
    .mount(&server)
    .await;
  let backend = ochami_backend(&server.uri());
  // The mock must outlive the router; the process exits soon enough.
  Box::leak(Box::new(server));
  backend
}

/// Make `server` a Hydra that returns the caller's token as the exchanged
/// token.
pub async fn mount_echoing_hydra(server: &MockServer) {
  Mock::given(method("POST"))
    .and(path(HYDRA_TOKEN_PATH))
    .respond_with(|request: &Request| {
      let assertion = url_form_value(&request.body, "assertion");
      ResponseTemplate::new(200)
        .set_body_json(serde_json::json!({ "access_token": assertion }))
    })
    .mount(server)
    .await;
}

/// An OpenCHAMI dispatcher for the API and Hydra at `base_url`.
pub fn ochami_backend(base_url: &str) -> StaticBackendDispatcher {
  let exchange = TokenExchangeConfig {
    client_id: "client".to_string(),
    client_secret: "secret".to_string(),
    subject_issuer: None,
    scope: Some("smd.read".to_string()),
  };
  StaticBackendDispatcher::new("ochami", base_url, TEST_PEM, Some(&exchange))
    .unwrap()
}

/// The value of `name` in a form body. JWTs and the test tokens contain
/// nothing that form encoding changes.
fn url_form_value(body: &[u8], name: &str) -> String {
  String::from_utf8_lossy(body)
    .split('&')
    .find_map(|pair| pair.strip_prefix(&format!("{name}=")))
    .unwrap_or_default()
    .to_string()
}
