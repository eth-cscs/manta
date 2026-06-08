//! Redfish-endpoint queries and CRUD operations.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::hsm::inventory::RedfishEndpointArray;

use crate::{
  server::common::{app_context::InfraContext, jwt_ops},
  service::authorization::validate_user_group_members_access,
};
pub use manta_shared::types::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

/// List Redfish endpoint registrations, applying any caller-supplied
/// filters (`id` / `fqdn` / `uuid` / `macaddr` / `ipaddress`).
///
/// Authorization rules:
/// - Admin tokens (carrying [`crate::service::authorization::PA_ADMIN`])
///   may list every endpoint, with or without filters.
/// - Non-admin callers MUST scope the request by `id`. The xname is
///   then validated against the caller's accessible groups; without
///   an `id`, the response could leak every BMC's identity and
///   credentials. The non-admin broad listing returns `BadRequest`.
pub async fn get_redfish_endpoints(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetRedfishEndpointsParams,
) -> Result<RedfishEndpointArray, Error> {
  tracing::info!("Get Redfish endpoints");

  if !jwt_ops::is_user_admin(token) {
    let Some(xname) = params.id.as_deref() else {
      return Err(Error::BadRequest(
        "Non-admin callers must scope a Redfish-endpoints query by `id`."
          .to_string(),
      ));
    };
    validate_user_group_members_access(infra, token, &[xname.to_string()])
      .await?;
  }

  infra.get_redfish_endpoints(token, params).await
}

/// Register a new Redfish endpoint with HSM.
///
/// The caller-supplied `UpdateRedfishEndpointParams` is converted to a
/// single-element `RedfishEndpointArray` inside the InfraContext
/// wrapper before reaching the backend.
pub async fn add_redfish_endpoint(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateRedfishEndpointParams,
) -> Result<(), Error> {
  tracing::info!("Add Redfish endpoint id={}", params.id);

  validate_user_group_members_access(
    infra,
    token,
    std::slice::from_ref(&params.id),
  )
  .await?;

  infra.add_redfish_endpoint(token, params).await
}

/// Update an existing Redfish endpoint's properties.
///
/// All fields on `UpdateRedfishEndpointParams` are written; partial
/// updates aren't supported by the backend contract.
pub async fn update_redfish_endpoint(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateRedfishEndpointParams,
) -> Result<(), Error> {
  tracing::info!("Update Redfish endpoint id={}", params.id);

  validate_user_group_members_access(
    infra,
    token,
    std::slice::from_ref(&params.id),
  )
  .await?;

  infra.update_redfish_endpoint(token, params).await
}

/// Delete a Redfish endpoint registration by id (BMC xname).
///
/// `NotFound` is surfaced by the backend when `id` does not match an
/// existing registration; the service forwards it unchanged.
pub async fn delete_redfish_endpoint(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  tracing::info!("Delete Redfish endpoint id={}", id);

  validate_user_group_members_access(infra, token, &[id.to_string()]).await?;

  infra.delete_redfish_endpoint(token, id).await
}
