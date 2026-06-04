//! CFS configuration queries, layer-detail lookups, and cascading deletion of
//! all dependent resources (sessions, BOS templates, IMS images).

use chrono::NaiveDateTime;
use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::get_groups_names_available;
use crate::service::infra_backend::DeletionCandidates;
pub use manta_shared::types::params::configuration::GetConfigurationParams;

/// Fetch and filter CFS configurations from the backend.
pub async fn get_configurations(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetConfigurationParams,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let limit_ref = params.limit.as_ref();

  let cfs_configuration_vec = infra
    .get_and_filter_configuration(
      token,
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

/// Fetch deletion candidates (no side effects).
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
      vec![settings_hsm_group_name.to_string()]
    } else {
      get_groups_names_available(infra, token, None, None).await?
    };

  infra
    .get_data_to_delete(
      token,
      &target_hsm_group_vec,
      configuration_name_pattern,
      since,
      until,
    )
    .await
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
    .delete_configurations_and_dependents(
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
