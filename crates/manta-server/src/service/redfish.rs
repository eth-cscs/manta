//! Redfish-endpoint queries and CRUD operations.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::types::hsm::inventory::{
  RedfishEndpoint, RedfishEndpointArray,
};

use crate::{
  server::common::{app_context::InfraContext, jwt_ops},
  service::authorization::validate_user_group_members_access,
};
pub use manta_shared::types::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

/// Convert a `UpdateRedfishEndpointParams` (CLI/HTTP wire shape) into a
/// backend [`RedfishEndpoint`] suitable for `add_redfish_endpoint` /
/// `update_redfish_endpoint`. Pure mapping — no I/O.
pub(crate) fn params_to_redfish_endpoint(
  params: UpdateRedfishEndpointParams,
) -> RedfishEndpoint {
  RedfishEndpoint {
    id: params.id,
    name: params.name,
    hostname: params.hostname,
    domain: params.domain,
    fqdn: params.fqdn,
    enabled: Some(params.enabled),
    user: params.user,
    password: params.password,
    use_ssdp: Some(params.use_ssdp),
    mac_required: Some(params.mac_required),
    mac_addr: params.mac_addr,
    ip_address: params.ip_address,
    rediscover_on_update: Some(params.rediscover_on_update),
    template_id: params.template_id,
    r#type: None,
    uuid: None,
    discovery_info: None,
  }
}

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

  infra
    .backend
    .get_redfish_endpoints(
      token,
      params.id.as_deref(),
      params.fqdn.as_deref(),
      None,
      params.uuid.as_deref(),
      params.macaddr.as_deref(),
      params.ipaddress.as_deref(),
      None,
    )
    .await
}

/// Register a new Redfish endpoint with HSM.
///
/// The caller-supplied `UpdateRedfishEndpointParams` is converted to a
/// single-element `RedfishEndpointArray` before reaching the backend.
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

  let endpoint = params_to_redfish_endpoint(params);
  let array = RedfishEndpointArray {
    redfish_endpoints: Some(vec![endpoint]),
  };
  infra.backend.add_redfish_endpoint(token, &array).await
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

  let endpoint = params_to_redfish_endpoint(params);
  infra.backend.update_redfish_endpoint(token, &endpoint).await
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

  infra
    .backend
    .delete_redfish_endpoint(token, id)
    .await
    .map(|_| ())
}
