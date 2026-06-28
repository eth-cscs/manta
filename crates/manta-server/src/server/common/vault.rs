//! Vault client used by handlers that need backend-specific secrets
//! (Gitea token for `create_session`, Kubernetes credentials for the
//! console and log-streaming handlers).
//!
//! ## Token-fetch flow
//!
//! Three steps every time secrets are needed (no caching):
//!
//! 1. [`http_client::auth_oidc_jwt`] POSTs the caller's bearer token
//!    to `/v1/auth/jwt-manta-<site>/login` and pulls the
//!    `auth.client_token` out of the response.
//! 2. [`http_client::get_secret`] GETs an arbitrary path with the
//!    `X-Vault-Token` header set to the client token and unwraps the
//!    `.data` field.
//! 3. The caller (e.g. [`http_client::get_shasta_vcs_token`])
//!    composes the secret path from `site_name` and reads the
//!    specific field it needs.
//!
//! The flow is non-interactive — there is no human prompt at any
//! point. If a site lacks Vault entirely, [`crate::server::common::app_context::InfraContext::vault_base_url`]
//! is `None` and the calling handler returns 501 before ever entering
//! this module. The Vault client token is short-lived and not cached
//! across requests; every call re-runs `auth_oidc_jwt`.

/// Thin Vault HTTP client. Authenticates via OIDC/JWT against a
/// per-site `jwt-manta-<site>` role, then reads K/V v2 secrets under
/// `manta/data/<...>`.
pub mod http_client {

  use std::sync::LazyLock;

  use manta_backend_dispatcher::error::Error;
  use serde_json::{Value, json};

  /// Vault API version prefix.
  const VAULT_API_PREFIX: &str = "/v1";

  /// Vault KV secret path prefix for manta.
  const VAULT_SECRET_PATH_PREFIX: &str = "manta/data";

  /// Vault role name used for JWT authentication.
  const VAULT_ROLE: &str = "manta";

  /// Process-wide `reqwest::Client` reused for every Vault call.
  ///
  /// Each call previously built a fresh client (re-doing the TLS
  /// handshake + connection-pool setup) — every console attach, log
  /// stream, and SAT-file apply did two such builds. `LazyLock` lets
  /// us share one `Client` (which is itself an `Arc` internally, so
  /// it's cheap to share across handlers) and keep keep-alive working.
  ///
  /// `Client::builder().build()` only fails on invalid TLS / proxy
  /// configuration; with all defaults it can't, so the unwrap here is
  /// safe — see the reqwest::ClientBuilder source for the conditions.
  static VAULT_HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
      .build()
      .expect("default reqwest::ClientBuilder build cannot fail")
  });

  /// Authenticate to Vault using a JWT token and return
  /// a Vault client token.
  ///
  /// Posts the caller's CSM bearer token to
  /// `/v1/auth/jwt-manta-<site_name>/login` against the `manta` role
  /// configured at the per-site auth mount.
  ///
  /// # Errors
  ///
  /// - [`Error::NetError`] when the request itself fails or Vault
  ///   responds with a non-success status (via
  ///   `error_for_status()`).
  /// - [`Error::MissingField`] when the response body omits the
  ///   server-issued `auth.client_token` field.
  pub async fn auth_oidc_jwt(
    vault_base_url: &str,
    shasta_token: &str,
    site_name: &str,
  ) -> Result<String, Error> {
    let role = VAULT_ROLE;

    let api_url = format!(
      "{vault_base_url}{VAULT_API_PREFIX}/auth/jwt-manta-{site_name}/login"
    );

    tracing::debug!("Accessing/login to {}", api_url);

    let request_payload = json!({ "jwt": shasta_token, "role": role });

    let resp = VAULT_HTTP_CLIENT
      .post(api_url)
      .header("X-Vault-Request", "true")
      .json(&request_payload)
      .send()
      .await?
      .error_for_status()?;

    let resp_value = resp.json::<Value>().await?;
    let client_token = resp_value["auth"]
      .get("client_token")
      .and_then(Value::as_str)
      .ok_or_else(|| {
        Error::MissingField(
          "Vault auth response missing 'client_token' field".to_string(),
        )
      })?;
    Ok(client_token.to_string())
  }

  /// Get a secret from Vault's KV store at `secret_path`.
  ///
  /// `secret_path` is concatenated onto `vault_base_url` verbatim;
  /// callers are responsible for the leading `/v1/...` prefix. The
  /// returned `Value` is the `.data` field of the Vault response
  /// (the K/V envelope itself is stripped here).
  ///
  /// # Errors
  ///
  /// [`Error::NetError`] when the HTTP request fails or Vault
  /// responds with a non-success status.
  pub async fn get_secret(
    vault_auth_token: &str,
    vault_base_url: &str,
    secret_path: &str,
  ) -> Result<Value, Error> {
    let api_url = vault_base_url.to_owned() + secret_path;

    tracing::debug!("Vault url to fetch VCS secrets is '{}'", api_url);

    let resp = VAULT_HTTP_CLIENT
      .get(api_url)
      .header("X-Vault-Token", vault_auth_token)
      .send()
      .await?
      .error_for_status()?;

    let secret_value: Value = resp.json().await?;
    Ok(secret_value["data"].clone())
  }

  /// Retrieve the Gitea VCS token from Vault.
  ///
  /// Reads `manta/data/<site_name>/vcs.data.token` after exchanging
  /// the caller's CSM bearer for a Vault client token via
  /// [`auth_oidc_jwt`]. Used by `service::session::create_session`
  /// and SAT-file rendering paths that need to clone a CFS layer's
  /// Gitea repository under the caller's identity.
  ///
  /// # Errors
  ///
  /// Any error produced by [`auth_oidc_jwt`] or [`get_secret`], plus
  /// [`Error::MissingField`] when the K/V secret has no `token`
  /// field.
  pub async fn get_shasta_vcs_token(
    shasta_token: &str,
    vault_base_url: &str,
    site_name: &str,
  ) -> Result<String, Error> {
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("{VAULT_SECRET_PATH_PREFIX}/{site_name}");

    let vault_secret = get_secret(
      &vault_token,
      vault_base_url,
      &format!("{VAULT_API_PREFIX}/{vault_secret_path}/vcs"),
    )
    .await?;

    let vcs_token = vault_secret["data"]
      .get("token")
      .and_then(Value::as_str)
      .ok_or_else(|| {
      Error::MissingField(
        "Vault secret response missing 'token' field".to_string(),
      )
    })?;

    Ok(vcs_token.to_string())
  }
}
