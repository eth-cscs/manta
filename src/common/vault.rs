pub mod http_client {

  use anyhow::Context;
  use serde_json::{Value, json};

  /// Authenticate to Vault using a JWT token and return
  /// a Vault client token.
  pub async fn auth_oidc_jwt(
    vault_base_url: &str,
    shasta_token: &str,
    site_name: &str,
  ) -> Result<String, anyhow::Error> {
    // NOTE: role is hardcoded to manta, this is the role that is created in vault for the
    // jwt-manta auth method. This role is created by the vault admin and is used to
    // authenticate
    let role = "manta";

    // rest client create new cfs sessions
    let client = reqwest::Client::builder().build()?;

    let api_url =
      format!("{}/v1/auth/jwt-manta-{}/login", vault_base_url, site_name);

    log::debug!("Accessing/login to {}", api_url);

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
        anyhow::anyhow!("Vault auth response missing 'client_token' field")
      })?;
    Ok(client_token.to_string())
  }

  /// Fetch a secret from Vault's KV store at `secret_path`.
  pub async fn fetch_secret(
    vault_auth_token: &str,
    vault_base_url: &str,
    secret_path: &str,
  ) -> Result<Value, anyhow::Error> {
    // rest client create new cfs sessions
    let client = reqwest::Client::builder().build()?;

    let api_url = vault_base_url.to_owned() + secret_path;

    log::debug!("Vault url to fetch VCS secrets is '{}'", api_url);

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
  ) -> Result<String, anyhow::Error> {
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("manta/data/{}", site_name);

    let vault_secret = fetch_secret(
      &vault_token,
      vault_base_url,
      &format!("/v1/{}/vcs", vault_secret_path),
    )
    .await
    .context("Failed to fetch VCS secret from Vault")?;

    let vcs_token = vault_secret["data"]
      .get("token")
      .and_then(Value::as_str)
      .ok_or_else(|| {
      anyhow::anyhow!("Vault secret response missing 'token' field")
    })?;

    Ok(vcs_token.to_string())
  }

  /// Retrieve Kubernetes secrets (API URL, token, CA cert)
  /// from Vault.
  pub async fn fetch_shasta_k8s_secrets_from_vault(
    vault_base_url: &str,
    site_name: &str,
    shasta_token: &str,
  ) -> Result<Value, anyhow::Error> {
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("manta/data/{}", site_name);

    let secret = fetch_secret(
      &vault_token,
      vault_base_url,
      &format!("/v1/{}/k8s", vault_secret_path),
    )
    .await
    .context("Failed to fetch k8s secrets from Vault")?;

    Ok(secret["data"].clone())
  }
}
