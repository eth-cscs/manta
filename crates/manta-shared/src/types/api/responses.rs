//! Lightweight response shapes for handlers that return ad-hoc JSON
//! maps such as `{ "created": true }` or `{ "id": "..." }`.
//!
//! Defined here so the OpenAPI spec carries a real type instead of
//! the catch-all `serde_json::Value`. Each struct matches the literal
//! JSON the handler emits today via `serde_json::json!({ ... })`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response for endpoints that simply confirm a write happened.
///
/// Emitted by `POST /api/v1/redfish-endpoints`,
/// `POST /api/v1/boot-parameters`, and `POST /api/v1/groups`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatedResponse {
  /// Always `true` on success.
  pub created: bool,
}

/// Response for `POST /api/v1/nodes` — echoes the registered xname.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNodeResponse {
  /// Physical location ID (xname) of the registered node.
  pub id: String,
}

/// Response for `POST /api/v1/sessions` — names of the created CFS
/// session and its underlying configuration.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionResponse {
  /// Name of the created CFS session.
  pub session_name: String,
  /// Name of the CFS configuration the session was attached to.
  pub configuration_name: String,
}

/// Response for `POST /api/v1/ephemeral-env` — the freshly provisioned
/// ephemeral host.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EphemeralEnvResponse {
  /// Hostname of the ephemeral environment.
  pub hostname: String,
}

/// Response for endpoints that simply confirm a backup / restore /
/// long-running migrate operation finished. Emitted by
/// `POST /api/v1/migrate/backup` and `POST /api/v1/migrate/restore`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CompletedResponse {
  /// Always `true` on success.
  pub completed: bool,
}

/// One migration pair's result; mirrors
/// `manta_server::service::migrate::NodeMigrationResult` on the wire.
/// Embedded inside [`MigrateNodesResponse`] without importing the
/// server-side type into the shared crate.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MigrateNodesPairResult {
  /// HSM group that received the nodes.
  pub target_hsm_name: String,
  /// HSM group that the nodes were moved out of.
  pub parent_hsm_name: String,
  /// Final member list of the target group after migration.
  pub target_members: Vec<String>,
  /// Remaining member list of the parent group after migration.
  pub parent_members: Vec<String>,
}

/// Response for `POST /api/v1/migrate/nodes` — moved xnames plus a
/// per-(target,parent) result list. Dry-run uses the same shape so the
/// CLI consumes one type regardless of mode.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MigrateNodesResponse {
  /// Xnames moved (or that would have been moved, in dry-run).
  pub xnames: Vec<String>,
  /// Per (target, parent) pair migration result.
  pub results: Vec<MigrateNodesPairResult>,
}
