//! Calls to the CSCS Keycloak's token endpoint that need no person
//! present: realm discovery and the client credentials grant of a
//! service account.
//!
//! Shared by `manta-cli` (CI runs with a service account; its refresh
//! and device-code flows reuse [`discover`], [`TokenResponse`] and
//! [`oauth_error_message`]) and `manta-cache-server` (which refreshes
//! its index as a service account). The interactive logins (browser,
//! device code) and the token cache stay in `manta-cli`: they talk to
//! the person at the terminal.
//!
//! Nothing here caches a token. A CSCS token may be sent only once to
//! an OpenCHAMI site (its Hydra accepts each `jti` once), so a caller
//! asks for a new one per request.

use std::time::Duration;

use serde::Deserialize;

use crate::common::error::MantaError;

/// Default issuer URL of the CSCS Keycloak realm (production).
pub const DEFAULT_OIDC_ISSUER_URL: &str =
  "https://auth.cscs.ch/auth/realms/cscs";

/// Environment variable holding a service account's client id.
pub const SA_CLIENT_ID_ENV_VAR: &str = "MANTA_SA_CLIENT_ID";

/// Environment variable holding a service account's client secret.
pub const SA_CLIENT_SECRET_ENV_VAR: &str = "MANTA_SA_CLIENT_SECRET";

/// Per-request timeout for every call to the Keycloak.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// The endpoints manta needs from the realm's discovery document.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Discovery {
  /// The realm's token endpoint.
  pub token_endpoint: String,
  /// The realm's device authorization endpoint, when it offers the
  /// device-code flow.
  pub device_authorization_endpoint: Option<String>,
}

/// A successful token-endpoint response.
///
/// No `Debug`: it would print the tokens in clear.
#[derive(Deserialize)]
pub struct TokenResponse {
  /// The access token.
  pub access_token: String,
  /// The refresh token, if the grant gives one.
  pub refresh_token: Option<String>,
  /// The ID token, if the grant gives one.
  pub id_token: Option<String>,
  /// Seconds until the access token expires, if given.
  pub expires_in: Option<u64>,
}

/// An OAuth error body (RFC 6749 §5.2).
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OAuthError {
  /// The error code, e.g. `invalid_grant`.
  pub error: String,
  /// A human-readable description, if given.
  pub error_description: Option<String>,
}

/// Best-effort human message out of an OAuth error body: its
/// description, else its code, else the body itself.
#[must_use]
pub fn oauth_error_message(body: &[u8]) -> String {
  serde_json::from_slice::<OAuthError>(body).map_or_else(
    |_| String::from_utf8_lossy(body).into_owned(),
    |e| e.error_description.unwrap_or(e.error),
  )
}

/// Fetch the discovery document of the realm at `issuer_url`
/// (`<issuer>/.well-known/openid-configuration`).
///
/// # Errors
///
/// The document cannot be fetched or parsed.
pub async fn discover(issuer_url: &str) -> Result<Discovery, MantaError> {
  let http = http_client()?;
  let url = format!(
    "{}/.well-known/openid-configuration",
    issuer_url.trim_end_matches('/')
  );
  let resp = http
    .get(&url)
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(|e| {
      MantaError::Other(format!(
        "failed to fetch OIDC discovery document '{url}': {e}"
      ))
    })?;
  resp.json().await.map_err(|e| {
    MantaError::Other(format!(
      "failed to parse the OIDC discovery document '{url}': {e}"
    ))
  })
}

/// A service account's client credentials: a confidential client of
/// the CSCS realm that gets its tokens with the client credentials
/// grant ([`Self::token`]).
#[derive(Clone)]
pub struct ServiceAccount {
  client_id: String,
  client_secret: String,
}

impl std::fmt::Debug for ServiceAccount {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ServiceAccount")
      .field("client_id", &self.client_id)
      .field("client_secret", &"<REDACTED>")
      .finish()
  }
}

impl ServiceAccount {
  /// A service account with these credentials.
  #[must_use]
  pub fn new(client_id: &str, client_secret: &str) -> Self {
    Self {
      client_id: client_id.to_string(),
      client_secret: client_secret.to_string(),
    }
  }

  /// The service account in [`SA_CLIENT_ID_ENV_VAR`] and
  /// [`SA_CLIENT_SECRET_ENV_VAR`], `None` when neither is set. A
  /// variable holding only whitespace counts as unset.
  ///
  /// # Errors
  ///
  /// Only one of the two variables is set.
  pub fn from_env() -> Result<Option<Self>, MantaError> {
    Self::from_values(
      std::env::var(SA_CLIENT_ID_ENV_VAR).ok(),
      std::env::var(SA_CLIENT_SECRET_ENV_VAR).ok(),
    )
  }

  /// [`Self::from_env`] with the variables' values injected.
  fn from_values(
    client_id: Option<String>,
    client_secret: Option<String>,
  ) -> Result<Option<Self>, MantaError> {
    let set = |v: Option<String>| v.filter(|v| !v.trim().is_empty());
    match (set(client_id), set(client_secret)) {
      (Some(client_id), Some(client_secret)) => Ok(Some(Self {
        client_id,
        client_secret,
      })),
      (None, None) => Ok(None),
      _ => Err(MantaError::Other(format!(
        "set both {SA_CLIENT_ID_ENV_VAR} and {SA_CLIENT_SECRET_ENV_VAR} \
         for a service account, or neither"
      ))),
    }
  }

  /// The account's client id.
  #[must_use]
  pub fn client_id(&self) -> &str {
    &self.client_id
  }

  /// A new access token from the realm at `issuer_url`, with the client
  /// credentials grant. Nothing is cached: there is no refresh token,
  /// so every token comes from the credentials.
  ///
  /// # Errors
  ///
  /// Discovery fails, or the Keycloak rejects the credentials.
  pub async fn token(&self, issuer_url: &str) -> Result<String, MantaError> {
    let discovery = discover(issuer_url).await?;
    let resp = http_client()?
      .post(&discovery.token_endpoint)
      .basic_auth(&self.client_id, Some(&self.client_secret))
      .form(&[("grant_type", "client_credentials")])
      .send()
      .await
      .map_err(|e| {
        MantaError::Other(format!(
          "failed to send client-credentials request: {e}"
        ))
      })?;
    let status = resp.status();
    let body = resp.bytes().await?;
    if !status.is_success() {
      return Err(MantaError::Other(format!(
        "service-account login rejected: {}",
        oauth_error_message(&body)
      )));
    }
    let token: TokenResponse = serde_json::from_slice(&body).map_err(|e| {
      MantaError::Other(format!(
        "failed to parse client-credentials response: {e}"
      ))
    })?;
    Ok(token.access_token)
  }
}

fn http_client() -> Result<reqwest::Client, MantaError> {
  reqwest::Client::builder()
    // OIDC endpoints never need redirects; following one could leak
    // a code or token to an unexpected host.
    .redirect(reqwest::redirect::Policy::none())
    .timeout(HTTP_TIMEOUT)
    .build()
    .map_err(|e| {
      MantaError::Other(format!(
        "failed to build the Keycloak HTTP client: {e}"
      ))
    })
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use wiremock::matchers::{body_string_contains, header, method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  async fn keycloak() -> MockServer {
    let keycloak = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/.well-known/openid-configuration"))
      .respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "token_endpoint": format!("{}/token", keycloak.uri()),
      })))
      .mount(&keycloak)
      .await;
    keycloak
  }

  #[tokio::test]
  async fn a_service_account_token_comes_from_the_client_credentials_grant() {
    let keycloak = keycloak().await;
    Mock::given(method("POST"))
      .and(path("/token"))
      // base64("ci-bot:s3cret")
      .and(header("authorization", "Basic Y2ktYm90OnMzY3JldA=="))
      .and(body_string_contains("grant_type=client_credentials"))
      .respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "access_token": "sa-token",
        "expires_in": 300,
      })))
      .expect(1)
      .mount(&keycloak)
      .await;

    let token = ServiceAccount::new("ci-bot", "s3cret")
      .token(&keycloak.uri())
      .await
      .unwrap();
    assert_eq!(token, "sa-token");
  }

  #[tokio::test]
  async fn refused_credentials_carry_the_keycloak_message() {
    let keycloak = keycloak().await;
    Mock::given(method("POST"))
      .and(path("/token"))
      .respond_with(ResponseTemplate::new(401).set_body_json(json!({
        "error": "unauthorized_client",
        "error_description": "Invalid client secret",
      })))
      .mount(&keycloak)
      .await;

    let err = ServiceAccount::new("ci-bot", "wrong")
      .token(&keycloak.uri())
      .await
      .unwrap_err()
      .to_string();
    assert!(err.contains("Invalid client secret"), "{err}");
  }

  #[test]
  fn service_account_needs_both_variables_or_neither() {
    let some = |v: &str| Some(v.to_string());
    let account = ServiceAccount::from_values(some("id"), some("secret"))
      .unwrap()
      .unwrap();
    assert_eq!(account.client_id(), "id");
    assert!(ServiceAccount::from_values(None, None).unwrap().is_none());
    assert!(
      ServiceAccount::from_values(some(" "), some(""))
        .unwrap()
        .is_none()
    );
    for (id, secret) in [(some("id"), None), (None, some("secret"))] {
      let err = ServiceAccount::from_values(id, secret)
        .unwrap_err()
        .to_string();
      assert!(err.contains(SA_CLIENT_ID_ENV_VAR), "{err}");
    }
  }

  #[test]
  fn debug_redacts_the_secret() {
    let rendered = format!("{:?}", ServiceAccount::new("id", "s3cret"));
    assert!(!rendered.contains("s3cret"), "{rendered}");
    assert!(rendered.contains("id"), "{rendered}");
  }

  #[test]
  fn negative_expires_in_is_rejected() {
    assert!(
      serde_json::from_str::<TokenResponse>(
        r#"{"access_token":"a","expires_in":-5}"#
      )
      .is_err()
    );
  }

  #[test]
  fn oauth_error_message_prefers_the_description() {
    assert_eq!(
      oauth_error_message(br#"{"error":"e","error_description":"d"}"#),
      "d"
    );
    assert_eq!(oauth_error_message(br#"{"error":"e"}"#), "e");
    assert_eq!(oauth_error_message(b"plain"), "plain");
  }
}
