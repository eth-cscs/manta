//! Kernel-parameter endpoints: list, add, apply, delete.

use serde::Serialize;
use serde_json::Value;

use manta_shared::types::dto::BootParameters;
use manta_shared::types::params::kernel_parameters::GetKernelParametersParams;

use super::{MantaClient, QueryBuilder};

/// Request body for `POST /kernel-parameters/apply` (replace mode).
#[derive(Serialize)]
pub struct ApplyKernelParametersRequest<'a> {
  pub xnames_expression: Option<&'a str>,
  pub hsm_group: Option<&'a str>,
  pub operation: &'a str,
  pub params: &'a str,
  pub overwrite: bool,
  pub project_sbps: bool,
  pub dry_run: bool,
}

/// Request body for `POST /kernel-parameters/add` (append/merge mode).
#[derive(Serialize)]
pub struct AddKernelParametersRequest<'a> {
  pub params: &'a str,
  pub xnames_expression: Option<&'a str>,
  pub hsm_group: Option<&'a str>,
  pub overwrite: bool,
  pub project_sbps: bool,
  pub dry_run: bool,
}

impl MantaClient {
  pub async fn get_kernel_parameters(
    &self,
    token: &str,
    params: &GetKernelParametersParams,
  ) -> anyhow::Result<Vec<BootParameters>> {
    let q = QueryBuilder::new()
      .opt("hsm_group", &params.group_name)
      .opt("nodes", &params.nodes)
      .build();
    self.get_json(token, "/kernel-parameters", &q).await
  }

  /// POST /kernel-parameters/apply — replace/add/delete kernel parameters on nodes.
  /// `operation` is one of "add", "apply", "delete".
  pub async fn apply_kernel_parameters(
    &self,
    token: &str,
    req: &ApplyKernelParametersRequest<'_>,
  ) -> anyhow::Result<Value> {
    self.post_json(token, "/kernel-parameters/apply", req).await
  }

  /// POST /kernel-parameters/add — merge new kernel parameters into existing entries.
  pub async fn add_kernel_parameters(
    &self,
    token: &str,
    req: &AddKernelParametersRequest<'_>,
  ) -> anyhow::Result<Value> {
    self.post_json(token, "/kernel-parameters/add", req).await
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
