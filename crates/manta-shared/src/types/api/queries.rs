//! Query-string parameter types for every `GET` and `DELETE`
//! endpoint whose query parameters are non-trivial.
//!
//! All structs derive `Deserialize` (server side), `Serialize` (CLI
//! side will mostly build via `QueryBuilder` rather than this trait,
//! but a few places use it), and `IntoParams` for the OpenAPI spec.

use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

/// Query parameters for `GET /api/v1/sessions`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct SessionQuery {
  /// HSM group whose sessions should be returned.
  pub hsm_group: Option<String>,
  /// Filter to sessions whose `ansible_limit` mentions any of these
  /// comma-separated xnames.
  pub xnames: Option<String>,
  /// Lower-bound session age expressed as a duration string
  /// (e.g. `"1h"`, `"2d"`).
  pub min_age: Option<String>,
  /// Upper-bound session age expressed as a duration string.
  pub max_age: Option<String>,
  /// Session type filter: `"image"` or `"runtime"`.
  pub session_type: Option<String>,
  /// Status filter: `"pending"`, `"running"`, or `"complete"`.
  pub status: Option<String>,
  /// Exact session name.
  pub name: Option<String>,
  /// Cap on the number of sessions returned (most recent first).
  pub limit: Option<u8>,
}

/// Query parameters for `GET /api/v1/sessions/{name}/logs`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct SessionLogsQuery {
  /// When true, prefix each log line with its timestamp.
  #[serde(default)]
  pub timestamps: bool,
}

/// Query parameters for `DELETE /api/v1/sessions/{name}`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeleteSessionQuery {
  /// When true, return deletion context without actually deleting
  /// (default: `false`).
  #[serde(default)]
  pub dry_run: bool,
}

/// Query parameters for `GET /api/v1/configurations`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct ConfigurationQuery {
  /// Exact configuration name to fetch.
  pub name: Option<String>,
  /// Glob pattern matched against configuration names.
  pub pattern: Option<String>,
  /// HSM group whose associated configurations should be returned.
  pub hsm_group: Option<String>,
  /// Cap on the number of configurations returned (most recent first).
  pub limit: Option<u8>,
}

/// Query parameters for `DELETE /api/v1/configurations`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeleteConfigurationsQuery {
  /// Glob pattern to match configuration names.
  pub pattern: Option<String>,
  /// ISO-8601 lower bound — only delete configurations created after
  /// this date.
  pub since: Option<String>,
  /// ISO-8601 upper bound — only delete configurations created before
  /// this date.
  pub until: Option<String>,
  /// When true, return deletion candidates without removing anything.
  #[serde(default)]
  pub dry_run: bool,
}

/// Query parameters for `GET /api/v1/groups/nodes` (the renamed
/// alias of the legacy `GET /api/v1/clusters`).
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct ClusterQuery {
  /// HSM group name to list nodes for. When omitted the response
  /// covers every group the bearer token can access.
  pub hsm_group: Option<String>,
  /// Optional power-status filter (e.g. `ON`, `OFF`, `READY`).
  pub status: Option<String>,
}

/// Query parameters for `GET /api/v1/groups`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct GroupQuery {
  /// Exact group name; returns all groups when `None`.
  pub name: Option<String>,
}

/// Query parameters for `DELETE /api/v1/groups/{label}`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeleteGroupQuery {
  /// Delete even if the group still has members (default: `false`).
  #[serde(default)]
  pub force: bool,
}

/// Query parameters for `GET /api/v1/groups/hardware` (the renamed
/// alias of the legacy `GET /api/v1/hardware-clusters`).
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct HardwareClusterQuery {
  /// HSM group name to inventory. When omitted the response covers
  /// every group the bearer token can access.
  pub hsm_group: Option<String>,
}

/// Query parameters for `GET /api/v1/hardware-nodes-list`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct HardwareNodesListQuery {
  /// Hosts expression (xnames, NIDs, or hostlist notation). The field
  /// name is retained for wire stability.
  pub xnames: String,
}

/// Query parameters for `GET /api/v1/templates`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct TemplateQuery {
  /// Exact template name.
  pub name: Option<String>,
  /// HSM group whose associated templates should be returned.
  pub hsm_group: Option<String>,
  /// Cap on the number of templates returned (most recent first).
  pub limit: Option<u8>,
}

/// Query parameters for `GET /api/v1/images`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct ImageQuery {
  /// Exact IMS image ID; returns just that image when set.
  pub id: Option<String>,
  /// Glob pattern matched against image name; applied server-side
  /// (`service::image::get_images`). Invalid glob returns 400.
  pub pattern: Option<String>,
  /// Cap on the number of images returned (most recent first).
  pub limit: Option<u8>,
}

/// Query parameters for `DELETE /api/v1/images`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct DeleteImagesQuery {
  /// Comma-separated list of IMS image IDs to delete.
  pub ids: String,
  /// When true, validate deletion eligibility without removing
  /// anything.
  #[serde(default)]
  pub dry_run: bool,
}

/// Query parameters for `GET /api/v1/boot-parameters`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct BootParametersQuery {
  /// HSM group whose members' boot parameters should be returned.
  pub hsm_group: Option<String>,
  /// Explicit comma-separated xnames; mutually exclusive with
  /// `hsm_group`.
  pub nodes: Option<String>,
}

/// Query parameters for `GET /api/v1/kernel-parameters`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct KernelParametersQuery {
  /// HSM group whose members' kernel parameters should be returned.
  pub hsm_group: Option<String>,
  /// Explicit comma-separated xnames; mutually exclusive with
  /// `hsm_group`.
  pub nodes: Option<String>,
}

/// Query parameters for `GET /api/v1/nodes`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct NodesQuery {
  /// Comma-separated xnames, NIDs, or hostlist expression
  /// (e.g. `x3000c0s1b0n[0-3]`).
  pub xname: String,
  /// Expand results to include nodes sharing the same blade.
  pub include_siblings: Option<bool>,
  /// Optional power-status filter (e.g. `ON`, `OFF`, `READY`).
  pub status: Option<String>,
}

/// Query parameters for `GET /api/v1/redfish-endpoints`.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct RedfishEndpointsQuery {
  /// Exact endpoint ID (BMC xname) filter.
  pub id: Option<String>,
  /// FQDN substring filter.
  pub fqdn: Option<String>,
  /// UUID exact-match filter.
  pub uuid: Option<String>,
  /// MAC-address exact-match filter (colon-separated hex).
  pub macaddr: Option<String>,
  /// IP-address exact-match filter (IPv4 or IPv6).
  pub ipaddress: Option<String>,
}

/// Query parameters for the WebSocket console endpoints
/// (`/nodes/{xname}/console`, `/sessions/{name}/console`).
#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct ConsoleQuery {
  /// Initial terminal width in columns (default `80`).
  #[serde(default = "default_cols")]
  pub cols: u16,
  /// Initial terminal height in rows (default `24`).
  #[serde(default = "default_rows")]
  pub rows: u16,
}

fn default_cols() -> u16 {
  80
}
fn default_rows() -> u16 {
  24
}
