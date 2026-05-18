//! BOS template endpoints: list, create-session.

use serde_json::Value;

use manta_shared::shared::dto::BosSessionTemplate;
use manta_shared::shared::params::template::GetTemplateParams;

use super::{MantaClient, QueryBuilder};

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

  #[allow(clippy::too_many_arguments)]
  pub async fn apply_template_session(
    &self,
    token: &str,
    name: &str,
    operation: &str,
    limit: &str,
    session_name: Option<&str>,
    include_disabled: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "operation": operation,
      "limit": limit,
      "session_name": session_name,
      "include_disabled": include_disabled,
      "dry_run": dry_run,
    });
    self
      .post_json(token, &format!("/templates/{}/sessions", name), &body)
      .await
  }
}
