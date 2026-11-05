//! The token sent on each request: cached, refreshed near expiry, or the
//! one the client was built with when neither is available.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;
use wiremock::matchers::{bearer_token, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::{MantaClient, OpenApiResultExt};
use crate::common::authentication::TokenRefresher;
use crate::common::oidc::{OidcSettings, TokenStore, now_secs};

const SITE: &str = "alps";
const GROUPS_PATH: &str = "/v2/groups/available";

fn write_cache(path: &Path, access_token: &str, expires_at: i64) {
  TokenStore {
    access_token: access_token.to_string(),
    refresh_token: Some("refresh".to_string()),
    id_token: None,
    expires_at: Some(expires_at),
  }
  .save(path)
  .unwrap();
}

fn cache_in(dir: &TempDir) -> PathBuf {
  dir.path().join("token.json")
}

/// A Keycloak that answers the refresh grant `refreshes` times.
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
  Mock::given(method("POST"))
    .and(path("/token"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "access_token": "refreshed",
      "refresh_token": "refresh-2",
      "expires_in": 300,
    })))
    .expect(refreshes)
    .mount(&server)
    .await;
  server
}

/// A manta-server whose group listing accepts only `token`, `calls` times.
async fn server_accepting(token: &str, calls: u64) -> MockServer {
  let server = MockServer::start().await;
  mount_groups(&server, token, calls).await;
  server
}

async fn mount_groups(server: &MockServer, token: &str, calls: u64) {
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

#[tokio::test]
async fn a_valid_cached_token_is_sent_without_refreshing() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = keycloak(0).await;
  let server = server_accepting("cached", 1).await;
  let client = client(&server, "built-with", &keycloak.uri(), cache_in(&dir));

  list_groups(&client).await.unwrap();
}

/// The `manta power` failure: the token expires between two polls.
#[tokio::test]
async fn a_token_expiring_between_requests_is_refreshed() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 300);
  let keycloak = keycloak(1).await;
  let server = MockServer::start().await;
  mount_groups(&server, "cached", 1).await;
  mount_groups(&server, "refreshed", 1).await;
  let client = client(&server, "cached", &keycloak.uri(), cache_in(&dir));

  list_groups(&client).await.unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 5);
  list_groups(&client).await.unwrap();

  let saved = TokenStore::load(&cache_in(&dir)).unwrap();
  assert_eq!(saved.access_token, "refreshed");
  assert_eq!(saved.refresh_token.as_deref(), Some("refresh-2"));
}

#[tokio::test]
async fn parallel_requests_share_one_refresh() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 5);
  let keycloak = keycloak(1).await;
  let server = server_accepting("refreshed", 2).await;
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
  write_cache(&cache_in(&dir), "cached", now_secs() + 5);
  let keycloak = MockServer::start().await; // answers 404 to everything
  let server = server_accepting("built-with", 1).await;
  let client = client(&server, "built-with", &keycloak.uri(), cache_in(&dir));

  list_groups(&client).await.unwrap();
}

#[tokio::test]
async fn a_corrupt_cache_keeps_the_token_the_client_was_built_with() {
  let dir = tempfile::tempdir().unwrap();
  std::fs::write(cache_in(&dir), "not json").unwrap();
  let server = server_accepting("built-with", 1).await;
  let client =
    client(&server, "built-with", "http://127.0.0.1:9", cache_in(&dir));

  list_groups(&client).await.unwrap();
}

/// `MANTA_TOKEN`: no refresher, so the token goes out as is.
#[tokio::test]
async fn without_a_refresher_the_built_with_token_is_sent() {
  let server = server_accepting("from-env", 1).await;
  let client = MantaClient::new_with_timeout(
    &server.uri(),
    SITE,
    None,
    Some("from-env"),
    None,
  )
  .unwrap();

  list_groups(&client).await.unwrap();
}

/// Log streams bypass the generated client and its hook.
#[tokio::test]
async fn a_log_stream_connects_with_the_refreshed_token() {
  let dir = tempfile::tempdir().unwrap();
  write_cache(&cache_in(&dir), "cached", now_secs() + 5);
  let keycloak = keycloak(1).await;
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/v2/sessions/s1/logs"))
    .and(bearer_token("refreshed"))
    .respond_with(ResponseTemplate::new(200).set_body_string("log line\n"))
    .expect(1)
    .mount(&server)
    .await;
  let client = client(&server, "cached", &keycloak.uri(), cache_in(&dir));

  assert!(client.stream_session_logs("s1", false).await.is_ok());
}
