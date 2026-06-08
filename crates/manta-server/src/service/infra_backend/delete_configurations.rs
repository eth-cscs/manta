//! Multi-resource configuration-deletion methods on `InfraContext`.

use chrono::NaiveDateTime;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::server::common::app_context::InfraContext;

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

impl InfraContext<'_> {
  /// Collect every artefact that would be deleted when removing matching configurations.
  pub async fn get_data_to_delete(
    &self,
    token: &str,
    hsm_name_available_vec: &[String],
    configuration_name_pattern_opt: Option<&str>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
  ) -> Result<DeletionCandidates, Error> {
    let (
      cfs_sessions_to_delete,
      bos_sessiontemplate_tuples,
      image_ids,
      configuration_names,
      cfs_session_tuples,
      configurations,
    ) = self
      .backend
      .get_data_to_delete(
        token,
        hsm_name_available_vec,
        configuration_name_pattern_opt,
        since_opt,
        until_opt,
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

  /// Delete CFS configurations along with their dependent images, sessions, and templates.
  pub async fn delete_configurations_and_dependents(
    &self,
    token: &str,
    cfs_configuration_name_vec: &[String],
    image_id_vec: &[String],
    cfs_session_name_vec: &[String],
    bos_sessiontemplate_name_vec: &[String],
  ) -> Result<(), Error> {
    self
      .backend
      .delete(
        token,
        cfs_configuration_name_vec,
        image_id_vec,
        cfs_session_name_vec,
        bos_sessiontemplate_name_vec,
      )
      .await
  }
}
