//! The token sent on each request: always a new one, from the cached
//! refresh token, a login when the Keycloak refuses it, or the service
//! account's credentials.

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
use crate::common::authentication::{ServiceAccount, TokenProvider};
use crate::common::oidc::{OidcSettings, TokenStore, now_secs};

const SITE: &str = "alps";
const GROUPS_PATH: &str = "/v2/groups/available";

fn write_cache(path: &Path) {
  TokenStore {
    access_token: "cached".to_string(),
    refresh_token: Some("refresh-0".to_string()),
    id_token: None,
    expires_at: Some(now_secs() + 300),
  }
  .save(path)
  .unwrap();
}

fn cache_in(dir: &TempDir) -> PathBuf {
  dir.path().join("token.json")
}

/// Mount the discovery document on `server`, with the device endpoint.
async fn discovery(server: &MockServer) {
  Mock::given(method("GET"))
    .and(path("/.well-known/openid-configuration"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "authorization_endpoint": format!("{}/auth", server.uri()),
      "token_endpoint": format!("{}/token", server.uri()),
      "device_authorization_endpoint": format!("{}/device", server.uri()),
    })))
    .mount(server)
    .await;
}

/// A Keycloak that answers the refresh grant `refreshes` times, issuing
/// `token-1`, `token-2`, … and rotating the refresh token alongside.
async fn keycloak(refreshes: u64) -> MockServer {
  let server = MockServer::start().await;
  discovery(&server).await;
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

/// A Keycloak that, like one rotating refresh tokens, accepts only the
/// refresh token it issued last, and answers slowly enough for two
/// unsynchronised refreshes to overlap.
async fn rotating_keycloak() -> MockServer {
  let server = MockServer::start().await;
  discovery(&server).await;
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

/// A Keycloak that refuses every refresh token (the session ended) and
/// completes a device-code login, issuing `login-token`. `logins` is how
/// many device codes it expects to hand out.
async fn session_ended_keycloak(logins: u64) -> MockServer {
  let server = MockServer::start().await;
  discovery(&server).await;
  Mock::given(method("POST"))
    .and(path("/token"))
    .and(body_string_contains("grant_type=refresh_token"))
    .respond_with(ResponseTemplate::new(400).set_body_json(json!({
      "error": "invalid_grant",
      "error_description": "Session not active",
    })))
    .mount(&server)
    .await;
  Mock::given(method("POST"))
    .and(path("/device"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "device_code": "dev-code",
      "user_code": "ABCD-EFGH",
      "verification_uri": "https://kc/device",
      "expires_in": 60,
      "interval": 0,
    })))
    .expect(logins)
    .mount(&server)
    .await;
  Mock::given(method("POST"))
    .and(path("/token"))
    .and(body_string_contains("device_code=dev-code"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "access_token": "login-token",
      "refresh_token": "login-refresh",
      "expires_in": 300,
    })))
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

/// Fail the test if `server` receives any request.
async fn expect_no_request(server: &MockServer) {
  Mock::given(wiremock::matchers::any())
    .respond_with(ResponseTemplate::new(500))
    .expect(0)
    .mount(server)
    .await;
}

fn provider(issuer: &str, cache: PathBuf, interactive: bool) -> TokenProvider {
  TokenProvider::from_cache(
    &OidcSettings {
      issuer_url: issuer,
      client_id: "manta-cli",
      headless: true,
    },
    cache,
    interactive,
  )
}

fn client(server: &MockServer, provider: TokenProvider) -> MantaClient {
  MantaClient::new_with_timeout(
    &server.uri(),
    SITE,
    None,
    Some(Arc::new(provider)),
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
  write_cache(&cache_in(&dir));
  let keycloak = keycloak(2).await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  accept(&server, "token-2", 1).await;
  let client =
    client(&server, provider(&keycloak.uri(), cache_in(&dir), false));

  list_groups(&client).await.unwrap();
  list_groups(&client).await.unwrap();

  assert_eq!(client.last_token().as_deref(), Some("token-2"));
  let saved = TokenStore::load(&cache_in(&dir)).unwrap();
  assert_eq!(saved.access_token, "token-2");
  assert_eq!(saved.refresh_token.as_deref(), Some("refresh-2"));
}

/// Parallel requests refresh one after another (the cache lock orders
/// them), so each uses the refresh token the previous one saved: the
/// rotating Keycloak would refuse the other.
#[tokio::test]
async fn parallel_requests_each_get_their_own_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  let keycloak = rotating_keycloak().await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  accept(&server, "token-2", 1).await;
  let client =
    client(&server, provider(&keycloak.uri(), cache_in(&dir), false));

  let (first, second) =
    tokio::join!(list_groups(&client), list_groups(&client));

  first.unwrap();
  second.unwrap();
}

/// Two `manta` runs share `token.json` but not an in-process lock, as two
/// providers here do. The file lock keeps them from spending the same
/// refresh token.
#[tokio::test]
async fn two_manta_runs_do_not_refresh_with_the_same_refresh_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  let keycloak = rotating_keycloak().await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  accept(&server, "token-2", 1).await;
  let first = client(&server, provider(&keycloak.uri(), cache_in(&dir), false));
  let second =
    client(&server, provider(&keycloak.uri(), cache_in(&dir), false));

  let (a, b) = tokio::join!(list_groups(&first), list_groups(&second));

  a.unwrap();
  b.unwrap();
}

/// The session ended mid-command: the request logs in again and goes out
/// with the new token, which is cached for the next one.
#[tokio::test]
async fn a_refused_refresh_token_leads_to_a_login() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  let keycloak = session_ended_keycloak(1).await;
  let server = MockServer::start().await;
  accept(&server, "login-token", 1).await;
  let client = client(&server, provider(&keycloak.uri(), cache_in(&dir), true));

  list_groups(&client).await.unwrap();

  let saved = TokenStore::load(&cache_in(&dir)).unwrap();
  assert_eq!(saved.refresh_token.as_deref(), Some("login-refresh"));
}

/// Parallel requests that all need a login get one login between them:
/// the others refresh with what it cached.
#[tokio::test]
async fn parallel_requests_share_one_login() {
  let dir = tempfile::tempdir().unwrap();
  let keycloak = MockServer::start().await;
  discovery(&keycloak).await;
  Mock::given(method("POST"))
    .and(path("/device"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "device_code": "dev-code",
      "user_code": "ABCD-EFGH",
      "verification_uri": "https://kc/device",
      "expires_in": 60,
      "interval": 0,
    })))
    .expect(1)
    .mount(&keycloak)
    .await;
  Mock::given(method("POST"))
    .and(path("/token"))
    .and(body_string_contains("device_code=dev-code"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "access_token": "login-token",
      "refresh_token": "login-refresh",
      "expires_in": 300,
    })))
    .mount(&keycloak)
    .await;
  Mock::given(method("POST"))
    .and(path("/token"))
    .and(body_string_contains("refresh_token=login-refresh"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "access_token": "refreshed-token",
      "expires_in": 300,
    })))
    .expect(1)
    .mount(&keycloak)
    .await;
  let server = MockServer::start().await;
  accept(&server, "login-token", 1).await;
  accept(&server, "refreshed-token", 1).await;
  let client = client(&server, provider(&keycloak.uri(), cache_in(&dir), true));

  let (first, second) =
    tokio::join!(list_groups(&client), list_groups(&client));

  first.unwrap();
  second.unwrap();
}

/// Without a terminal nobody could complete a login: the request is not
/// sent, and the error says how to log in.
#[tokio::test]
async fn without_a_login_and_a_terminal_the_request_fails() {
  let dir = tempfile::tempdir().unwrap();
  let server = MockServer::start().await;
  expect_no_request(&server).await;
  // Unroutable: no Keycloak call is expected either.
  let client = client(
    &server,
    provider("http://127.0.0.1:9", cache_in(&dir), false),
  );

  let err = list_groups(&client).await.unwrap_err().to_string();

  assert!(err.starts_with("not logged in"), "got: {err}");
  assert!(err.contains("MANTA_SA_CLIENT_ID"), "got: {err}");
}

#[tokio::test]
async fn a_refused_refresh_token_without_a_terminal_says_why() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  let keycloak = session_ended_keycloak(0).await;
  let server = MockServer::start().await;
  expect_no_request(&server).await;
  let client =
    client(&server, provider(&keycloak.uri(), cache_in(&dir), false));

  let err = list_groups(&client).await.unwrap_err().to_string();

  assert!(err.contains("Session not active"), "got: {err}");
  assert!(err.contains("stdin is not a terminal"), "got: {err}");
}

/// A Keycloak that is down is not fixed by a login: the request fails
/// without one, even at a terminal.
#[tokio::test]
async fn a_keycloak_outage_fails_the_request_without_a_login() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  let keycloak = MockServer::start().await;
  discovery(&keycloak).await;
  Mock::given(method("POST"))
    .and(path("/token"))
    .respond_with(ResponseTemplate::new(503).set_body_string("down"))
    .expect(1)
    .mount(&keycloak)
    .await;
  Mock::given(method("POST"))
    .and(path("/device"))
    .respond_with(ResponseTemplate::new(500))
    .expect(0)
    .mount(&keycloak)
    .await;
  let server = MockServer::start().await;
  expect_no_request(&server).await;
  let client = client(&server, provider(&keycloak.uri(), cache_in(&dir), true));

  let err = list_groups(&client).await.unwrap_err().to_string();

  assert!(err.contains("could not refresh the token"), "got: {err}");
  assert!(err.contains("down"), "got: {err}");
}

/// A cache that cannot be written loses nothing for this request: the
/// refreshed token is still sent.
#[tokio::test]
async fn a_cache_that_cannot_be_saved_still_gives_the_new_token() {
  use std::os::unix::fs::PermissionsExt;

  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  std::fs::write(cache_in(&dir).with_extension("lock"), "").unwrap();
  let read_only = std::fs::Permissions::from_mode(0o500);
  std::fs::set_permissions(dir.path(), read_only).unwrap();
  let keycloak = keycloak(1).await;
  let server = MockServer::start().await;
  accept(&server, "token-1", 1).await;
  let client =
    client(&server, provider(&keycloak.uri(), cache_in(&dir), false));

  let result = list_groups(&client).await;

  std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))
    .unwrap();
  result.unwrap();
  // The save did fail: the cache still holds what was there.
  let saved = TokenStore::load(&cache_in(&dir)).unwrap();
  assert_eq!(saved.access_token, "cached");
}

#[tokio::test]
async fn without_a_provider_no_token_is_sent() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path(GROUPS_PATH))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1)
    .mount(&server)
    .await;
  let client =
    MantaClient::new_with_timeout(&server.uri(), SITE, None, None).unwrap();

  list_groups(&client).await.unwrap();

  let requests = server.received_requests().await.unwrap();
  assert!(!requests[0].headers.contains_key("authorization"));
}

/// Log streams bypass the generated client and its hook.
#[tokio::test]
async fn a_log_stream_connects_with_a_new_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir));
  let keycloak = keycloak(1).await;
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/v2/sessions/s1/logs"))
    .and(bearer_token("token-1"))
    .respond_with(ResponseTemplate::new(200).set_body_string("log line\n"))
    .expect(1)
    .mount(&server)
    .await;
  let client =
    client(&server, provider(&keycloak.uri(), cache_in(&dir), false));

  assert!(client.stream_session_logs("s1", false).await.is_ok());
}

/// CI: no cache and no refresh token, a new token from the credentials
/// for every request.
#[tokio::test]
async fn a_service_account_gets_a_new_token_for_every_request() {
  let keycloak = MockServer::start().await;
  discovery(&keycloak).await;
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
  let provider = TokenProvider::for_service_account(
    &OidcSettings {
      issuer_url: &keycloak.uri(),
      client_id: "manta-cli",
      headless: true,
    },
    ServiceAccount::new("ci-bot", "s3cret"),
  );
  let client = client(&server, provider);

  list_groups(&client).await.unwrap();
  list_groups(&client).await.unwrap();
}
