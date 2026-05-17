pub mod http_client {

  use manta_backend_dispatcher::error::Error;
  use serde_json::{Value, json};

  /// Vault API version prefix.
  const VAULT_API_PREFIX: &str = "/v1";

  /// Vault KV secret path prefix for manta.
  const VAULT_SECRET_PATH_PREFIX: &str = "manta/data";

  /// Vault role name used for JWT authentication.
  const VAULT_ROLE: &str = "manta";

  /// Authenticate to Vault using a JWT token and return
  /// a Vault client token.
  pub async fn auth_oidc_jwt(
    vault_base_url: &str,
    shasta_token: &str,
    site_name: &str,
  ) -> Result<String, Error> {
    let role = VAULT_ROLE;

    let client = reqwest::Client::builder().build()?;

    let api_url =
      format!("{}{}/auth/jwt-manta-{}/login", vault_base_url, VAULT_API_PREFIX, site_name);

    tracing::debug!("Accessing/login to {}", api_url);

    let request_payload = json!({ "jwt": shasta_token, "role": role });

    let resp = client
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

  /// Fetch a secret from Vault's KV store at `secret_path`.
  pub async fn fetch_secret(
    vault_auth_token: &str,
    vault_base_url: &str,
    secret_path: &str,
  ) -> Result<Value, Error> {
    let client = reqwest::Client::builder().build()?;

    let api_url = vault_base_url.to_owned() + secret_path;

    tracing::debug!("Vault url to fetch VCS secrets is '{}'", api_url);

    let resp = client
      .get(api_url)
      .header("X-Vault-Token", vault_auth_token)
      .send()
      .await?
      .error_for_status()?;

    let secret_value: Value = resp.json().await?;
    Ok(secret_value["data"].clone())
  }

  /// Retrieve the Gitea VCS token from Vault.
  pub async fn fetch_shasta_vcs_token(
    shasta_token: &str,
    vault_base_url: &str,
    site_name: &str,
  ) -> Result<String, Error> {
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("{}/{}", VAULT_SECRET_PATH_PREFIX, site_name);

    let vault_secret = fetch_secret(
      &vault_token,
      vault_base_url,
      &format!("{}/{}/vcs", VAULT_API_PREFIX, vault_secret_path),
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

  /// Retrieve Kubernetes secrets (API URL, token, CA cert)
  /// from Vault.
  pub async fn fetch_shasta_k8s_secrets_from_vault(
    vault_base_url: &str,
    site_name: &str,
    shasta_token: &str,
  ) -> Result<Value, Error> {
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("{}/{}", VAULT_SECRET_PATH_PREFIX, site_name);

    let secret = fetch_secret(
      &vault_token,
      vault_base_url,
      &format!("{}/{}/k8s", VAULT_API_PREFIX, vault_secret_path),
    )
    .await?;

    Ok(secret["data"].clone())
  }
}
