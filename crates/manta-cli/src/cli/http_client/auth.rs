//! `/auth/*` endpoints — no bearer-token requirement.
//!
//! These are the only HTTP calls the CLI makes without an
//! `Authorization: Bearer` header. The site name is still sent via
//! `X-Manta-Site` so the server knows which backend (CSM / OCHAMI)
//! and which Keycloak realm to talk to.

use std::time::Instant;

use anyhow::Context;

use super::MantaClient;

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
    use manta_shared::shared::auth::{AuthTokenRequest, AuthTokenResponse};
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
      .context("HTTP POST /auth/token failed")?;
    tracing::debug!(
      status = %resp.status(),
      elapsed_ms = started.elapsed().as_millis() as u64,
      "/auth/token response"
    );
    let body: AuthTokenResponse = Self::parse_json(resp).await?;
    Ok(body.token)
  }

  /// `POST /api/v1/auth/validate` — check whether the backend still
  /// accepts `token`. Returns `Ok(())` on 200, an error otherwise.
  pub async fn validate_token(&self, token: &str) -> anyhow::Result<()> {
    use manta_shared::shared::auth::ValidateTokenRequest;
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
      .context("HTTP POST /auth/validate failed")?;
    tracing::debug!(
      status = %resp.status(),
      elapsed_ms = started.elapsed().as_millis() as u64,
      "/auth/validate response"
    );
    Self::parse_no_content(resp).await
  }
}
