//! Ephemeral CFS environment provisioning — launches a temporary IMS
//! customize container booted from an existing IMS image and returns
//! its SSH hostname.
//!
//! Unlike most service modules this one bypasses the backend
//! dispatcher and talks to CSM via [`csm_rs::ShastaClient`] directly:
//! the IMS `jobs.customize` API is CSM-specific and isn't exposed
//! through `manta-backend-dispatcher`. The handler is wired only on
//! the CSM site.
//!
//! Flow:
//!
//! 1. Read the JWT's `preferred_username` claim.
//! 2. Look the user's public SSH key up in IMS — registered keys are
//!    keyed by username on the IMS side.
//! 3. POST an IMS customize job referencing `image_id` and the key id.
//! 4. Extract the SSH hostname from the response and return it.

use csm_rs::ShastaClient;
use manta_backend_dispatcher::error::Error;

use crate::server::common::app_context::InfraContext;
use crate::server::common::jwt_ops;
use crate::wire_conv;

const EPHEMERAL_IMAGE_NAME: &str = "__ephemeral_image";

/// Launch an ephemeral IMS customize container against `image_id` and
/// return its SSH hostname.
///
/// The caller's preferred username is read from the JWT and used to
/// look up their registered SSH public key in IMS. If no key is
/// registered, returns `NotFound` with a message pointing the user
/// at platform admins. The hostname is plucked from the IMS response
/// at `/ssh_containers/0/connection_info/customer_access/host`; a
/// missing field is reported as `MissingField` rather than a generic
/// error so operators can tell schema drift from real failures.
///
/// # Errors
///
/// - [`Error::JwtMalformed`] (via `wire_conv::to_backend`) when the
///   JWT carries no `preferred_username`.
/// - [`Error::BadRequest`] if the Shasta HTTP client cannot be built
///   or the IMS customize job submission fails.
/// - [`Error::NotFound`] when the caller has no SSH public key
///   registered in IMS.
/// - [`Error::MissingField`] when the IMS response is missing the
///   server-generated key id or the SSH container host field.
pub async fn exec(
  infra: &InfraContext<'_>,
  token: &str,
  image_id: &str,
) -> Result<String, Error> {
  let user_public_key_name =
    jwt_ops::get_preferred_username(token).map_err(wire_conv::to_backend)?;

  tracing::info!("Looking for user '{}' public SSH key", user_public_key_name);

  let shasta = ShastaClient::new(
    infra.shasta_base_url,
    infra.shasta_root_cert.to_vec(),
    infra.socks5_proxy.map(|s| s.to_string()),
  )
  .map_err(|e| {
    Error::BadRequest(format!("Could not build Shasta HTTP client: {e}"))
  })?;

  let user_public_ssh_id = if let Ok(Some(user_public_ssh_key)) = shasta
    .ims_public_keys_v3_get_single(token, &user_public_key_name)
    .await
  {
    user_public_ssh_key.id.ok_or_else(|| {
      Error::MissingField(
        "IMS public-key response missing server-generated 'id'".to_string(),
      )
    })?
  } else {
    return Err(Error::NotFound(format!(
      "User '{user_public_key_name}' does not have an SSH public key in Alps. \
       Please contact platform sys admins."
    )));
  };

  tracing::info!("SSH key found with ID {}", user_public_ssh_id);
  tracing::info!(
    "Creating ephemeral environment based on image ID {}",
    image_id
  );

  let resp_json = shasta
    .ims_job_post_customize(
      token,
      EPHEMERAL_IMAGE_NAME,
      image_id,
      &user_public_ssh_id,
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
