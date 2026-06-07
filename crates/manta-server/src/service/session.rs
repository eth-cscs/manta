//! CFS session queries, creation, deletion, and console-readiness validation.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::component::Component;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::get_groups_names_available;
use crate::service::node_ops;
pub use manta_shared::types::params::session::GetSessionParams;

/// Fetch and filter CFS sessions from the backend.
pub async fn get_sessions(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetSessionParams,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  tracing::info!("Get CFS sessions");

  infra
    .get_and_filter_sessions(
      token,
      params
        .hsm_group
        .as_ref()
        .map(|v| vec![v.clone()])
        .unwrap_or_default(),
      params.xnames.iter().map(String::as_str).collect(),
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

/// Fetch session and related data, validate session exists.
pub async fn prepare_session_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  session_name: &str,
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<SessionDeletionContext, Error> {
  let group_available_names =
    get_groups_names_available(infra, token, None, settings_hsm_group_name_opt)
      .await?;

  tracing::info!("Fetching data from the backend...");
  let start = std::time::Instant::now();

  let (
    group_available_vec,
    cfs_session_vec,
    cfs_component_vec,
    bss_bootparameters_vec,
  ) = tokio::try_join!(
    infra.get_group_available(token),
    infra.get_and_filter_sessions(
      token,
      group_available_names,
      Vec::new(),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    ),
    infra.get_cfs_components(token, None, None, None),
    infra.get_all_bootparameters(token),
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

/// Execute the session deletion.
pub async fn execute_session_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  deletion_ctx: &SessionDeletionContext,
  dry_run: bool,
) -> Result<(), Error> {
  infra
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

/// Resolve ansible-limit hosts to xnames and create a CFS session.
///
/// Returns `(cfs_configuration_name, cfs_session_name)`.
#[allow(clippy::too_many_arguments)]
pub async fn create_cfs_session(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  cfs_conf_sess_name: Option<&str>,
  playbook_yaml_file_name_opt: Option<&str>,
  hsm_group_opt: Option<&str>,
  repo_name_vec: &[&str],
  repo_last_commit_id_vec: &[&str],
  ansible_limit_opt: Option<&str>,
  ansible_verbosity: Option<&str>,
  ansible_passthrough: Option<&str>,
) -> Result<(String, String), Error> {
  let ansible_limit = if let Some(ansible_limit) = ansible_limit_opt {
    let xname_vec =
      node_ops::resolve_hosts_expression(infra, token, ansible_limit, false)
        .await?;
    Some(xname_vec.join(","))
  } else {
    None
  };

  infra
    .apply_session(
      gitea_token,
      token,
      cfs_conf_sess_name,
      playbook_yaml_file_name_opt,
      hsm_group_opt,
      repo_name_vec,
      repo_last_commit_id_vec,
      ansible_limit.as_deref(),
      ansible_verbosity,
      ansible_passthrough,
    )
    .await
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
/// target group is outside the accessible set — matching the existing
/// access-denial shape used by
/// [`crate::service::authorization::get_groups_names_available`].
pub async fn validate_session_access(
  infra: &InfraContext<'_>,
  token: &str,
  session_name: &str,
) -> Result<CfsSessionGetResponse, Error> {
  let sessions = infra
    .get_and_filter_sessions(
      token,
      Vec::new(),
      Vec::new(),
      None,
      None,
      None,
      None,
      Some(&session_name.to_string()),
      None,
      None,
    )
    .await?;

  let session = sessions.into_iter().next().ok_or_else(|| {
    Error::NotFound(format!("CFS session '{session_name}'"))
  })?;

  let target_groups = session.get_target_hsm().unwrap_or_default();
  if !target_groups.is_empty() {
    let accessible = infra.get_group_name_available(token).await?;
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
  let sessions = infra
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

  let session = sessions
    .first()
    .ok_or_else(|| Error::NotFound(format!("CFS session '{name}'")))?;

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

  fn session_with_result_id(name: &str, result_id: Option<&str>) -> CfsSessionGetResponse {
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
    let session =
      session_with_result_id("sat-img-v1", Some("ims-image-abc"));
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
