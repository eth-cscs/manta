//! SSE log streaming endpoints.
//!
//! `GET /sessions/{name}/logs` returns a `text/event-stream` body the
//! CLI tails line-by-line. The progenitor-generated client doesn't
//! handle SSE streams (its `get_session_logs` deserialises the
//! response as a one-shot), so the stream lives on the raw
//! `reqwest::Client` instead. Bearer auth comes from the default
//! header set in [`crate::http_client::MantaClient::new_with_timeout`];
//! the `X-Manta-Site` header is attached per call.

use anyhow::{Context, bail};
use futures::TryStreamExt;
use tokio::io::{AsyncBufRead, BufReader};
use tokio_util::io::StreamReader;

use super::MantaClient;
use super::client::unwrap_error_body;

impl MantaClient {
  /// Stream CFS session logs from `GET /sessions/{name}/logs` (SSE).
  ///
  /// Returns a buffered reader over the SSE byte stream. The caller is
  /// responsible for stripping the `data: ` prefix that the server wraps
  /// around each log line.
  pub async fn stream_session_logs(
    &self,
    session_name: &str,
    timestamps: bool,
  ) -> anyhow::Result<impl AsyncBufRead + Send + Unpin> {
    let url = format!("{}/sessions/{}/logs", self.base_url(), session_name);
    let builder = self
      .raw
      .get(&url)
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
      bail!("GET session logs returned {status}: {}", unwrap_error_body(&body));
    }

    let byte_stream = resp.bytes_stream().map_err(std::io::Error::other);
    Ok(BufReader::new(StreamReader::new(byte_stream)))
  }
}
