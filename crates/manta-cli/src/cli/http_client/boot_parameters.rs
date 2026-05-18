//! BSS boot-parameter endpoints: list, add, apply boot config, update, delete.

use serde_json::Value;

use manta_shared::shared::dto::BootParameters;
use manta_shared::shared::params::boot_parameters::{
  GetBootParametersParams, UpdateBootParametersParams,
};

use super::{MantaClient, QueryBuilder};

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

  // Thin wrapper below relays CLI flags straight into a JSON body; a Params
  // struct would just relocate the same argument list, so we suppress
  // `clippy::too_many_arguments` instead of adding boilerplate types.
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_boot_config(
    &self,
    token: &str,
    hosts_expression: &str,
    boot_image_id: Option<&str>,
    boot_image_configuration: Option<&str>,
    kernel_parameters: Option<&str>,
    runtime_configuration: Option<&str>,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "hosts_expression": hosts_expression,
      "boot_image_id": boot_image_id,
      "boot_image_configuration": boot_image_configuration,
      "kernel_parameters": kernel_parameters,
      "runtime_configuration": runtime_configuration,
      "dry_run": dry_run,
    });
    self.post_json(token, "/boot-config", &body).await
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
