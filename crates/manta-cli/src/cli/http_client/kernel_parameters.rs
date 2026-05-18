//! Kernel-parameter endpoints: list, add, apply, delete.

use serde_json::Value;

use manta_shared::shared::dto::BootParameters;
use manta_shared::shared::params::kernel_parameters::GetKernelParametersParams;

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_kernel_parameters(
    &self,
    token: &str,
    params: &GetKernelParametersParams,
  ) -> anyhow::Result<Vec<BootParameters>> {
    let q = QueryBuilder::new()
      .opt("hsm_group", &params.hsm_group)
      .opt("nodes", &params.nodes)
      .build();
    self.get_json(token, "/kernel-parameters", &q).await
  }

  /// POST /kernel-parameters/apply — replace/add/delete kernel parameters on nodes.
  /// `operation` is one of "add", "apply", "delete".
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_kernel_parameters(
    &self,
    token: &str,
    xnames_expression: Option<&str>,
    hsm_group: Option<&str>,
    operation: &str,
    params: &str,
    overwrite: bool,
    project_sbps: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "xnames_expression": xnames_expression,
      "hsm_group": hsm_group,
      "operation": operation,
      "params": params,
      "overwrite": overwrite,
      "project_sbps": project_sbps,
      "dry_run": dry_run,
    });
    self
      .post_json(token, "/kernel-parameters/apply", &body)
      .await
  }

  /// POST /kernel-parameters/add — merge new kernel parameters into existing entries.
  #[allow(clippy::too_many_arguments)]
  pub async fn add_kernel_parameters(
    &self,
    token: &str,
    params_str: &str,
    xnames_expression: Option<&str>,
    hsm_group: Option<&str>,
    overwrite: bool,
    project_sbps: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "params": params_str,
      "xnames_expression": xnames_expression,
      "hsm_group": hsm_group,
      "overwrite": overwrite,
      "project_sbps": project_sbps,
      "dry_run": dry_run,
    });
    self.post_json(token, "/kernel-parameters/add", &body).await
  }

  /// DELETE /kernel-parameters — remove named kernel parameters from node BSS entries.
  pub async fn delete_kernel_parameters(
    &self,
    token: &str,
    params_str: &str,
    xnames_expression: Option<&str>,
    hsm_group: Option<&str>,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "params": params_str,
      "xnames_expression": xnames_expression,
      "hsm_group": hsm_group,
      "dry_run": dry_run,
    });
    self
      .delete_json_with_body(token, "/kernel-parameters", &body)
      .await
  }
}
