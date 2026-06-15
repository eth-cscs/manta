//! CFS configuration queries, layer-detail lookups, and cascading deletion of
//! all dependent resources (sessions, BOS templates, IMS images).

use chrono::NaiveDateTime;
use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::{
  validate_user_group_access, validate_user_group_vec_access,
};
pub use manta_shared::types::api::configuration::GetConfigurationParams;

/// Data gathered for deletion review and execution.
#[derive(serde::Serialize)]
pub struct DeletionCandidates {
  /// CFS sessions whose desired-config matches a candidate configuration.
  pub cfs_sessions_to_delete: Vec<CfsSessionGetResponse>,
  /// BOS session templates to delete: `(name, cfs_config, description)`.
  pub bos_sessiontemplate_tuples: Vec<(String, String, String)>,
  /// IMS image IDs to delete (built by the matching sessions).
  pub image_ids: Vec<String>,
  /// Names of the configurations selected for deletion.
  pub configuration_names: Vec<String>,
  /// CFS sessions summary tuples: `(name, config_name, status)`.
  pub cfs_session_tuples: Vec<(String, String, String)>,
  /// Full configuration objects selected for deletion.
  pub configurations: Vec<CfsConfigurationResponse>,
}

/// List CFS configurations the caller may see.
///
/// When `params.group_name` is set, access to that group is validated
/// first; otherwise the search is scoped to every group the token
/// already grants access to. Name / pattern / date filters and the
/// per-call `limit` are applied by the backend.
pub async fn get_configurations(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetConfigurationParams,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  // Get list of target groups the user is asking for
  let target_group_vec: Vec<String> = if let Some(group) = &params.group_name {
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

  // Validate groups and get list of groups available
  validate_user_group_vec_access(infra, token, &target_group_vec).await?;

  let limit_ref = params.limit.as_ref();

  let cfs_configuration_vec = infra
    .backend
    .get_and_filter_configuration(
      token,
      params.name.as_deref(),
      params.pattern.as_deref(),
      &target_group_vec,
      params.since,
      params.until,
      limit_ref,
    )
    .await?;

  Ok(cfs_configuration_vec)
}

/// Like [`get_configurations`] but pairs every row with a
/// `safe_to_delete` verdict by fetching CFS components and running
/// the pure [`build_configuration_analysis`] linker.
///
/// The verdict is **CFS-components-only**: a configuration is unsafe
/// if any CFS component lists it as its `desired_config`. The endpoint
/// does not check whether any BSS-referenced image was built from the
/// configuration; skipping the BSS and IMS fetches keeps this listing
/// fast and avoids the upstream-proxy resets that fanning out four
/// heavy fetches has been prone to.
pub async fn get_configurations_with_analysis(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetConfigurationParams,
) -> Result<Vec<crate::service::analysis::ConfigurationAnalysis>, Error> {
  let configs = get_configurations(infra, token, params).await?;
  let components =
    infra.backend.get_cfs_components(token, None, None, None).await?;
  Ok(crate::service::analysis::build_configuration_analysis(
    configs,
    components,
    vec![],
    vec![],
  ))
}

/// Collect every resource that would be removed by a cascading
/// configuration delete, without actually deleting anything.
///
/// Returns the configurations matching `configuration_name_pattern`
/// (within `since`/`until` if provided) plus the CFS sessions, BOS
/// session templates, and IMS images that depend on them. The CLI
/// shows this set as a confirmation prompt before invoking
/// [`delete_configurations_and_derivatives`].
///
/// When `settings_hsm_group_name_opt` is `Some(name)`, the caller's
/// access to that group is validated first; when `None`, the
/// candidate set is scoped to every group the token already grants
/// access to. The backend's `get_data_to_delete` only walks resources
/// reachable from the supplied group set, so the candidates returned
/// here are guaranteed to be reachable through the caller's
/// accessible-group lens.
pub(crate) async fn get_deletion_candidates(
  infra: &InfraContext<'_>,
  token: &str,
  settings_hsm_group_name_opt: Option<&str>,
  configuration_name_pattern: Option<&str>,
  since: Option<NaiveDateTime>,
  until: Option<NaiveDateTime>,
) -> Result<DeletionCandidates, Error> {
  validate_date_range(since, until)?;

  let target_hsm_group_vec =
    if let Some(settings_hsm_group_name) = settings_hsm_group_name_opt {
      // Defense-in-depth: today the handler always passes `None`, but
      // if a future caller (CLI, another handler) routes a user-
      // supplied group label through here, an unchecked group would
      // let the caller cascade-delete configurations they don't own.
      validate_user_group_access(infra, token, settings_hsm_group_name).await?;
      vec![settings_hsm_group_name.to_string()]
    } else {
      infra.backend.get_group_name_available(token).await?
    };

  let (
    cfs_sessions_to_delete,
    bos_sessiontemplate_tuples,
    image_ids,
    configuration_names,
    cfs_session_tuples,
    configurations,
  ) = infra
    .backend
    .get_data_to_delete(
      token,
      &target_hsm_group_vec,
      configuration_name_pattern,
      since,
      until,
    )
    .await?;
  Ok(DeletionCandidates {
    cfs_sessions_to_delete,
    bos_sessiontemplate_tuples,
    image_ids,
    configuration_names,
    cfs_session_tuples,
    configurations,
  })
}

/// Validate that a `(since, until)` date range is well-ordered.
///
/// Extracted so the HTTP handler and CLI can share the check without
/// constructing a full backend context.
pub fn validate_date_range(
  since: Option<NaiveDateTime>,
  until: Option<NaiveDateTime>,
) -> Result<(), Error> {
  if let (Some(s), Some(u)) = (since, until)
    && s > u
  {
    return Err(Error::BadRequest(
      "'since' date can't be after 'until' date".to_string(),
    ));
  }
  Ok(())
}

/// Apply a cascading delete previously planned by
/// [`get_deletion_candidates`].
///
/// Removes the named configurations together with every dependent
/// CFS session, BOS session template, and IMS image listed in
/// `candidates`. The two-step plan/apply split exists so the caller
/// can show the user exactly what is about to disappear before any
/// state changes.
pub(crate) async fn delete_configurations_and_derivatives(
  infra: &InfraContext<'_>,
  token: &str,
  candidates: &DeletionCandidates,
) -> Result<(), Error> {
  let cfs_session_name_vec: Vec<String> = candidates
    .cfs_session_tuples
    .iter()
    .map(|(session, _, _)| session.clone())
    .collect();

  let bos_sessiontemplate_name_vec: Vec<String> = candidates
    .bos_sessiontemplate_tuples
    .iter()
    .map(|(sessiontemplate, _, _)| sessiontemplate.clone())
    .collect();

  infra
    .backend
    .delete(
      token,
      &candidates.configuration_names,
      &candidates.image_ids,
      &cfs_session_name_vec,
      &bos_sessiontemplate_name_vec,
    )
    .await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::NaiveDateTime;

  fn dt(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").unwrap()
  }

  #[test]
  fn validate_date_range_ok_when_since_before_until() {
    assert!(
      validate_date_range(
        Some(dt("2024-01-01T00:00:00")),
        Some(dt("2024-01-02T00:00:00"))
      )
      .is_ok()
    );
  }

  #[test]
  fn validate_date_range_ok_when_equal() {
    let d = dt("2024-01-01T00:00:00");
    assert!(validate_date_range(Some(d), Some(d)).is_ok());
  }

  #[test]
  fn validate_date_range_ok_when_either_none() {
    let d = dt("2024-01-01T00:00:00");
    assert!(validate_date_range(Some(d), None).is_ok());
    assert!(validate_date_range(None, Some(d)).is_ok());
    assert!(validate_date_range(None, None).is_ok());
  }

  #[test]
  fn validate_date_range_err_when_since_after_until() {
    let result = validate_date_range(
      Some(dt("2024-01-02T00:00:00")),
      Some(dt("2024-01-01T00:00:00")),
    );
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("'since' date can't be after 'until' date")
    );
  }
}
