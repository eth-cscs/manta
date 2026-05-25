//! Ephemeral CFS environment provisioning — launches a temporary
//! container booted from an existing IMS image and returns its
//! hostname.

use csm_rs::ShastaClient;
use manta_backend_dispatcher::error::Error;

use crate::server::common::app_context::InfraContext;
use manta_shared::common::jwt_ops;

const EPHEMERAL_IMAGE_NAME: &str = "__ephemeral_image";

/// Create an ephemeral CFS environment and return the SSH hostname.
pub async fn exec(
  infra: &InfraContext<'_>,
  token: &str,
  image_id: &str,
) -> Result<String, Error> {
  let user_public_key_name =
    jwt_ops::get_preferred_username(token).map_err(|e| {
      Error::JwtMalformed(format!(
        "claim 'preferred_user' not found in JWT token: {e}"
      ))
    })?;

  tracing::info!("Looking for user '{}' public SSH key", user_public_key_name);

  let shasta = ShastaClient::new(
    infra.shasta_base_url,
    infra.shasta_root_cert.to_vec(),
    infra.socks5_proxy.map(|s| s.to_string()),
  )
  .map_err(|e| {
    Error::BadRequest(format!("Could not build Shasta HTTP client: {e}"))
  })?;

  let user_public_ssh_id_value = if let Ok(Some(user_public_ssh_value)) = shasta
    .ims_public_keys_v3_get_single(token, &user_public_key_name)
    .await
  {
    user_public_ssh_value["id"].clone()
  } else {
    return Err(Error::NotFound(format!(
      "User '{user_public_key_name}' does not have an SSH public key in Alps. \
       Please contact platform sys admins."
    )));
  };

  tracing::info!("SSH key found with ID {}", user_public_ssh_id_value);
  tracing::info!(
    "Creating ephemeral environment based on image ID {}",
    image_id
  );

  let resp_json = shasta
    .ims_job_post_customize(
      token,
      EPHEMERAL_IMAGE_NAME,
      image_id,
      user_public_ssh_id_value.as_str().ok_or_else(|| {
        Error::MissingField("SSH key ID is not a string".to_string())
      })?,
    )
    .await
    .map_err(|e| {
      Error::BadRequest(format!(
        "Could not create ephemeral environment based on image ID {image_id}: {e}"
      ))
    })?;

  let hostname = resp_json
    .pointer("/ssh_containers/0/connection_info/customer_access/host")
    .and_then(|v| v.as_str())
    .ok_or_else(|| {
      Error::MissingField(
        "Failed to get SSH container hostname from ephemeral env response"
          .to_string(),
      )
    })?
    .to_string();

  tracing::info!("Ephemeral environment created — SSH hostname: {}", hostname);

  Ok(hostname)
}
