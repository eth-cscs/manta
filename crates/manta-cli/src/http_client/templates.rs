//! BOS template endpoints: list, create-session.

use serde::Serialize;
use serde_json::Value;

use manta_shared::shared::dto::BosSessionTemplate;
use manta_shared::shared::params::template::GetTemplateParams;

use super::{MantaClient, QueryBuilder};

/// Request body for `POST /templates/{name}/sessions`.
#[derive(Serialize)]
pub struct ApplyTemplateSessionRequest<'a> {
  pub operation: &'a str,
  pub limit: &'a str,
  pub session_name: Option<&'a str>,
  pub include_disabled: bool,
  pub dry_run: bool,
}

impl MantaClient {
  pub async fn get_templates(
    &self,
    token: &str,
    params: &GetTemplateParams,
  ) -> anyhow::Result<Vec<BosSessionTemplate>> {
    let q = QueryBuilder::new()
      .opt("name", &params.name)
      .opt("hsm_group", &params.hsm_group)
      .opt_display("limit", &params.limit)
      .build();
    self.get_json(token, "/templates", &q).await
  }

  pub async fn apply_template_session(
    &self,
    token: &str,
    name: &str,
    req: &ApplyTemplateSessionRequest<'_>,
  ) -> anyhow::Result<Value> {
    self
      .post_json(token, &format!("/templates/{name}/sessions"), req)
      .await
  }
}
