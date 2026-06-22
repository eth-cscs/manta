//! HAND-ROLLED — not generated from the OpenAPI spec.
//!
//! SSE (server-sent events) log streaming endpoint.
//! `GET /sessions/{name}/logs` returns a `text/event-stream` body the
//! CLI tails line-by-line.
//!
//! ## Why not auto-generated
//!
//! progenitor's `get_session_logs` deserialises the response as a
//! one-shot JSON document — fine for short responses, useless for an
//! open `text/event-stream` connection that streams lines indefinitely.
//! The CLI needs the body as `impl AsyncBufRead`, so this endpoint
//! stays on the raw `reqwest::Client` (`MantaClient::raw_client()`).
//! Bearer auth comes from the default `Authorization` header set in
//! [`crate::http_client::MantaClient::new_with_timeout`]; the
//! `X-Manta-Site` header is attached per call.
//!
//! Adding a new streaming endpoint? Hand-roll it here. Everything
//! else should be added to the server with `#[utoipa::path(...)]`
//! and consumed through the regenerated `client.openapi.*` methods.

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
      bail!(
        "GET session logs returned {status}: {}",
        unwrap_error_body(&body)
      );
    }

    let byte_stream = resp.bytes_stream().map_err(std::io::Error::other);
    Ok(BufReader::new(StreamReader::new(byte_stream)))
  }
}
