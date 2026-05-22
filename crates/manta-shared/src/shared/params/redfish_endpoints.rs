//! Parameters for `GET`, `POST`, and `PUT` on `/redfish-endpoints`.

/// Typed parameters for fetching Redfish endpoints.
///
/// All fields are optional filters; setting none returns every
/// registered endpoint.
pub struct GetRedfishEndpointsParams {
  /// Exact endpoint ID (BMC xname).
  pub id: Option<String>,
  /// FQDN substring filter.
  pub fqdn: Option<String>,
  /// UUID exact match.
  pub uuid: Option<String>,
  /// MAC-address exact match (colon-separated hex).
  pub macaddr: Option<String>,
  /// IP-address exact match (IPv4 or IPv6).
  pub ipaddress: Option<String>,
}

/// Typed parameters for updating/adding a Redfish endpoint.
#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct UpdateRedfishEndpointParams {
  /// Xname identifying the BMC (e.g. `x3000c0s1b0`).
  pub id: String,
  /// Optional human-readable name.
  pub name: Option<String>,
  /// Hostname portion of the BMC FQDN.
  pub hostname: Option<String>,
  /// Domain portion of the BMC FQDN.
  pub domain: Option<String>,
  /// Full FQDN; overrides hostname+domain when set.
  pub fqdn: Option<String>,
  /// Whether the endpoint is enabled for discovery.
  pub enabled: bool,
  /// BMC username for Redfish authentication.
  pub user: Option<String>,
  /// BMC password for Redfish authentication.
  pub password: Option<String>,
  /// Use SSDP for automatic endpoint discovery.
  pub use_ssdp: bool,
  /// Whether a MAC address is required for geolocation.
  pub mac_required: bool,
  /// BMC MAC address (colon-separated).
  pub mac_addr: Option<String>,
  /// BMC IP address (IPv4 or IPv6).
  pub ip_address: Option<String>,
  /// Trigger a rediscovery pass when the endpoint is updated.
  pub rediscover_on_update: bool,
  /// ID of a discovery template to apply.
  pub template_id: Option<String>,
}
