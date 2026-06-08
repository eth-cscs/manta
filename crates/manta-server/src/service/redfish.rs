//! Redfish-endpoint queries and CRUD operations.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::hsm::inventory::RedfishEndpointArray;

use crate::{
  server::common::app_context::InfraContext,
  service::authorization::validate_user_group_members_access,
};
pub use manta_shared::types::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

/// List Redfish endpoint registrations, applying any caller-supplied
/// filters (`id` / `fqdn` / `uuid` / `macaddr` / `ipaddress`).
///
/// When `params.id` is set, the caller's group access to that BMC
/// xname is validated first; broad listings (no `id`) skip the
/// per-xname check since the backend already scopes by token. Admin
/// tokens short-circuit either way.
pub async fn get_redfish_endpoints(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetRedfishEndpointsParams,
) -> Result<RedfishEndpointArray, Error> {
  tracing::info!("Get Redfish endpoints");

  let xname_vec = if let Some(xname) = params.id.as_deref() {
    vec![xname.to_string()]
  } else {
    vec![]
  };
  validate_user_group_members_access(infra, token, &xname_vec).await?;

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
