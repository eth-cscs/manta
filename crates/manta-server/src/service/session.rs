//! CFS session queries, creation, deletion, and console-readiness
//! validation.
//!
//! Session deletion follows the plan/apply pattern shared with
//! [`crate::service::boot_parameters`] and
//! [`crate::service::configuration`]:
//! [`prepare_session_deletion`] collects everything the delete will
//! need (the session itself, image ids it produced, CFS components,
//! BSS boot parameters) without mutating state, and
//! [`execute_session_deletion`] applies the plan. The two-step shape
//! lets the CLI render a confirmation prompt with the full blast
//! radius before any backend write.
//!
//! [`validate_session_access`] and [`validate_console_session`] are
//! standalone pre-checks used by handlers that need to fail-fast
//! before doing anything else (typically a console attach).

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_session::ApplySessionTrait;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::component::Component;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::{
  validate_user_group_members_access, validate_user_group_vec_access,
};
use crate::service::node_ops;
pub use manta_shared::types::api::session::GetSessionParams;

/// List CFS sessions visible to the caller, applying every filter on
/// `params`.
///
/// The backend rejects mixing group and xname filters: an explicit
/// `params.xnames` list wins and the group set is left empty;
/// otherwise the request is scoped to `params.group` (single label)
/// or to every group the token already grants access to. Group
/// access and xname membership are validated before the backend
/// call so the response can never leak rows the caller couldn't
/// have listed directly.
pub async fn get_sessions(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetSessionParams,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  tracing::info!("Get CFS sessions");

  // The backend rejects requests that pass both group names and
  // xnames, so an explicit xname filter wins and skips the group
  // expansion. Otherwise, use the requested group or fall back to the
  // caller's accessible groups.
  let target_group_vec: Vec<String> = if !params.xnames.is_empty() {
    Vec::new()
  } else if let Some(group) = &params.group {
    vec![group.clone()]
  } else {
    infra
      .backend
      .get_group_available(token)
      .await?
      .iter()
      .map(|group| group.label.clone())
      .collect()
  };

  validate_user_group_vec_access(infra, token, &target_group_vec).await?;
  validate_user_group_members_access(infra, token, &params.xnames).await?;

  infra
    .backend
    .get_and_filter_sessions(
      token,
      target_group_vec,
      params.xnames.iter().map(|xname| xname.as_ref()).collect(),
      params.min_age.as_ref(),
      params.max_age.as_ref(),
      params.session_type.as_ref(),
      params.status.as_ref(),
      params.name.as_ref(),
      params.limit.as_ref(),
      None,
    )
    .await
}

/// Data needed to delete/cancel a session.
#[derive(serde::Serialize)]
pub struct SessionDeletionContext {
  /// The session to be deleted.
  pub session: CfsSessionGetResponse,
  /// IMS image IDs produced by this session (empty for non-image sessions).
  pub image_ids: Vec<String>,
  /// All HSM groups the token has access to (used for membership checks).
  pub group_available_vec: Vec<Group>,
  /// CFS component states (used to clear desired-config references).
  pub cfs_component_vec: Vec<Component>,
  /// BSS boot parameters (used to unset boot image refs pointing at session images).
  pub bss_bootparameters_vec: Vec<BootParameters>,
}

/// Collect everything a session-delete operation will need, without
/// mutating any state.
///
/// Validates group access first, then fans out four backend calls in
/// parallel (groups, sessions, CFS components, BSS boot parameters)
/// because each is independent and the latency dominates the
/// operation. Returns `NotFound` when the named session isn't in the
/// (group-scoped) result set. The image ids the session produced are
/// extracted up front so the apply step doesn't need to re-derive
/// them.
pub async fn prepare_session_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  session_name: &str,
  settings_group_name_opt: Option<&str>,
) -> Result<SessionDeletionContext, Error> {
  // One backend fetch + in-memory validation, replacing the prior
  // three round-trips. See `service::group::resolve_target_and_available_groups`.
  let (group_available_vec, target_group_vec) =
    crate::service::group::resolve_target_and_available_groups(
      infra,
      token,
      settings_group_name_opt,
    )
    .await?;

  tracing::info!("Fetching data from the backend...");
  let start = std::time::Instant::now();

  let (cfs_session_vec, cfs_component_vec, bss_bootparameters_vec) = tokio::try_join!(
    infra.backend.get_and_filter_sessions(
      token,
      target_group_vec,
      Vec::new(),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    ),
    infra.backend.get_cfs_components(token, None, None, None),
    infra.backend.get_all_bootparameters(token),
  )?;

  tracing::info!(
    "Time elapsed to fetch information from backend: {:?}",
    start.elapsed()
  );

  let session = cfs_session_vec
    .into_iter()
    .find(|s| s.name == session_name)
    .ok_or_else(|| Error::NotFound(format!("CFS session '{session_name}'")))?;

  let image_ids = session.get_result_id_vec();

  Ok(SessionDeletionContext {
    session,
    image_ids,
    group_available_vec,
    cfs_component_vec,
    bss_bootparameters_vec,
  })
}

/// Apply a session delete previously planned by
/// [`prepare_session_deletion`].
///
/// Delegates to the backend's combined delete/cancel routine, which
/// also rewrites CFS component desired-config refs and unsets BSS
/// boot-image refs that pointed at the session's images. With
/// `dry_run = true` the routine returns the would-be changes without
/// touching the backend.
pub async fn execute_session_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  deletion_ctx: &SessionDeletionContext,
  dry_run: bool,
) -> Result<(), Error> {
  infra
    .backend
    .delete_and_cancel_session(
      token,
      &deletion_ctx.group_available_vec,
      &deletion_ctx.session,
      &deletion_ctx.cfs_component_vec,
      &deletion_ctx.bss_bootparameters_vec,
      dry_run,
    )
    .await
}

/// Parameters for [`create_cfs_session`]. Bundled to keep the
/// service entry point readable at the call site (the handler-level
/// `CreateSessionRequest` body folds 1:1 into this).
pub struct CreateCfsSessionParams<'a> {
  /// Optional caller-supplied session name; backend autogenerates one
  /// when absent.
  pub cfs_conf_sess_name: Option<&'a str>,
  /// Optional playbook path inside the rendered configuration.
  pub playbook_yaml_file_name: Option<&'a str>,
  /// HSM group the session targets when no `ansible_limit` is given.
  pub group: Option<&'a str>,
  /// VCS repository names mirroring `repo_last_commit_ids`.
  pub repo_names: &'a [&'a str],
  /// Commit SHAs, one per `repo_names` entry.
  pub repo_last_commit_ids: &'a [&'a str],
  /// Hosts expression (xnames / NIDs / hostlist) limiting the session;
  /// resolved to xnames before the CFS request.
  pub ansible_limit: Option<&'a str>,
  /// Ansible verbosity flag (`-v` .. `-vvv`).
  pub ansible_verbosity: Option<&'a str>,
  /// Arbitrary args forwarded to `ansible-playbook`.
  pub ansible_passthrough: Option<&'a str>,
}

/// Create a CFS session, expanding the ansible-limit hosts expression
/// to xnames first.
///
/// `params.ansible_limit` is parsed as a hostlist / NID / xname
/// expression the same way other entry points do, then joined with
/// commas for the CFS request — CFS itself is happy with either form
/// but downstream tooling expects xnames. When `params.ansible_limit`
/// is `None`, the session targets the full group selected by
/// `params.group`. Returns
/// `(cfs_configuration_name, cfs_session_name)`.
pub async fn create_cfs_session(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  params: CreateCfsSessionParams<'_>,
) -> Result<(String, String), Error> {
  let ansible_limit = if let Some(ansible_limit) = params.ansible_limit {
    let xname_vec = node_ops::from_user_hosts_expression_to_xname_vec(
      infra,
      token,
      ansible_limit,
      false,
    )
    .await?;
    Some(xname_vec.join(","))
  } else {
    None
  };

  infra
    .backend
    .apply_session(
      gitea_token,
      infra.gitea_base_url,
      token,
      params.cfs_conf_sess_name,
      params.playbook_yaml_file_name,
      params.group,
      params.repo_names,
      params.repo_last_commit_ids,
      ansible_limit.as_deref(),
      params.ansible_verbosity,
      params.ansible_passthrough,
    )
    .await
}

/// Fetch a single CFS session by name.
///
/// Returns `NotFound` when no session with that name exists.
async fn fetch_session_by_name(
  infra: &InfraContext<'_>,
  token: &str,
  name: &str,
) -> Result<CfsSessionGetResponse, Error> {
  let sessions = infra
    .backend
    .get_and_filter_sessions(
      token,
      Vec::new(),
      Vec::new(),
      None,
      None,
      None,
      None,
      Some(&name.to_string()),
      None,
      None,
    )
    .await?;

  sessions
    .into_iter()
    .next()
    .ok_or_else(|| Error::NotFound(format!("CFS session '{name}'")))
}

/// Fetch a session by name and validate that the caller is allowed
/// to act on it.
///
/// Access is granted when every HSM group named in the session's
/// `target.groups` overlaps the caller's accessible groups (the union
/// returned by `InfraContext::get_group_name_available`). A session
/// that targets no HSM groups (e.g. a runtime session) is treated as
/// not gated by group access.
///
/// Returns the fetched session so the caller doesn't double-GET.
/// `NotFound` when the session doesn't exist; `BadRequest` when any
/// target group is outside the accessible set — matching the
/// access-denial shape used by
/// [`crate::service::authorization::validate_user_group_access`].
pub async fn validate_session_access(
  infra: &InfraContext<'_>,
  token: &str,
  session_name: &str,
) -> Result<CfsSessionGetResponse, Error> {
  let session = fetch_session_by_name(infra, token, session_name).await?;

  let target_groups = session.get_target_hsm().unwrap_or_default();
  if !target_groups.is_empty() {
    let accessible = infra.backend.get_group_name_available(token).await?;
    if let Some(unauthorized) =
      target_groups.iter().find(|g| !accessible.contains(g))
    {
      return Err(Error::BadRequest(format!(
        "Can't access CFS session '{session_name}': it targets HSM \
         group '{unauthorized}' which is not in your accessible set"
      )));
    }
  }

  Ok(session)
}

/// Reject sessions that didn't produce a result image.
///
/// `BadRequest` when the session has no `result_id` — callers
/// shouldn't try to PATCH a non-existent image. csm-rs's deeper check
/// inside `collect_and_stamp_image` remains as a defence-in-depth
/// safety net.
pub fn require_result_image(
  session: &CfsSessionGetResponse,
) -> Result<(), Error> {
  if session.get_first_result_id().is_none() {
    return Err(Error::BadRequest(format!(
      "CFS session '{}' produced no image (no result_id); refusing to stamp",
      session.name
    )));
  }
  Ok(())
}

/// Validate that a CFS session is suitable for attaching a console.
///
/// Returns `NotFound` if the session doesn't exist, `BadRequest` if the
/// session is not image-type or has missing internal state, and `Conflict`
/// if it is not running.
pub async fn validate_console_session(
  infra: &InfraContext<'_>,
  token: &str,
  name: &str,
) -> Result<(), Error> {
  let session = fetch_session_by_name(infra, token, name).await?;

  let target_def = session
    .target
    .as_ref()
    .and_then(|t| t.definition.as_ref())
    .ok_or_else(|| {
      Error::BadRequest(format!(
        "CFS session '{name}' has no target definition"
      ))
    })?;

  if target_def != "image" {
    return Err(Error::BadRequest(format!(
      "CFS session '{name}' is not an image-type session (got '{target_def}')"
    )));
  }

  let status = session
    .status
    .as_ref()
    .and_then(|s| s.session.as_ref())
    .and_then(|s| s.status.as_ref())
    .ok_or_else(|| {
      Error::BadRequest(format!("CFS session '{name}' has no status"))
    })?;

  if status != "running" {
    return Err(Error::Conflict(format!(
      "CFS session '{name}' is not running (status: '{status}')"
    )));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  //! Function-level tests for the boundary-check helpers. The
  //! `InfraContext`-touching helpers (`validate_session_access`,
  //! `get_sessions`, etc.) are exercised through integration tests
  //! against `router()` — see `crates/manta-server/tests/`.

  use super::{Error, require_result_image};
  use manta_backend_dispatcher::types::cfs::session::{
    Artifact, CfsSessionGetResponse, Status,
  };

  fn session_with_result_id(
    name: &str,
    result_id: Option<&str>,
  ) -> CfsSessionGetResponse {
    CfsSessionGetResponse {
      name: name.to_string(),
      configuration: None,
      ansible: None,
      target: None,
      status: Some(Status {
        artifacts: Some(vec![Artifact {
          image_id: None,
          result_id: result_id.map(str::to_string),
          r#type: None,
        }]),
        session: None,
      }),
      tags: None,
      debug_on_failure: false,
      logs: None,
    }
  }

  #[test]
  fn require_result_image_accepts_session_with_result_id() {
    let session = session_with_result_id("sat-img-v1", Some("ims-image-abc"));
    assert!(require_result_image(&session).is_ok());
  }

  #[test]
  fn require_result_image_rejects_session_without_result_id() {
    let session = session_with_result_id("sat-img-v1", None);
    let err = require_result_image(&session).unwrap_err();
    assert!(
      matches!(err, Error::BadRequest(_)),
      "expected BadRequest, got {err:?}"
    );
    assert!(err.to_string().contains("sat-img-v1"));
    assert!(err.to_string().contains("no result_id"));
  }

  #[test]
  fn require_result_image_rejects_session_with_no_artifacts() {
    let session = CfsSessionGetResponse {
      name: "sat-img-v1".to_string(),
      configuration: None,
      ansible: None,
      target: None,
      status: None,
      tags: None,
      debug_on_failure: false,
      logs: None,
    };
    let err = require_result_image(&session).unwrap_err();
    assert!(matches!(err, Error::BadRequest(_)));
  }
}
