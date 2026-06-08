//! CFS configuration endpoints: list, bulk-delete.

use serde_json::Value;

use manta_shared::types::dto::CfsConfigurationResponse;
use manta_shared::types::params::configuration::GetConfigurationParams;

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_configurations(
    &self,
    token: &str,
    params: &GetConfigurationParams,
  ) -> anyhow::Result<Vec<CfsConfigurationResponse>> {
    let q = QueryBuilder::new()
      .opt("name", &params.name)
      .opt("pattern", &params.pattern)
      .opt("hsm_group", &params.group_name)
      .opt_display("limit", &params.limit)
      .build();
    self.get_json(token, "/configurations", &q).await
  }

  pub async fn delete_configurations(
    &self,
    token: &str,
    pattern: Option<&str>,
    since: Option<&str>,
    until: Option<&str>,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let q = QueryBuilder::new()
      .pair("dry_run", dry_run.to_string())
      .opt("pattern", &pattern.map(String::from))
      .opt("since", &since.map(String::from))
      .opt("until", &until.map(String::from))
      .build();
    self
      .delete_json_with_query(token, "/configurations", &q)
      .await
  }
}
