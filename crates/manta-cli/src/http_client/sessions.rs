//! CFS session endpoints: list, create, delete, stream logs.

use anyhow::{Context, bail};
use futures::TryStreamExt;
use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncBufRead, BufReader};
use tokio_util::io::StreamReader;

use manta_shared::types::dto::CfsSessionGetResponse;
use manta_shared::types::params::session::GetSessionParams;

use super::{MantaClient, QueryBuilder};

/// Request body for `POST /sessions`.
#[derive(Serialize)]
pub struct CreateSessionRequest<'a> {
  pub cfs_conf_sess_name: Option<&'a str>,
  pub playbook_yaml_file_name: Option<&'a str>,
  pub group_name: Option<&'a str>,
  pub repo_names: &'a [&'a str],
  pub repo_last_commit_ids: &'a [&'a str],
  pub ansible_limit: Option<&'a str>,
  pub ansible_verbosity: Option<&'a str>,
  pub ansible_passthrough: Option<&'a str>,
}

impl MantaClient {
  pub async fn get_sessions(
    &self,
    token: &str,
    params: &GetSessionParams,
  ) -> anyhow::Result<Vec<CfsSessionGetResponse>> {
    let q = QueryBuilder::new()
      .opt("hsm_group", &params.group)
      .vec("xnames", &params.xnames)
      .opt("min_age", &params.min_age)
      .opt("max_age", &params.max_age)
      .opt("session_type", &params.session_type)
      .opt("status", &params.status)
      .opt("name", &params.name)
      .opt_display("limit", &params.limit)
      .build();
    self.get_json(token, "/sessions", &q).await
  }

  pub async fn create_session(
    &self,
    token: &str,
    req: &CreateSessionRequest<'_>,
  ) -> anyhow::Result<(String, String)> {
    let resp: Value = self.post_json(token, "/sessions", req).await?;
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

  pub async fn delete_session(
    &self,
    token: &str,
    name: &str,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let q = [("dry_run", dry_run.to_string())];
    self
      .delete_json_with_query(token, &format!("/sessions/{name}"), &q)
      .await
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
    let url = format!("{}/sessions/{}/logs", self.base_url(), session_name);
    let builder = self
      .http_client()
      .get(&url)
      .bearer_auth(token)
      .header("X-Manta-Site", self.site_name())
      .query(&[("timestamps", timestamps.to_string())]);
    Self::log_request_as_curl(&builder);
    let resp = builder
      .send()
      .await
      .context("HTTP GET session logs failed")?;

    if !resp.status().is_success() {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      bail!("GET session logs returned {status}: {body}");
    }

    let byte_stream = resp.bytes_stream().map_err(std::io::Error::other);
    Ok(BufReader::new(StreamReader::new(byte_stream)))
  }
}
