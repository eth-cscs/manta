use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::apply_session::ApplySessionTrait;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::component::Component;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::common::app_context::InfraContext;
use crate::common::authorization::get_groups_names_available;

/// Typed parameters for fetching CFS sessions.
pub struct GetSessionParams {
  pub hsm_group: Option<String>,
  pub xnames: Vec<String>,
  pub min_age: Option<String>,
  pub max_age: Option<String>,
  pub session_type: Option<String>,
  pub status: Option<String>,
  pub name: Option<String>,
  pub limit: Option<u8>,
}

/// Fetch and filter CFS sessions from the backend.
pub async fn get_sessions(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetSessionParams,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  tracing::info!("Get CFS sessions");

  infra.backend
    .get_and_filter_sessions(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
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
  pub session: CfsSessionGetResponse,
  pub image_ids: Vec<String>,
  pub group_available_vec: Vec<Group>,
  pub cfs_component_vec: Vec<Component>,
  pub bss_bootparameters_vec: Vec<BootParameters>,
}

/// Fetch session and related data, validate session exists.
pub async fn prepare_session_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  session_name: &str,
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<SessionDeletionContext, Error> {
  let group_available_names = get_groups_names_available(
    infra.backend,
    token,
    None,
    settings_hsm_group_name_opt,
  )
  .await
  .map_err(|e| Error::Message(e.to_string()))?;

  tracing::info!("Fetching data from the backend...");
  let start = std::time::Instant::now();

  let (
    group_available_vec,
    cfs_session_vec,
    cfs_component_vec,
    bss_bootparameters_vec,
  ) = tokio::try_join!(
    infra.backend.get_group_available(token),
    infra.backend.get_and_filter_sessions(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
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
    infra.backend.get_cfs_components(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      None,
      None,
      None,
    ),
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

/// Execute the session deletion.
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
      infra.shasta_base_url,
      infra.shasta_root_cert,
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
  let backend = infra.backend;

  let ansible_limit = if let Some(ansible_limit) = ansible_limit_opt {
    let xname_vec = crate::common::node_ops::resolve_hosts_expression(
      backend,
      token,
      ansible_limit,
      false,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;
    Some(xname_vec.join(","))
  } else {
    None
  };

  backend
    .apply_session(
      gitea_token,
      infra.gitea_base_url,
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
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
    .backend
    .get_and_filter_sessions(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      Vec::new(),
      Vec::new(),
      None, None, None, None,
      Some(&name.to_string()),
      None, None,
    )
    .await?;

  let session = sessions
    .first()
    .ok_or_else(|| Error::NotFound(format!("CFS session '{name}'")))?;

  let target_def = session
    .target
    .as_ref()
    .and_then(|t| t.definition.as_ref())
    .ok_or_else(|| Error::BadRequest(format!("CFS session '{name}' has no target definition")))?;

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
    .ok_or_else(|| Error::BadRequest(format!("CFS session '{name}' has no status")))?;

  if status != "running" {
    return Err(Error::Conflict(format!(
      "CFS session '{name}' is not running (status: '{status}')"
    )));
  }

  Ok(())
}
