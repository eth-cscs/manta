//! HTTP request/response bodies and CLI-built parameter structs for
//! the CFS session endpoints (`/api/v1/sessions`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/sessions`.
///
/// The CLI submits this when the user runs `manta run session`; the
/// server deserialises it in `handlers::session::create_session`.
/// `repo_names` and `repo_last_commit_ids` are parallel-indexed —
/// `repo_last_commit_ids[i]` is the commit SHA for `repo_names[i]`.
/// The two vectors must therefore have the same length.
///
/// Paired with [`super::responses::CreateSessionResponse`].
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
  /// Explicit name for the CFS session and configuration;
  /// auto-generated when absent.
  pub cfs_conf_sess_name: Option<String>,
  /// Ansible playbook filename inside the repository.
  pub playbook_yaml_file_name: Option<String>,
  /// Target HSM group name.
  pub hsm_group: Option<String>,
  /// Git repository names (parallel-indexed with
  /// `repo_last_commit_ids`).
  pub repo_names: Vec<String>,
  /// Git commit SHAs matching each entry in `repo_names`.
  pub repo_last_commit_ids: Vec<String>,
  /// Ansible `--limit` expression restricting which xnames are
  /// targeted (the service-layer authz check rejects group names —
  /// pre-resolve them client-side).
  pub ansible_limit: Option<String>,
  /// Ansible verbosity level (e.g. `"-v"`, `"-vvv"`).
  pub ansible_verbosity: Option<String>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<String>,
}

/// Typed parameters for fetching CFS sessions.
pub struct GetSessionParams {
  /// Group whose sessions should be returned.
  pub group: Option<String>,
  /// Filter to sessions whose `ansible_limit` mentions any of these
  /// xnames. Empty means "no xname filter".
  pub xnames: Vec<String>,
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
