use crate::common::authentication;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::{Context, Error, bail};
use csm_rs::ims;

/// Name used for ephemeral IMS images created during environment setup.
const EPHEMERAL_IMAGE_NAME: &str = "__ephemeral_image";

/// Create an ephemeral CFS environment for testing.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  image_id: &str,
) -> Result<(), Error> {
  let shasta_token = authentication::get_api_token(backend, site_name).await?;

  // Take user name and check if there is an SSH public key with that name already in Alps
  let user_public_key_name =
    csm_rs::common::jwt_ops::get_preferred_username(&shasta_token)
      .context("claim 'preferred_user' not found in JWT token")?;

  log::info!("Looking for user '{}' public SSH key", user_public_key_name);

  let user_public_ssh_id_value = if let Ok(Some(user_public_ssh_value)) =
    ims::public_keys::http_client::v3::get_single(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &user_public_key_name,
    )
    .await
  {
    user_public_ssh_value["id"].clone()
  } else {
    bail!(
      "User '{}' does not have an SSH public \
       key in Alps, Please contact platform \
       sys admins.",
      user_public_key_name
    );
  };

  log::info!("SSH key found with ID {}", user_public_ssh_id_value);

  // If public ssh key not found, then pompt user to provide public key
  // NOT YET. At this stage just throw an erro because the key was not found

  // Create IMS Job
  log::info!(
    "Creating ephemeral environment based on image ID {}",
    image_id
  );
  let resp_json = ims::job::http_client::post_customize(
    &shasta_token,
    shasta_base_url,
    shasta_root_cert,
    EPHEMERAL_IMAGE_NAME,
    image_id,
    user_public_ssh_id_value
      .as_str()
      .context("SSH key ID is not a string")?,
  )
  .await
  .with_context(|| {
    format!(
      "Could not create ephemeral environment \
       based on image ID {image_id}"
    )
  })?;

  let hostname_value = resp_json
    .pointer("/ssh_containers/0/connection_info/customer_access/host")
    .cloned()
    .context(
      "Failed to get SSH container hostname \
       from ephemeral env response",
    )?;

  log::info!(
    "Ephemeral environment successfully created! \
     hostname with ssh enabled: {}",
    hostname_value.as_str().unwrap_or("unknown")
  );
  println!("{}", hostname_value.as_str().unwrap_or("unknown"));

  Ok(())
}
