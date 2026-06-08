//! BOS (session + session-template) backend methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bos::{
  ClusterSessionTrait, ClusterTemplateTrait,
};
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// List BOS session templates filtered by HSM groups / members.
  pub async fn get_and_filter_templates(
    &self,
    token: &str,
    hsm_group_name_vec: &[String],
    hsm_member_vec: &[String],
    bos_sessiontemplate_name_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    self
      .backend
      .get_and_filter_templates(
        token,
        hsm_group_name_vec,
        hsm_member_vec,
        bos_sessiontemplate_name_opt,
        limit_number_opt,
      )
      .await
  }

  /// Create a BOS session from a template.
  pub async fn post_template_session(
    &self,
    token: &str,
    bos_session: BosSession,
  ) -> Result<BosSession, Error> {
    self.backend.post_template_session(token, bos_session).await
  }
}
