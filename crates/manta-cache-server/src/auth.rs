//! The cache's own credential: a CSCS Keycloak service account.
//!
//! The cache refreshes by calling manta-server like any client, so it
//! needs a CSCS token that manta-server can exchange at each site.
//! [`ServiceAccountTokens`] gets a **new token for every request** with
//! the client credentials grant: an OpenCHAMI site's Hydra accepts each
//! token only once, and a CSCS access token lasts minutes, so a stored
//! token would not do. Nothing is cached. One token is valid for every
//! site, so one account serves the whole cache.
//!
//! The credentials come from `MANTA_SA_CLIENT_ID` and
//! `MANTA_SA_CLIENT_SECRET`, the same variables the CLI reads.

use manta_cache::{TokenError, TokenSource};
use manta_shared::common::oidc::ServiceAccount;

/// A service account at a CSCS Keycloak issuer, as a [`TokenSource`].
#[derive(Debug)]
pub struct ServiceAccountTokens {
  account: ServiceAccount,
  issuer_url: String,
}

impl ServiceAccountTokens {
  /// Tokens for `account` from the realm at `issuer_url`.
  pub fn new(account: ServiceAccount, issuer_url: &str) -> Self {
    Self {
      account,
      issuer_url: issuer_url.to_owned(),
    }
  }

  /// The account's client id, for the startup summary and `/dump`.
  pub fn client_id(&self) -> &str {
    self.account.client_id()
  }

  /// The Keycloak issuer the tokens come from.
  pub fn issuer_url(&self) -> &str {
    &self.issuer_url
  }
}

impl TokenSource for ServiceAccountTokens {
  async fn token(&self) -> Result<String, TokenError> {
    Ok(self.account.token(&self.issuer_url).await?)
  }
}

#[cfg(test)]
pub mod test_support {
  //! Service accounts for tests: one against a mock Keycloak, one that
  //! must never be asked for a token.

  use std::sync::atomic::{AtomicUsize, Ordering};

  use wiremock::matchers::{method, path};
  use wiremock::{Mock, MockServer, Request, ResponseTemplate};

  use super::*;

  /// Client id of the test service accounts.
  pub const CLIENT_ID: &str = "manta-cache-test";

  /// Client secret of the test service accounts; must never appear in
  /// any response.
  pub const CLIENT_SECRET: &str = "test-client-secret";

  /// A Keycloak that hands out `sa-token-1`, `sa-token-2`, … to the
  /// client credentials grant: a new token per call.
  pub async fn mock_keycloak() -> MockServer {
    let keycloak = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/.well-known/openid-configuration"))
      .respond_with(ResponseTemplate::new(200).set_body_json(
        serde_json::json!({ "token_endpoint": format!("{}/token", keycloak.uri()) }),
      ))
      .mount(&keycloak)
      .await;
    let issued = AtomicUsize::new(0);
    Mock::given(method("POST"))
      .and(path("/token"))
      .respond_with(move |_: &Request| {
        let n = issued.fetch_add(1, Ordering::SeqCst) + 1;
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
          "access_token": format!("sa-token-{n}"),
          "expires_in": 300,
        }))
      })
      .mount(&keycloak)
      .await;
    keycloak
  }

  /// The test account at `keycloak`.
  pub fn tokens_from(keycloak: &MockServer) -> ServiceAccountTokens {
    ServiceAccountTokens::new(
      ServiceAccount::new(CLIENT_ID, CLIENT_SECRET),
      &keycloak.uri(),
    )
  }

  /// The test account at an issuer nothing listens on, for tests that
  /// never refresh.
  pub fn unused_tokens() -> ServiceAccountTokens {
    ServiceAccountTokens::new(
      ServiceAccount::new(CLIENT_ID, CLIENT_SECRET),
      "http://127.0.0.1:9",
    )
  }
}
