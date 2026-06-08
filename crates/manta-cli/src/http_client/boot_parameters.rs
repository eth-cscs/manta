//! BSS boot-parameter endpoints: list, add, apply boot config, update, delete.

use serde_json::Value;

use manta_shared::types::dto::BootParameters;
use manta_shared::types::params::boot_parameters::{
  GetBootParametersParams, UpdateBootParametersParams,
};
pub use manta_shared::types::wire::boot_parameters::ApplyBootConfigRequest;

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_boot_parameters(
    &self,
    token: &str,
    params: &GetBootParametersParams,
  ) -> anyhow::Result<Vec<BootParameters>> {
    let q = QueryBuilder::new()
      .opt("hsm_group", &params.group_name)
      .opt("nodes", &params.host_expression)
      .build();
    self.get_json(token, "/boot-parameters", &q).await
  }

  pub async fn add_boot_parameters(
    &self,
    token: &str,
    bp: &BootParameters,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/boot-parameters", bp).await?;
    Ok(())
  }

  pub async fn apply_boot_config(
    &self,
    token: &str,
    req: &ApplyBootConfigRequest,
  ) -> anyhow::Result<Value> {
    self.post_json(token, "/boot-config", req).await
  }

  pub async fn update_boot_parameters(
    &self,
    token: &str,
    params: &UpdateBootParametersParams,
  ) -> anyhow::Result<()> {
    self.put_no_content(token, "/boot-parameters", params).await
  }

  pub async fn delete_boot_parameters(
    &self,
    token: &str,
    hosts: Vec<String>,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "hosts": hosts });
    self
      .delete_no_content_with_body(token, "/boot-parameters", &body)
      .await
  }
}
