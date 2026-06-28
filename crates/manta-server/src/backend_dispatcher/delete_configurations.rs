//! [`DeleteConfigurationsAndDataRelatedTrait`] impl for
//! [`StaticBackendDispatcher`].
//!
//! Implements the "delete CFS configurations and everything that
//! references them" cascade — CFS sessions, IMS images, BOS templates,
//! and the configurations themselves. Used by the `delete configs`
//! CLI path and the corresponding HTTP handler. Ochami uses the
//! trait default and returns [`Error::Message`] for both methods.

use super::*;

impl DeleteConfigurationsAndDataRelatedTrait for StaticBackendDispatcher {
  /// Dry-run inventory: enumerate every artifact that would be
  /// touched by a matching [`delete`](Self::delete) call without
  /// actually deleting anything. Returns
  /// `(sessions, image_refs, configuration_names,
  /// session_template_names, template_refs, configurations)` where
  /// the `(String, String, String)` triples carry id/name/derived-id
  /// triples ready to be shown to the operator before they confirm.
  async fn get_data_to_delete(
    &self,
    token: &str,
    group_name_available_vec: &[String],
    configuration_name_pattern_opt: Option<&str>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
  ) -> Result<
    (
      Vec<CfsSessionGetResponse>,
      Vec<(String, String, String)>,
      Vec<String>,
      Vec<String>,
      Vec<(String, String, String)>,
      Vec<CfsConfigurationResponse>,
    ),
    Error,
  > {
    dispatch!(
      self,
      get_data_to_delete,
      token,
      group_name_available_vec,
      configuration_name_pattern_opt,
      since_opt,
      until_opt
    )
  }

  /// Execute the cascade: delete BOS templates, CFS sessions, IMS
  /// images, and finally the CFS configurations themselves, in that
  /// order so foreign-key-like references are gone before the
  /// configurations they point at. Each input slice is the exact set
  /// of names/ids to delete — typically the output of
  /// [`get_data_to_delete`](Self::get_data_to_delete).
  async fn delete(
    &self,
    token: &str,
    cfs_configuration_name_vec: &[String],
    image_id_vec: &[String],
    cfs_session_name_vec: &[String],
    bos_sessiontemplate_name_vec: &[String],
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete,
      token,
      cfs_configuration_name_vec,
      image_id_vec,
      cfs_session_name_vec,
      bos_sessiontemplate_name_vec
    )
  }
}
