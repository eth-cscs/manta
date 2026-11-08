//! The token sent on each request: always a new one from the cached
//! refresh token, or the one the client was built with when there is none.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::json;
use tempfile::TempDir;
use wiremock::matchers::{
  bearer_token, body_string_contains, header, method, path,
};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use super::{MantaClient, OpenApiResultExt};
use crate::common::authentication::{ServiceAccount, TokenRefresher};
use crate::common::oidc::{OidcSettings, TokenStore, now_secs};

const SITE: &str = "alps";
const GROUPS_PATH: &str = "/v2/groups/available";

fn write_cache(path: &Path, access_token: &str, expires_at: i64) {
  TokenStore {
    access_token: access_token.to_string(),
    refresh_token: Some("refresh-0".to_string()),
    id_token: None,
    expires_at: Some(expires_at),
  }
  .save(path)
  .unwrap();
}

fn cache_in(dir: &TempDir) -> PathBuf {
  dir.path().join("token.json")
}

/// A Keycloak that answers the refresh grant `refreshes` times, issuing
/// `token-1`, `token-2`, … and rotating the refresh token alongside.
async fn keycloak(refreshes: u64) -> MockServer {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/.well-known/openid-configuration"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "authorization_endpoint": format!("{}/auth", server.uri()),
      "token_endpoint": format!("{}/token", server.uri()),
    })))
    .mount(&server)
    .await;
  let issued = AtomicUsize::new(0);
  Mock::given(method("POST"))
    .and(path("/token"))
    .respond_with(move |_: &Request| {
      let n = issued.fetch_add(1, Ordering::SeqCst) + 1;
      ResponseTemplate::new(200).set_body_json(json!({
        "access_token": format!("token-{n}"),
        "refresh_token": format!("refresh-{n}"),
        "expires_in": 300,
      }))
    })
    .expect(refreshes)
    .mount(&server)
    .await;
  server
}

/// Make `server`'s group listing accept `token` exactly `calls` times.
async fn accept(server: &MockServer, token: &str, calls: u64) {
  Mock::given(method("GET"))
    .and(path(GROUPS_PATH))
    .and(bearer_token(token))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(calls)
    .mount(server)
    .await;
}

/// A client built with `built_with`, as at the start of a command.
fn client(
  server: &MockServer,
  built_with: &str,
  issuer: &str,
  cache: PathBuf,
) -> MantaClient {
  let refresher = TokenRefresher::new(
    &OidcSettings {
      issuer_url: issuer,
      client_id: "manta-cli",
      headless: true,
    },
    cache,
  );
  MantaClient::new_with_timeout(
    &server.uri(),
    SITE,
    None,
    Some(built_with),
    Some(Arc::new(refresher)),
  )
  .unwrap()
}

async fn list_groups(client: &MantaClient) -> anyhow::Result<Vec<String>> {
  client
    .openapi
    .get_available_groups(client.site_name())
    .await
    .into_anyhow()
    .await
}

/// Hydra accepts each token once, so even a valid cached token is not
/// sent: every request gets its own.
#[tokio::test]
async fn every_request_gets_a_new_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = keycloak(2).await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  accept(&server, "token-2", 1).await;
  let client = client(&server, "cached", &keycloak.uri(), cache_in(&dir));

  list_groups(&client).await.unwrap();
  list_groups(&client).await.unwrap();

  let saved = TokenStore::load(&cache_in(&dir)).unwrap();
  assert_eq!(saved.access_token, "token-2");
  assert_eq!(saved.refresh_token.as_deref(), Some("refresh-2"));
}

/// Parallel requests refresh one after another, so each uses the refresh
/// token the previous one saved, and each gets its own access token.
#[tokio::test]
async fn parallel_requests_each_get_their_own_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = keycloak(2).await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  accept(&server, "token-2", 1).await;
  let client = client(&server, "cached", &keycloak.uri(), cache_in(&dir));

  let (first, second) =
    tokio::join!(list_groups(&client), list_groups(&client));

  first.unwrap();
  second.unwrap();
}

/// Never a login prompt mid-command: the request goes out with what the
/// client has, and the server decides.
#[tokio::test]
async fn a_failed_refresh_keeps_the_token_the_client_was_built_with() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = MockServer::start().await; // answers 404 to everything
  let server = MockServer::start().await;
  accept(&server, "built-with", 1).await;
  let client = client(&server, "built-with", &keycloak.uri(), cache_in(&dir));

  list_groups(&client).await.unwrap();
}

#[tokio::test]
async fn a_corrupt_cache_keeps_the_token_the_client_was_built_with() {
  let dir = tempfile::tempdir().unwrap();
  std::fs::write(cache_in(&dir), "not json").unwrap();
  let server = MockServer::start().await;
  accept(&server, "built-with", 1).await;
  let client =
    client(&server, "built-with", "http://127.0.0.1:9", cache_in(&dir));

  list_groups(&client).await.unwrap();
}

#[tokio::test]
async fn without_a_refresher_the_built_with_token_is_sent() {
  let server = MockServer::start().await;
  accept(&server, "fixed", 1).await;
  let client = MantaClient::new_with_timeout(
    &server.uri(),
    SITE,
    None,
    Some("fixed"),
    None,
  )
  .unwrap();

  list_groups(&client).await.unwrap();
}

/// Log streams bypass the generated client and its hook.
#[tokio::test]
async fn a_log_stream_connects_with_a_new_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = keycloak(1).await;
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/v2/sessions/s1/logs"))
    .and(bearer_token("token-1"))
    .respond_with(ResponseTemplate::new(200).set_body_string("log line\n"))
    .expect(1)
    .mount(&server)
    .await;
  let client = client(&server, "cached", &keycloak.uri(), cache_in(&dir));

  assert!(client.stream_session_logs("s1", false).await.is_ok());
}

/// A Keycloak that, like one rotating refresh tokens, accepts only the
/// refresh token it issued last, and answers slowly enough for two
/// unsynchronised refreshes to overlap.
async fn rotating_keycloak() -> MockServer {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/.well-known/openid-configuration"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "authorization_endpoint": format!("{}/auth", server.uri()),
      "token_endpoint": format!("{}/token", server.uri()),
    })))
    .mount(&server)
    .await;
  let issued = std::sync::Mutex::new(0_usize);
  Mock::given(method("POST"))
    .and(path("/token"))
    .respond_with(move |request: &Request| {
      let body = String::from_utf8_lossy(&request.body).to_string();
      let mut last = issued.lock().unwrap();
      if !body.contains(&format!("refresh_token=refresh-{last}")) {
        return ResponseTemplate::new(400)
          .set_body_json(json!({ "error": "invalid_grant" }));
      }
      *last += 1;
      ResponseTemplate::new(200)
        .set_body_json(json!({
          "access_token": format!("token-{last}"),
          "refresh_token": format!("refresh-{last}"),
          "expires_in": 300,
        }))
        .set_delay(std::time::Duration::from_millis(100))
    })
    .mount(&server)
    .await;
  server
}

/// Two `manta` runs share `token.json` but not an in-process lock, as two
/// refreshers here do. The file lock keeps them from spending the same
/// refresh token.
#[tokio::test]
async fn two_manta_runs_do_not_refresh_with_the_same_refresh_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = rotating_keycloak().await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  accept(&server, "token-2", 1).await;
  let first = client(&server, "cached", &keycloak.uri(), cache_in(&dir));
  let second = client(&server, "cached", &keycloak.uri(), cache_in(&dir));

  let (a, b) = tokio::join!(list_groups(&first), list_groups(&second));

  a.unwrap();
  b.unwrap();
}

/// CI: no cache and no refresh token, a new token from the credentials
/// for every request.
#[tokio::test]
async fn a_service_account_gets_a_new_token_for_every_request() {
  let keycloak = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/.well-known/openid-configuration"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "authorization_endpoint": format!("{}/auth", keycloak.uri()),
      "token_endpoint": format!("{}/token", keycloak.uri()),
    })))
    .mount(&keycloak)
    .await;
  let issued = AtomicUsize::new(0);
  Mock::given(method("POST"))
    .and(path("/token"))
    // base64("ci-bot:s3cret")
    .and(header("authorization", "Basic Y2ktYm90OnMzY3JldA=="))
    .and(body_string_contains("grant_type=client_credentials"))
    .respond_with(move |_: &Request| {
      let n = issued.fetch_add(1, Ordering::SeqCst) + 1;
      ResponseTemplate::new(200).set_body_json(json!({
        "access_token": format!("sa-token-{n}"),
        "expires_in": 300,
      }))
    })
    .expect(2)
    .mount(&keycloak)
    .await;
  let server = MockServer::start().await;
  accept(&server, "sa-token-1", 1).await;
  accept(&server, "sa-token-2", 1).await;
  let refresher = TokenRefresher::for_service_account(
    &OidcSettings {
      issuer_url: &keycloak.uri(),
      client_id: "manta-cli",
      headless: true,
    },
    ServiceAccount::new("ci-bot", "s3cret"),
  );
  let client = MantaClient::new_with_timeout(
    &server.uri(),
    SITE,
    None,
    Some("first"),
    Some(Arc::new(refresher)),
  )
  .unwrap();

  list_groups(&client).await.unwrap();
  list_groups(&client).await.unwrap();
}
