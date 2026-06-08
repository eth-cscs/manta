//! BOS template endpoints: list, create-session.

use serde_json::Value;

use manta_shared::types::dto::BosSessionTemplate;
use manta_shared::types::params::template::GetTemplateParams;
pub use manta_shared::types::wire::template::{
  BosOperation, PostTemplateSessionRequest,
};

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_templates(
    &self,
    token: &str,
    params: &GetTemplateParams,
  ) -> anyhow::Result<Vec<BosSessionTemplate>> {
    let q = QueryBuilder::new()
      .opt("name", &params.name)
      .opt("hsm_group", &params.group_name)
      .opt_display("limit", &params.limit)
      .build();
    self.get_json(token, "/templates", &q).await
  }

  pub async fn apply_template_session(
    &self,
    token: &str,
    name: &str,
    req: &PostTemplateSessionRequest,
  ) -> anyhow::Result<Value> {
    self
      .post_json(token, &format!("/templates/{name}/sessions"), req)
      .await
  }
}
