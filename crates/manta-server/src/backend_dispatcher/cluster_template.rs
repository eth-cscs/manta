//! [`ClusterTemplateTrait`] (BOS session template) impl for
//! [`StaticBackendDispatcher`].
//!
//! Forwards to `/apis/bos/v2/sessiontemplates`. Ochami uses the trait
//! default and returns [`Error::Message`] ("not implemented for this
//! backend").

use super::*;

impl ClusterTemplateTrait for StaticBackendDispatcher {
  /// `GET /sessiontemplates/{id}` when an id is supplied, otherwise
  /// the full list. Returned as a `Vec` for shape parity with the
  /// list call; a single-id lookup yields a one-element vector.
  async fn get_template(
    &self,
    token: &str,
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(self, get_template, token, bos_session_template_id_opt)
  }

  /// Fetch templates and apply client-side filtering by visible
  /// group names, observed xname members, an exact template name, and
  /// a max result count. BOS does not support these as native query
  /// params, so the backend over-fetches and narrows in-process.
  async fn get_and_filter_templates(
    &self,
    token: &str,
    group_name_vec: &[String],
    group_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(
      self,
      get_and_filter_templates,
      token,
      group_name_vec,
      group_member_vec,
      bos_sessiontemplate_name_opt,
      limit_number_opt
    )
  }

  /// `GET /sessiontemplates` — every template (unfiltered).
  async fn get_all_templates(
    &self,
    token: &str,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(self, get_all_templates, token)
  }

  /// `PUT /sessiontemplates/{name}` — create-or-replace. Returns the
  /// persisted template (with backend-assigned timestamps).
  async fn put_template(
    &self,
    token: &str,
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    dispatch!(self, put_template, token, bos_template, bos_template_name)
  }

  /// `DELETE /sessiontemplates/{id}`.
  async fn delete_template(
    &self,
    token: &str,
    bos_template_id: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_template, token, bos_template_id)
  }
}
