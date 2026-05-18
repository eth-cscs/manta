//! WebSocket console handlers (node + session).

use axum::{
  Json,
  extract::{
    Path, Query,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  http::StatusCode,
  response::IntoResponse,
};
use futures::StreamExt;
use manta_backend_dispatcher::{
  interfaces::console::ConsoleTrait,
  types::{K8sAuth, K8sDetails},
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use utoipa::IntoParams;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, require_k8s_url, require_vault,
  to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// WS /api/v1/nodes/{xname}/console — Interactive node console
// ---------------------------------------------------------------------------

/// Query parameters for WebSocket console endpoints (initial terminal size).
#[derive(Deserialize, IntoParams)]
pub struct ConsoleQuery {
  /// Initial terminal width in columns (default 80).
  #[serde(default = "default_cols")]
  pub cols: u16,
  /// Initial terminal height in rows (default 24).
  #[serde(default = "default_rows")]
  pub rows: u16,
}

fn default_cols() -> u16 {
  80
}
fn default_rows() -> u16 {
  24
}

/// `WS /api/v1/nodes/{xname}/console` — attach an interactive PTY console to a node via WebSocket.
#[utoipa::path(get, path = "/nodes/{xname}/console", tag = "console",
  params(("xname" = String, Path, description = "Node xname"), ConsoleQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 101, description = "WebSocket upgrade"),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured",    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all, fields(xname = %xname))]
pub async fn console_node_ws(
  ctx: RequestCtx,
  Path(xname): Path<String>,
  Query(q): Query<ConsoleQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let (k8s_api_url, vault_base_url) = {
    let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
    let k = require_k8s_url(infra.k8s_api_url)?.to_string();
    let v = require_vault(infra.vault_base_url)?.to_string();
    (k, v)
  };

  let k8s = K8sDetails {
    api_url: k8s_api_url,
    authentication: K8sAuth::Vault {
      base_url: vault_base_url,
    },
  };

  let timeout = state.console_inactivity_timeout;
  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for node {xname}");
    if let Some(site) = state.sites.get(&site_name) {
      match site
        .backend
        .attach_to_node_console(
          &token, &site_name, &xname, q.cols, q.rows, &k8s,
        )
        .await
      {
        Ok((console_in, console_out)) => {
          run_console_bridge(socket, console_in, console_out, timeout).await;
          tracing::info!("WebSocket console closed for node {xname}");
        }
        Err(e) => {
          tracing::error!("Failed to attach to node console {xname}: {e:#}");
        }
      }
    }
  }))
}

// ---------------------------------------------------------------------------
// WS /api/v1/sessions/{name}/console — Interactive CFS session console
// ---------------------------------------------------------------------------

/// `WS /api/v1/sessions/{name}/console` — attach an interactive PTY console to a CFS session pod via WebSocket.
#[utoipa::path(get, path = "/sessions/{name}/console", tag = "console",
  params(("name" = String, Path, description = "Session name"), ConsoleQuery, SiteHeader),
  security(("bearerAuth" = [])),
  responses(
    (status = 101, description = "WebSocket upgrade"),
    (status = 401, description = "Unauthorized",                   body = ErrorResponse),
    (status = 500, description = "Internal error",                 body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured",    body = ErrorResponse),
  )
)]
#[tracing::instrument(skip_all, fields(session = %name))]
pub async fn console_session_ws(
  ctx: RequestCtx,
  Path(name): Path<String>,
  Query(q): Query<ConsoleQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let (state, token, site_name) = ctx.into_parts();
  let (k8s_api_url, vault_base_url) = {
    let infra = state.infra_context(&site_name).map_err(to_handler_error)?;
    let k = require_k8s_url(infra.k8s_api_url)?.to_string();
    let v = require_vault(infra.vault_base_url)?.to_string();
    service::session::validate_console_session(&infra, &token, &name)
      .await
      .map_err(to_handler_error)?;
    (k, v)
  };

  let k8s = K8sDetails {
    api_url: k8s_api_url,
    authentication: K8sAuth::Vault {
      base_url: vault_base_url,
    },
  };

  let timeout = state.console_inactivity_timeout;
  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for session {name}");
    if let Some(site) = state.sites.get(&site_name) {
      match site
        .backend
        .attach_to_session_console(
          &token, &site_name, &name, q.cols, q.rows, &k8s,
        )
        .await
      {
        Ok((console_in, console_out)) => {
          run_console_bridge(socket, console_in, console_out, timeout).await;
          tracing::info!("WebSocket console closed for session {name}");
        }
        Err(e) => {
          tracing::error!("Failed to attach to session console {name}: {e:#}");
        }
      }
    }
  }))
}

/// Bridge a WebSocket connection to a console's stdin/stdout streams.
///
/// - Binary and text WS frames are forwarded as raw bytes to console stdin.
/// - Text frames matching `{"type":"resize","cols":N,"rows":N}` are silently
///   consumed (dynamic resize is not yet supported by the ConsoleTrait).
/// - Console stdout is forwarded as Binary WS frames.
/// - Either side closing or erroring terminates the bridge.
/// - The bridge closes automatically after `inactivity_timeout` of silence
///   from the client, releasing the Kubernetes pod attachment.
async fn run_console_bridge(
  mut socket: WebSocket,
  mut console_in: Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
  console_out: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  inactivity_timeout: std::time::Duration,
) {
  let mut out_stream = tokio_util::io::ReaderStream::new(console_out);
  let mut deadline = tokio::time::Instant::now() + inactivity_timeout;

  loop {
    tokio::select! {
      msg = socket.recv() => {
        match msg {
          Some(Ok(Message::Binary(data))) => {
            deadline = tokio::time::Instant::now() + inactivity_timeout;
            if console_in.write_all(&data).await.is_err() { break; }
          }
          Some(Ok(Message::Text(text))) => {
            deadline = tokio::time::Instant::now() + inactivity_timeout;
            // Consume resize control messages silently; forward everything else.
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text)
              && v.get("type").and_then(|t| t.as_str()) == Some("resize")
            {
              continue;
            }
            if console_in.write_all(text.as_bytes()).await.is_err() { break; }
          }
          Some(Ok(Message::Close(_))) | None => break,
          Some(Ok(_)) => {} // Ping/Pong handled by axum automatically
          Some(Err(_)) => break,
        }
      }
      chunk = out_stream.next() => {
        match chunk {
          Some(Ok(data)) => {
            if socket.send(Message::Binary(data)).await.is_err() { break; }
          }
          Some(Err(_)) | None => break,
        }
      }
      _ = tokio::time::sleep_until(deadline) => {
        tracing::warn!("Console session idle for {:?}, closing", inactivity_timeout);
        break;
      }
    }
  }
}
