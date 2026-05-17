//! Thin HTTP client for forwarding CLI calls to a remote manta server.
//!
//! When `manta_server_url` is set in the CLI configuration, commands call
//! `MantaClient` methods instead of the service layer.  The server resolves
//! CA certificates, base URLs, and credentials internally — the CLI only
//! sends `X-Manta-Site` + `Authorization: Bearer <token>`.

use anyhow::{Context, bail};
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::io::{AsyncBufRead, BufReader};
use tokio_util::io::StreamReader;

use csm_rs::node::types::NodeDetails;
use manta_shared::shared::dto::{
  BootParameters, BosSessionTemplate, CfsConfigurationResponse,
  CfsSessionGetResponse, Group, Image,
};

use manta_shared::shared::params::{
  boot_parameters::{GetBootParametersParams, UpdateBootParametersParams},
  cluster::GetClusterParams,
  configuration::GetConfigurationParams,
  group::GetGroupParams,
  hardware::{GetHardwareClusterParams, GetHardwareNodesListParams},
  image::GetImagesParams,
  kernel_parameters::GetKernelParametersParams,
  node::GetNodesParams,
  redfish_endpoints::{GetRedfishEndpointsParams, UpdateRedfishEndpointParams},
  session::GetSessionParams,
  template::GetTemplateParams,
};

/// Image entry as returned by `GET /api/v1/images`.
#[derive(serde::Deserialize)]
struct ImageEntry {
  image: serde_json::Value,
  configuration_name: String,
  image_id: String,
  is_linked: bool,
}

/// HTTP client that forwards CLI requests to a manta server.
#[derive(Debug)]
pub struct MantaClient {
  client: reqwest::Client,
  base_url: String,
  site_name: String,
}

impl MantaClient {
  /// Build a client pointing at `server_url` for the given `site_name`.
  ///
  /// If `server_url` has no scheme, `http://` is prepended. This lets users
  /// write `manta_server_url = "localhost:8080"` in their config without
  /// triggering a "URL scheme is not allowed" error from reqwest.
  pub fn new(server_url: &str, site_name: &str) -> anyhow::Result<Self> {
    let normalized = if server_url.starts_with("http://")
      || server_url.starts_with("https://")
    {
      server_url.to_owned()
    } else {
      format!("http://{}", server_url)
    };

    let client = reqwest::Client::builder()
      .build()
      .context("Failed to build HTTP client")?;
    Ok(Self {
      client,
      base_url: format!("{}/api/v1", normalized.trim_end_matches('/')),
      site_name: site_name.to_owned(),
    })
  }

  // ── private helpers ───────────────────────────────────────────────────────

  async fn parse_json<T: DeserializeOwned>(
    resp: reqwest::Response,
  ) -> anyhow::Result<T> {
    if resp.status().is_success() {
      resp
        .json::<T>()
        .await
        .context("Failed to parse response JSON")
    } else {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("Server returned {}: {}", status, body)
    }
  }

  async fn parse_no_content(resp: reqwest::Response) -> anyhow::Result<()> {
    if resp.status().is_success() {
      Ok(())
    } else {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("Server returned {}: {}", status, body)
    }
  }

  async fn get_json<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .get(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query)
      .send()
      .await
      .context("HTTP GET failed")?;
    Self::parse_json(resp).await
  }

  async fn post_json<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .post(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP POST failed")?;
    Self::parse_json(resp).await
  }

  async fn put_no_content(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .put(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP PUT failed")?;
    Self::parse_no_content(resp).await
  }

  async fn delete_no_content(
    &self,
    token: &str,
    path: &str,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  async fn delete_no_content_with_query(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  async fn delete_no_content_with_body(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<()> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_no_content(resp).await
  }

  async fn delete_json_with_body<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    body: &impl serde::Serialize,
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .json(body)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_json(resp).await
  }

  async fn delete_json_with_query<T: DeserializeOwned>(
    &self,
    token: &str,
    path: &str,
    query: &[(&str, String)],
  ) -> anyhow::Result<T> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self
      .client
      .delete(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(query)
      .send()
      .await
      .context("HTTP DELETE failed")?;
    Self::parse_json(resp).await
  }

  // ── GET endpoints ─────────────────────────────────────────────────────────

  pub async fn get_sessions(
    &self,
    token: &str,
    params: &GetSessionParams,
  ) -> anyhow::Result<Vec<CfsSessionGetResponse>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.hsm_group {
      q.push(("hsm_group", v.clone()));
    }
    if !params.xnames.is_empty() {
      q.push(("xnames", params.xnames.join(",")));
    }
    if let Some(v) = &params.min_age {
      q.push(("min_age", v.clone()));
    }
    if let Some(v) = &params.max_age {
      q.push(("max_age", v.clone()));
    }
    if let Some(v) = &params.session_type {
      q.push(("session_type", v.clone()));
    }
    if let Some(v) = &params.status {
      q.push(("status", v.clone()));
    }
    if let Some(v) = &params.name {
      q.push(("name", v.clone()));
    }
    if let Some(v) = &params.limit {
      q.push(("limit", v.to_string()));
    }
    self.get_json(token, "/sessions", &q).await
  }

  pub async fn get_configurations(
    &self,
    token: &str,
    params: &GetConfigurationParams,
  ) -> anyhow::Result<Vec<CfsConfigurationResponse>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.name {
      q.push(("name", v.clone()));
    }
    if let Some(v) = &params.pattern {
      q.push(("pattern", v.clone()));
    }
    if let Some(v) = &params.hsm_group {
      q.push(("hsm_group", v.clone()));
    }
    if let Some(v) = &params.limit {
      q.push(("limit", v.to_string()));
    }
    self.get_json(token, "/configurations", &q).await
  }

  pub async fn get_groups(
    &self,
    token: &str,
    params: &GetGroupParams,
  ) -> anyhow::Result<Vec<Group>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.group_name {
      q.push(("name", v.clone()));
    }
    self.get_json(token, "/groups", &q).await
  }

  pub async fn get_nodes(
    &self,
    token: &str,
    params: &GetNodesParams,
  ) -> anyhow::Result<Vec<NodeDetails>> {
    let mut q: Vec<(&str, String)> = vec![("xname", params.xname.clone())];
    if params.include_siblings {
      q.push(("include_siblings", "true".to_string()));
    }
    if let Some(v) = &params.status_filter {
      q.push(("status", v.clone()));
    }
    self.get_json(token, "/nodes", &q).await
  }

  pub async fn get_images(
    &self,
    token: &str,
    params: &GetImagesParams,
  ) -> anyhow::Result<Vec<(Image, String, String, bool)>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.id {
      q.push(("id", v.clone()));
    }
    if let Some(v) = &params.hsm_group {
      q.push(("hsm_group", v.clone()));
    }
    if let Some(v) = &params.limit {
      q.push(("limit", v.to_string()));
    }
    let entries: Vec<ImageEntry> = self.get_json(token, "/images", &q).await?;
    entries
      .into_iter()
      .map(|e| {
        let img: Image = serde_json::from_value(e.image)
          .context("Failed to deserialize image")?;
        Ok((img, e.configuration_name, e.image_id, e.is_linked))
      })
      .collect()
  }

  pub async fn get_templates(
    &self,
    token: &str,
    params: &GetTemplateParams,
  ) -> anyhow::Result<Vec<BosSessionTemplate>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.name {
      q.push(("name", v.clone()));
    }
    if let Some(v) = &params.hsm_group {
      q.push(("hsm_group", v.clone()));
    }
    if let Some(v) = &params.limit {
      q.push(("limit", v.to_string()));
    }
    self.get_json(token, "/templates", &q).await
  }

  pub async fn get_boot_parameters(
    &self,
    token: &str,
    params: &GetBootParametersParams,
  ) -> anyhow::Result<Vec<BootParameters>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.hsm_group {
      q.push(("hsm_group", v.clone()));
    }
    if let Some(v) = &params.nodes {
      q.push(("nodes", v.clone()));
    }
    self.get_json(token, "/boot-parameters", &q).await
  }

  pub async fn get_kernel_parameters(
    &self,
    token: &str,
    params: &GetKernelParametersParams,
  ) -> anyhow::Result<Vec<BootParameters>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.hsm_group {
      q.push(("hsm_group", v.clone()));
    }
    if let Some(v) = &params.nodes {
      q.push(("nodes", v.clone()));
    }
    self.get_json(token, "/kernel-parameters", &q).await
  }

  pub async fn get_redfish_endpoints(
    &self,
    token: &str,
    params: &GetRedfishEndpointsParams,
  ) -> anyhow::Result<serde_json::Value> {
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(v) = &params.id {
      q.push(("id", v.clone()));
    }
    if let Some(v) = &params.fqdn {
      q.push(("fqdn", v.clone()));
    }
    if let Some(v) = &params.uuid {
      q.push(("uuid", v.clone()));
    }
    if let Some(v) = &params.macaddr {
      q.push(("macaddr", v.clone()));
    }
    if let Some(v) = &params.ipaddress {
      q.push(("ipaddress", v.clone()));
    }
    self.get_json(token, "/redfish-endpoints", &q).await
  }

  pub async fn get_clusters(
    &self,
    token: &str,
    params: &GetClusterParams,
  ) -> anyhow::Result<Vec<NodeDetails>> {
    let mut q: Vec<(&str, String)> = Vec::new();
    let hsm = params
      .hsm_group_name
      .as_deref()
      .or(params.settings_hsm_group_name.as_deref());
    if let Some(v) = hsm {
      q.push(("hsm_group", v.to_string()));
    }
    if let Some(v) = &params.status_filter {
      q.push(("status", v.clone()));
    }
    self.get_json(token, "/clusters", &q).await
  }

  pub async fn get_hardware_clusters(
    &self,
    token: &str,
    params: &GetHardwareClusterParams,
  ) -> anyhow::Result<Value> {
    let mut q: Vec<(&str, String)> = Vec::new();
    let hsm = params
      .hsm_group_name
      .as_deref()
      .or(params.settings_hsm_group_name.as_deref());
    if let Some(v) = hsm {
      q.push(("hsm_group", v.to_string()));
    }
    self.get_json(token, "/hardware-clusters", &q).await
  }

  pub async fn get_hardware_nodes_list(
    &self,
    token: &str,
    params: &GetHardwareNodesListParams,
  ) -> anyhow::Result<Value> {
    let q: Vec<(&str, String)> = vec![("xnames", params.xnames.clone())];
    self.get_json(token, "/hardware-nodes-list", &q).await
  }

  // ── POST endpoints ────────────────────────────────────────────────────────

  pub async fn add_node(
    &self,
    token: &str,
    id: &str,
    group: &str,
    enabled: bool,
    arch: Option<String>,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "id": id, "group": group, "enabled": enabled, "arch": arch });
    let _: Value = self.post_json(token, "/nodes", &body).await?;
    Ok(())
  }

  pub async fn create_group(
    &self,
    token: &str,
    group: Group,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/groups", &group).await?;
    Ok(())
  }

  pub async fn add_nodes_to_group(
    &self,
    token: &str,
    name: &str,
    hosts_expression: &str,
  ) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let body = serde_json::json!({ "hosts_expression": hosts_expression });
    let resp: Value = self
      .post_json(token, &format!("/groups/{}/members", name), &body)
      .await?;
    let added = resp["added"]
      .as_array()
      .map(|a| {
        a.iter()
          .filter_map(|v| v.as_str().map(String::from))
          .collect()
      })
      .unwrap_or_default();
    let removed = resp["removed"]
      .as_array()
      .map(|a| {
        a.iter()
          .filter_map(|v| v.as_str().map(String::from))
          .collect()
      })
      .unwrap_or_default();
    Ok((added, removed))
  }

  pub async fn add_boot_parameters(
    &self,
    token: &str,
    bp: &BootParameters,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/boot-parameters", bp).await?;
    Ok(())
  }

  // Thin wrappers below relay CLI flags straight into a JSON body; a Params
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

  pub async fn add_redfish_endpoint(
    &self,
    token: &str,
    params: UpdateRedfishEndpointParams,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/redfish-endpoints", &params).await?;
    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn create_session(
    &self,
    token: &str,
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name: Option<&str>,
    hsm_group: Option<&str>,
    repo_names: &[&str],
    repo_last_commit_ids: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
  ) -> anyhow::Result<(String, String)> {
    let body = serde_json::json!({
      "cfs_conf_sess_name": cfs_conf_sess_name,
      "playbook_yaml_file_name": playbook_yaml_file_name,
      "hsm_group": hsm_group,
      "repo_names": repo_names,
      "repo_last_commit_ids": repo_last_commit_ids,
      "ansible_limit": ansible_limit,
      "ansible_verbosity": ansible_verbosity,
      "ansible_passthrough": ansible_passthrough,
    });
    let resp: Value = self.post_json(token, "/sessions", &body).await?;
    let session_name = resp["session_name"]
      .as_str()
      .context("missing session_name in response")?
      .to_string();
    let config_name = resp["configuration_name"]
      .as_str()
      .context("missing configuration_name in response")?
      .to_string();
    Ok((session_name, config_name))
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

  pub async fn migrate_nodes(
    &self,
    token: &str,
    target_hsm_names: &[String],
    parent_hsm_names: &[String],
    hosts_expression: &str,
    dry_run: bool,
    create_hsm_group: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "target_hsm_names": target_hsm_names,
      "parent_hsm_names": parent_hsm_names,
      "hosts_expression": hosts_expression,
      "dry_run": dry_run,
      "create_hsm_group": create_hsm_group,
    });
    self.post_json(token, "/migrate/nodes", &body).await
  }

  pub async fn migrate_backup(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "bos": bos, "destination": destination });
    let _: Value = self.post_json(token, "/migrate/backup", &body).await?;
    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn migrate_restore(
    &self,
    token: &str,
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite: bool,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({
      "bos_file": bos_file,
      "cfs_file": cfs_file,
      "hsm_file": hsm_file,
      "ims_file": ims_file,
      "image_dir": image_dir,
      "overwrite": overwrite,
    });
    let _: Value = self.post_json(token, "/migrate/restore", &body).await?;
    Ok(())
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

  pub async fn add_hw_component(
    &self,
    token: &str,
    target: &str,
    parent_cluster: &str,
    pattern: &str,
    create_hsm_group: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "parent_cluster": parent_cluster,
      "pattern": pattern,
      "create_hsm_group": create_hsm_group,
      "dry_run": dry_run,
    });
    self
      .post_json(
        token,
        &format!("/hardware-clusters/{}/members", target),
        &body,
      )
      .await
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn apply_sat_file(
    &self,
    token: &str,
    sat_file_content: &str,
    values: Option<serde_json::Value>,
    values_file_content: Option<&str>,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<&str>,
    reboot: bool,
    watch_logs: bool,
    timestamps: bool,
    image_only: bool,
    session_template_only: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({
      "sat_file_content": sat_file_content,
      "values": values,
      "values_file_content": values_file_content,
      "ansible_verbosity": ansible_verbosity,
      "ansible_passthrough": ansible_passthrough,
      "reboot": reboot,
      "watch_logs": watch_logs,
      "timestamps": timestamps,
      "image_only": image_only,
      "session_template_only": session_template_only,
      "overwrite": overwrite,
      "dry_run": dry_run,
    });
    let _: Value = self.post_json(token, "/sat-file", &body).await?;
    Ok(())
  }

  // ── PUT endpoints ─────────────────────────────────────────────────────────

  pub async fn update_boot_parameters(
    &self,
    token: &str,
    params: &UpdateBootParametersParams,
  ) -> anyhow::Result<()> {
    self.put_no_content(token, "/boot-parameters", params).await
  }

  pub async fn update_redfish_endpoint(
    &self,
    token: &str,
    params: &UpdateRedfishEndpointParams,
  ) -> anyhow::Result<()> {
    self
      .put_no_content(token, "/redfish-endpoints", params)
      .await
  }

  // ── DELETE endpoints ──────────────────────────────────────────────────────

  pub async fn delete_node(&self, token: &str, id: &str) -> anyhow::Result<()> {
    self
      .delete_no_content(token, &format!("/nodes/{}", id))
      .await
  }

  pub async fn delete_group(
    &self,
    token: &str,
    label: &str,
    force: bool,
  ) -> anyhow::Result<()> {
    let q = [("force", force.to_string())];
    self
      .delete_no_content_with_query(token, &format!("/groups/{}", label), &q)
      .await
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

  pub async fn delete_redfish_endpoint(
    &self,
    token: &str,
    id: &str,
  ) -> anyhow::Result<()> {
    self
      .delete_no_content(token, &format!("/redfish-endpoints/{}", id))
      .await
  }

  pub async fn delete_session(
    &self,
    token: &str,
    name: &str,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let q = [("dry_run", dry_run.to_string())];
    self
      .delete_json_with_query(token, &format!("/sessions/{}", name), &q)
      .await
  }

  pub async fn delete_images(
    &self,
    token: &str,
    ids: &[&str],
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let q = [("ids", ids.join(",")), ("dry_run", dry_run.to_string())];
    self.delete_json_with_query(token, "/images", &q).await
  }

  pub async fn delete_configurations(
    &self,
    token: &str,
    pattern: Option<&str>,
    since: Option<&str>,
    until: Option<&str>,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let mut q: Vec<(&str, String)> = vec![("dry_run", dry_run.to_string())];
    if let Some(v) = pattern {
      q.push(("pattern", v.to_string()));
    }
    if let Some(v) = since {
      q.push(("since", v.to_string()));
    }
    if let Some(v) = until {
      q.push(("until", v.to_string()));
    }
    self
      .delete_json_with_query(token, "/configurations", &q)
      .await
  }

  pub async fn delete_group_members(
    &self,
    token: &str,
    name: &str,
    xnames_expression: &str,
    dry_run: bool,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "xnames_expression": xnames_expression, "dry_run": dry_run });
    self
      .delete_no_content_with_body(
        token,
        &format!("/groups/{}/members", name),
        &body,
      )
      .await
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

  pub async fn delete_hw_component(
    &self,
    token: &str,
    target: &str,
    parent_cluster: &str,
    pattern: &str,
    delete_hsm_group: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "parent_cluster": parent_cluster,
      "pattern": pattern,
      "delete_hsm_group": delete_hsm_group,
      "dry_run": dry_run,
    });
    self
      .delete_json_with_body(
        token,
        &format!("/hardware-clusters/{}/members", target),
        &body,
      )
      .await
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn apply_hw_configuration(
    &self,
    token: &str,
    target: &str,
    parent_cluster: &str,
    pattern: &str,
    mode: &str,
    dry_run: bool,
    create_target_hsm_group: bool,
    delete_empty_parent_hsm_group: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "parent_cluster": parent_cluster,
      "pattern": pattern,
      "mode": mode,
      "dry_run": dry_run,
      "create_target_hsm_group": create_target_hsm_group,
      "delete_empty_parent_hsm_group": delete_empty_parent_hsm_group,
    });
    self
      .post_json(
        token,
        &format!("/hardware-clusters/{}/configuration", target),
        &body,
      )
      .await
  }

  pub async fn power(
    &self,
    token: &str,
    action: &str,
    targets_expression: &str,
    target_type: &str,
    force: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "action": action,
      "targets_expression": targets_expression,
      "target_type": target_type,
      "force": force,
    });
    self.post_json(token, "/power", &body).await
  }

  pub async fn create_ephemeral_env(
    &self,
    token: &str,
    image_id: &str,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({ "image_id": image_id });
    self.post_json(token, "/ephemeral-env", &body).await
  }

  /// Stream CFS session logs from `GET /sessions/{name}/logs` (SSE).
  ///
  /// Returns a buffered reader over the SSE byte stream.  The caller is
  /// responsible for stripping the `data: ` prefix that the server wraps
  /// around each log line.
  pub async fn stream_session_logs(
    &self,
    token: &str,
    session_name: &str,
    timestamps: bool,
  ) -> anyhow::Result<impl AsyncBufRead + Send + Unpin> {
    let url = format!("{}/sessions/{}/logs", self.base_url, session_name);
    let resp = self
      .client
      .get(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", &self.site_name)
      .query(&[("timestamps", timestamps.to_string())])
      .send()
      .await
      .context("HTTP GET session logs failed")?;

    if !resp.status().is_success() {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("GET session logs returned {}: {}", status, body);
    }

    let byte_stream = resp.bytes_stream().map_err(std::io::Error::other);
    Ok(BufReader::new(StreamReader::new(byte_stream)))
  }

  /// Open a WebSocket console to a node and return async I/O streams.
  ///
  /// The returned `AsyncWrite` carries terminal stdin to the server; the
  /// returned `AsyncRead` delivers console output back to the terminal.
  pub async fn console_node(
    &self,
    token: &str,
    xname: &str,
    cols: u16,
    rows: u16,
  ) -> anyhow::Result<(
    Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
    Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  )> {
    let url = format!(
      "{}/nodes/{}/console?cols={}&rows={}",
      ws_base_url(&self.base_url),
      xname,
      cols,
      rows,
    );
    self.connect_console_ws(token, &url).await
  }

  /// Open a WebSocket console to a CFS session container.
  pub async fn console_session(
    &self,
    token: &str,
    session_name: &str,
    cols: u16,
    rows: u16,
  ) -> anyhow::Result<(
    Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
    Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  )> {
    let url = format!(
      "{}/sessions/{}/console?cols={}&rows={}",
      ws_base_url(&self.base_url),
      session_name,
      cols,
      rows,
    );
    self.connect_console_ws(token, &url).await
  }

  /// Connect to a WebSocket URL with bearer auth and return stdin/stdout pipes.
  ///
  /// Spawns a background task that bridges between the WebSocket and two
  /// `tokio::io::duplex` pipes. The caller receives:
  /// - an `AsyncWrite` to write terminal stdin (sent as Binary WS frames)
  /// - an `AsyncRead` to read console output (received as Binary WS frames)
  async fn connect_console_ws(
    &self,
    token: &str,
    url: &str,
  ) -> anyhow::Result<(
    Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
    Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  )> {
    use futures::{SinkExt, StreamExt};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::HeaderValue;

    let mut req = url.into_client_request().context("Invalid WebSocket URL")?;
    req.headers_mut().insert(
      "Authorization",
      HeaderValue::from_str(&format!("Bearer {}", token))
        .context("Invalid token header value")?,
    );
    req.headers_mut().insert(
      "X-Manta-Site",
      HeaderValue::from_str(&self.site_name)
        .context("Invalid site-name header value")?,
    );

    let (ws_stream, _) = tokio_tungstenite::connect_async(req)
      .await
      .context("WebSocket connection failed")?;

    let (mut ws_sink, mut ws_source) = ws_stream.split();

    // stdin pipe: run_console_loop writes to stdin_cli_end;
    //             bridge reads from stdin_bridge_end and sends Binary WS frames
    let (stdin_cli_end, mut stdin_bridge_end) = tokio::io::duplex(65536);
    // stdout pipe: bridge receives Binary WS frames and writes to stdout_bridge_end;
    //              run_console_loop reads from stdout_cli_end
    let (mut stdout_bridge_end, stdout_cli_end) = tokio::io::duplex(65536);

    tokio::spawn(async move {
      let mut buf = vec![0u8; 4096];
      loop {
        tokio::select! {
          n = stdin_bridge_end.read(&mut buf) => {
            match n {
              Ok(0) | Err(_) => break,
              Ok(n) => {
                let data = tokio_util::bytes::Bytes::copy_from_slice(&buf[..n]);
                if ws_sink.send(Message::Binary(data)).await.is_err() {
                  break;
                }
              }
            }
          }
          frame = ws_source.next() => {
            match frame {
              Some(Ok(Message::Binary(data))) => {
                if stdout_bridge_end.write_all(&data).await.is_err() { break; }
              }
              Some(Ok(Message::Text(text))) => {
                if stdout_bridge_end.write_all(text.as_bytes()).await.is_err() { break; }
              }
              Some(Ok(Message::Close(_))) | None => break,
              Some(Err(_)) => break,
              Some(Ok(_)) => {} // Ping/Pong ignored
            }
          }
        }
      }
    });

    Ok((Box::new(stdin_cli_end), Box::new(stdout_cli_end)))
  }
}

/// Convert an `http://` or `https://` base URL to the corresponding `ws://` / `wss://` URL.
fn ws_base_url(http_url: &str) -> String {
  if let Some(rest) = http_url.strip_prefix("https://") {
    format!("wss://{}", rest)
  } else if let Some(rest) = http_url.strip_prefix("http://") {
    format!("ws://{}", rest)
  } else {
    http_url.to_owned()
  }
}
