//! `/auth/*` endpoints — no bearer-token requirement.
//!
//! These are the only HTTP calls the CLI makes without an
//! `Authorization: Bearer` header. The site name is still sent via
//! `X-Manta-Site` so the server knows which backend (CSM / OCHAMI)
//! and which Keycloak realm to talk to.

use std::time::Instant;

use super::MantaClient;

/// Marker error attached as anyhow context whenever an `/auth/*`
/// HTTP call fails at the TCP/timeout layer (i.e., the manta server
/// — and therefore the auth path through it — is unreachable, not
/// "wrong credentials"). Lets [`crate::common::authentication`] tell
/// the two cases apart so an unreachable server short-circuits
/// instead of triggering the re-prompt loop.
#[derive(Debug)]
pub struct AuthServerUnreachable {
  /// The base manta server URL (without `/api/v1`) that was tried.
  /// Surfaced in the error message and recoverable by the loop via
  /// `downcast_ref` if a caller needs to log it separately.
  pub url: String,
}

impl std::fmt::Display for AuthServerUnreachable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "cannot reach manta server at {} for authentication. \
       Is the server running, and is `manta_server_url` in your \
       config correct?",
      self.url,
    )
  }
}

impl std::error::Error for AuthServerUnreachable {}

impl MantaClient {
  /// Map a `reqwest::Error` from an `/auth/*` send into an
  /// `anyhow::Error`. Connect-level / timeout failures get the typed
  /// [`AuthServerUnreachable`] as context so the interactive-login
  /// loop can detect "server unreachable" without string-matching.
  fn map_auth_send_error(
    &self,
    e: reqwest::Error,
    method_and_path: &str,
  ) -> anyhow::Error {
    if e.is_connect() || e.is_timeout() {
      let url = self.base_url().trim_end_matches("/api/v1").to_string();
      anyhow::Error::new(e).context(AuthServerUnreachable { url })
    } else {
      anyhow::Error::new(e).context(format!("{method_and_path} failed"))
    }
  }
}

impl MantaClient {
  /// `POST /api/v1/auth/token` — exchange Keycloak credentials for a CSM
  /// bearer token. The server proxies the request to the configured backend
  /// and returns its response; on failure the client never learns whether
  /// the username or password was wrong (server returns a generic 401).
  pub async fn get_token(
    &self,
    username: &str,
    password: &str,
  ) -> anyhow::Result<String> {
    use manta_shared::types::auth::{AuthTokenRequest, AuthTokenResponse};
    let url = format!("{}/auth/token", self.base_url());
    tracing::debug!(url = %url, site = %self.site_name(), "POST /auth/token");
    let builder = self
      .http_client()
      .post(&url)
      .header("X-Manta-Site", self.site_name())
      .json(&AuthTokenRequest {
        username: username.to_owned(),
        password: password.to_owned(),
      });
    Self::log_request_as_curl(&builder);
    let started = Instant::now();
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_auth_send_error(e, "HTTP POST /auth/token"))?;
    tracing::debug!(
      status = %resp.status(),
      elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
      "/auth/token response"
    );
    let body: AuthTokenResponse = Self::parse_json(resp).await?;
    Ok(body.token)
  }

  /// `POST /api/v1/auth/validate` — check whether the backend still
  /// accepts `token`. Returns `Ok(())` on 200, an error otherwise.
  pub async fn validate_token(&self, token: &str) -> anyhow::Result<()> {
    use manta_shared::types::auth::ValidateTokenRequest;
    let url = format!("{}/auth/validate", self.base_url());
    tracing::debug!(url = %url, site = %self.site_name(), "POST /auth/validate");
    let builder = self
      .http_client()
      .post(&url)
      .header("X-Manta-Site", self.site_name())
      .json(&ValidateTokenRequest {
        token: token.to_owned(),
      });
    Self::log_request_as_curl(&builder);
    let started = Instant::now();
    let resp = builder
      .send()
      .await
      .map_err(|e| self.map_auth_send_error(e, "HTTP POST /auth/validate"))?;
    tracing::debug!(
      status = %resp.status(),
      elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
      "/auth/validate response"
    );
    Self::parse_no_content(resp).await
  }
}
