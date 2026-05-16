//! CFS configuration queries, layer-detail lookups, and cascading deletion of
//! all dependent resources (sessions, BOS templates, IMS images).

use manta_backend_dispatcher::error::Error;
use chrono::NaiveDateTime;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::common::app_context::InfraContext;
use crate::common::authorization::get_groups_names_available;

/// Typed parameters for fetching CFS configurations.
pub struct GetConfigurationParams {
  pub name: Option<String>,
  pub pattern: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub since: Option<NaiveDateTime>,
  pub until: Option<NaiveDateTime>,
  pub limit: Option<u8>,
}

/// Fetch and filter CFS configurations from the backend.
pub async fn get_configurations(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetConfigurationParams,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let limit_ref = params.limit.as_ref();

  let cfs_configuration_vec = infra.backend
    .get_and_filter_configuration(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      params.name.as_deref(),
      params.pattern.as_deref(),
      &target_hsm_group_vec,
      params.since,
      params.until,
      limit_ref,
    )
    .await?;

  Ok(cfs_configuration_vec)
}

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

/// Fetch deletion candidates (no side effects).
pub async fn get_deletion_candidates(
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
      vec![settings_hsm_group_name.to_string()]
    } else {
      get_groups_names_available(
        infra.backend,
        token,
        None,
        settings_hsm_group_name_opt,
      )
      .await?
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
      infra.shasta_base_url,
      infra.shasta_root_cert,
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

/// Execute the deletion of configurations and derivatives.
pub async fn delete_configurations_and_derivatives(
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
      infra.shasta_base_url,
      infra.shasta_root_cert,
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
    assert!(validate_date_range(Some(dt("2024-01-01T00:00:00")), Some(dt("2024-01-02T00:00:00"))).is_ok());
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
    let result = validate_date_range(Some(dt("2024-01-02T00:00:00")), Some(dt("2024-01-01T00:00:00")));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("'since' date can't be after 'until' date"));
  }
}
