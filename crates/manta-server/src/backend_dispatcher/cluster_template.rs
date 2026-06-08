//! `ClusterTemplateTrait` (BOS session template) impl for
//! `StaticBackendDispatcher`.

use super::*;

impl ClusterTemplateTrait for StaticBackendDispatcher {
  async fn get_template(
    &self,
    token: &str,
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(self, get_template, token, bos_session_template_id_opt)
  }

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

  async fn get_all_templates(
    &self,
    token: &str,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    dispatch!(self, get_all_templates, token)
  }

  async fn put_template(
    &self,
    token: &str,
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    dispatch!(self, put_template, token, bos_template, bos_template_name)
  }

  async fn delete_template(
    &self,
    token: &str,
    bos_template_id: &str,
  ) -> Result<(), Error> {
    dispatch!(self, delete_template, token, bos_template_id)
  }
}
