//! BSS boot-parameter endpoints: list, add, apply boot config, update, delete.

use serde::Serialize;
use serde_json::Value;

use manta_shared::shared::dto::BootParameters;
use manta_shared::shared::params::boot_parameters::{
  GetBootParametersParams, UpdateBootParametersParams,
};

use super::{MantaClient, QueryBuilder};

/// Request body for `POST /boot-config`.
#[derive(Serialize)]
pub struct ApplyBootConfigRequest<'a> {
  pub hosts_expression: &'a str,
  pub boot_image_id: Option<&'a str>,
  pub boot_image_configuration: Option<&'a str>,
  pub kernel_parameters: Option<&'a str>,
  pub runtime_configuration: Option<&'a str>,
  pub dry_run: bool,
}

impl MantaClient {
  pub async fn get_boot_parameters(
    &self,
    token: &str,
    params: &GetBootParametersParams,
  ) -> anyhow::Result<Vec<BootParameters>> {
    let q = QueryBuilder::new()
      .opt("hsm_group", &params.hsm_group)
      .opt("nodes", &params.nodes)
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
    req: &ApplyBootConfigRequest<'_>,
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
