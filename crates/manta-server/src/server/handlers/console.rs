//! WebSocket console handlers (interactive PTY attachments).
//!
//! - `WS /api/v1/nodes/{xname}/console`    → [`console_node_ws`] —
//!   attach to a single node's console.
//! - `WS /api/v1/sessions/{name}/console`  → [`console_session_ws`] —
//!   attach to a CFS-session-spawned ephemeral environment.
//!
//! Both require Vault (for Kubernetes credentials) and the per-site
//! `k8s_api_url` — the handlers return `501 Not Implemented` if
//! either is missing (see [`super::require_vault`] /
//! [`super::require_k8s_url`]). Idle sessions are reaped after
//! [`crate::server::ServerState::console_inactivity_timeout`].

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
  interfaces::console::{ConsoleAttachment, ConsoleTrait, TermSize},
  types::{K8sAuth, K8sDetails},
};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;

use super::{
  ErrorResponse, RequestCtx, SiteHeader, require_k8s_url, require_vault,
  to_handler_error,
};
use crate::service;

// ---------------------------------------------------------------------------
// WS /api/v1/nodes/{xname}/console — Interactive node console
// ---------------------------------------------------------------------------

pub use manta_shared::types::api::queries::ConsoleQuery;

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
  // Read what we need from the borrowed infra and authorize the xname;
  // the borrow ends with the block.
  let (k8s_api_url, vault_base_url, timeout) = {
    let infra = ctx.infra();
    let k = require_k8s_url(infra.k8s_api_url)?.to_string();
    let v = require_vault(infra.vault_base_url)?.to_string();
    // Authorization: caller must have group access to this xname.
    // Without this an authenticated user with any group could open
    // an interactive PTY on any node in the cluster.
    service::authorization::validate_user_group_members_access(
      &infra,
      &ctx.token,
      std::slice::from_ref(&xname),
    )
    .await
    .map_err(to_handler_error)?;
    (k, v, ctx.state.console_inactivity_timeout)
  };

  let k8s = K8sDetails {
    api_url: k8s_api_url,
    authentication: K8sAuth::Vault {
      base_url: vault_base_url,
    },
  };

  // Move owned state into the spawned WebSocket task. Cannot use
  // `ctx.infra()` inside the closure because it borrows from ctx.
  let RequestCtx {
    state,
    token,
    site_name,
  } = ctx;

  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for node {xname}");
    if let Some(site) = state.sites.get(&site_name) {
      match site
        .backend
        .attach_to_node_console(
          &token,
          &site_name,
          &xname,
          TermSize {
            width: q.cols,
            height: q.rows,
          },
          &k8s,
        )
        .await
      {
        Ok(ConsoleAttachment {
          stdin,
          stdout,
          resize,
        }) => {
          run_console_bridge(socket, stdin, stdout, resize, timeout).await;
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
  // Validate vault/k8s presence, then authorize the caller against
  // the session's target groups, then check session liveness.
  let (k8s_api_url, vault_base_url, timeout) = {
    let infra = ctx.infra();
    let k = require_k8s_url(infra.k8s_api_url)?.to_string();
    let v = require_vault(infra.vault_base_url)?.to_string();
    // Authorization: the caller's accessible groups must overlap the
    // session's target.groups. validate_console_session does NOT do
    // this check (only "is image-type and running"), so without this
    // call any authenticated user could attach to any image session.
    service::session::validate_session_access(&infra, &ctx.token, &name)
      .await
      .map_err(to_handler_error)?;
    service::session::validate_console_session(&infra, &ctx.token, &name)
      .await
      .map_err(to_handler_error)?;
    (k, v, ctx.state.console_inactivity_timeout)
  };

  let k8s = K8sDetails {
    api_url: k8s_api_url,
    authentication: K8sAuth::Vault {
      base_url: vault_base_url,
    },
  };

  // Move owned state into the spawned WebSocket task.
  let RequestCtx {
    state,
    token,
    site_name,
  } = ctx;

  Ok(ws.on_upgrade(move |socket| async move {
    tracing::info!("WebSocket console opened for session {name}");
    if let Some(site) = state.sites.get(&site_name) {
      match site
        .backend
        .attach_to_session_console(
          &token,
          &site_name,
          &name,
          TermSize {
            width: q.cols,
            height: q.rows,
          },
          &k8s,
        )
        .await
      {
        Ok(ConsoleAttachment {
          stdin,
          stdout,
          resize,
        }) => {
          run_console_bridge(socket, stdin, stdout, resize, timeout).await;
          tracing::info!("WebSocket console closed for session {name}");
        }
        Err(e) => {
          tracing::error!("Failed to attach to session console {name}: {e:#}");
        }
      }
    }
  }))
}

/// Minimal abstraction over the WebSocket half of the bridge so the
/// timeout / message-handling loop can be unit-tested against an
/// in-process mock channel. Cancel-safety follows from the underlying
/// `WebSocket::recv` / `WebSocket::send` (both documented cancel-safe)
/// and from `tokio::sync::mpsc` for the test impl.
#[allow(async_fn_in_trait)]
trait ConsoleSocket: Send + Unpin {
  async fn recv(&mut self) -> Option<Result<Message, axum::Error>>;
  async fn send(&mut self, msg: Message) -> Result<(), axum::Error>;
}

impl ConsoleSocket for WebSocket {
  async fn recv(&mut self) -> Option<Result<Message, axum::Error>> {
    WebSocket::recv(self).await
  }
  async fn send(&mut self, msg: Message) -> Result<(), axum::Error> {
    WebSocket::send(self, msg).await
  }
}

/// Bridge a WebSocket connection to a console's stdin/stdout streams.
///
/// - Binary and text WS frames are forwarded as raw bytes to console stdin.
/// - Text frames matching `{"type":"resize","cols":N,"rows":N}` are parsed
///   and forwarded to `resize`; the backend implementation pushes them
///   onto the underlying transport's resize channel (k8s exec subprotocol
///   channel 4). The bytes are not forwarded to stdin.
/// - Console stdout is forwarded as Binary WS frames.
/// - Either side closing or erroring terminates the bridge.
/// - The bridge closes automatically after `inactivity_timeout` of silence
///   from the client, releasing the Kubernetes pod attachment.
async fn run_console_bridge<S: ConsoleSocket>(
  mut socket: S,
  mut console_in: Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
  console_out: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  resize: Sender<TermSize>,
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
            // Resize control messages are parsed and forwarded to the
            // backend's resize channel; everything else goes to stdin.
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text)
              && v.get("type").and_then(|t| t.as_str()) == Some("resize")
            {
              let cols = v.get("cols").and_then(|c| c.as_u64()).unwrap_or(0);
              let rows = v.get("rows").and_then(|r| r.as_u64()).unwrap_or(0);
              if cols > 0 && rows > 0 && cols <= u64::from(u16::MAX) && rows <= u64::from(u16::MAX) {
                #[allow(clippy::cast_possible_truncation)]
                let size = TermSize {
                  width: cols as u16,
                  height: rows as u16,
                };
                // Drop the event on a full channel rather than blocking
                // the bridge; resize is idempotent state, not a sequence
                // of deltas.
                let _ = resize.try_send(size);
              }
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
      () = tokio::time::sleep_until(deadline) => {
        tracing::warn!("Console session idle for {:?}, closing", inactivity_timeout);
        break;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  //! Tests for [`run_console_bridge`] using an in-process mock socket
  //! and tokio's paused-time scheduler.
  //!
  //! Each test follows the same pattern: spawn the bridge with a
  //! `MockSocket` and a sink/empty pair for the console streams,
  //! drive it with `tokio::time::advance` + occasional message sends
  //! over the test-side handles, then assert on whether the bridge
  //! handle has resolved (loop exited) or not.

  use super::*;
  use std::pin::Pin;
  use std::sync::{Arc, Mutex};
  use std::task::{Context, Poll};
  use std::time::Duration;
  use tokio::sync::mpsc;

  /// Records every byte written through it so a test can assert what
  /// (if anything) the bridge forwarded to console stdin.
  struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

  impl tokio::io::AsyncWrite for CaptureWriter {
    fn poll_write(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
      buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
      self.0.lock().unwrap().extend_from_slice(buf);
      Poll::Ready(Ok(buf.len()))
    }
    fn poll_flush(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
      Poll::Ready(Ok(()))
    }
    fn poll_shutdown(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
      Poll::Ready(Ok(()))
    }
  }

  /// An AsyncRead that never yields. Use as `console_out` in tests
  /// that don't want the server-side branch of the bridge to fire —
  /// `tokio::io::empty()` is unsuitable here because it returns EOF on
  /// the first read, which exits the bridge via the `None` arm.
  struct PendingReader;

  impl tokio::io::AsyncRead for PendingReader {
    fn poll_read(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
      _: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
      Poll::Pending
    }
  }

  /// In-process stand-in for axum's `WebSocket` used only in tests.
  /// `rx` is driven by the test (simulated client → server frames);
  /// `tx` is observed by the test (simulated server → client frames).
  struct MockSocket {
    rx: mpsc::UnboundedReceiver<Result<Message, axum::Error>>,
    tx: mpsc::UnboundedSender<Message>,
  }

  impl ConsoleSocket for MockSocket {
    async fn recv(&mut self) -> Option<Result<Message, axum::Error>> {
      self.rx.recv().await
    }
    async fn send(&mut self, msg: Message) -> Result<(), axum::Error> {
      self.tx.send(msg).map_err(axum::Error::new)
    }
  }

  fn new_mock_socket() -> (
    MockSocket,
    mpsc::UnboundedSender<Result<Message, axum::Error>>,
    mpsc::UnboundedReceiver<Message>,
  ) {
    let (in_tx, in_rx) = mpsc::unbounded_channel();
    let (out_tx, out_rx) = mpsc::unbounded_channel();
    (
      MockSocket {
        rx: in_rx,
        tx: out_tx,
      },
      in_tx,
      out_rx,
    )
  }

  /// Wait for either the bridge to finish or `cap` of paused time to
  /// elapse. Returns `true` if the bridge exited within the budget.
  async fn bridge_exited_within(
    handle: &mut tokio::task::JoinHandle<()>,
    cap: Duration,
  ) -> bool {
    tokio::select! {
      _ = handle => true,
      () = tokio::time::sleep(cap) => false,
    }
  }

  #[tokio::test(start_paused = true)]
  async fn inactivity_timeout_fires_when_no_traffic() {
    let (socket, _in_tx, _out_rx) = new_mock_socket();
    let console_in = Box::new(tokio::io::sink());
    let console_out = Box::new(PendingReader);
    let (resize_tx, _resize_rx) = mpsc::channel(8);

    let mut handle = tokio::spawn(async move {
      run_console_bridge(
        socket,
        console_in,
        console_out,
        resize_tx,
        Duration::from_secs(60),
      )
      .await;
    });

    // Just before the deadline — bridge should still be alive.
    assert!(
      !bridge_exited_within(&mut handle, Duration::from_secs(59)).await,
      "bridge exited before the 60s inactivity timeout"
    );
    // Cross the deadline — bridge should exit.
    assert!(
      bridge_exited_within(&mut handle, Duration::from_secs(5)).await,
      "bridge did not exit after the inactivity timeout"
    );
  }

  #[tokio::test(start_paused = true)]
  async fn client_binary_message_resets_deadline() {
    let (socket, in_tx, _out_rx) = new_mock_socket();
    let console_in = Box::new(tokio::io::sink());
    let console_out = Box::new(PendingReader);
    let (resize_tx, _resize_rx) = mpsc::channel(8);

    let mut handle = tokio::spawn(async move {
      run_console_bridge(
        socket,
        console_in,
        console_out,
        resize_tx,
        Duration::from_secs(60),
      )
      .await;
    });

    // At t≈59s send a binary frame — that resets the deadline to t+60.
    tokio::time::sleep(Duration::from_secs(59)).await;
    in_tx
      .send(Ok(Message::Binary(b"hi".to_vec().into())))
      .unwrap();
    // Yield so the bridge actually processes the message before we
    // advance time again.
    tokio::task::yield_now().await;

    // At original-t+90 (post-reset deadline is original-t+119) — still alive.
    assert!(
      !bridge_exited_within(&mut handle, Duration::from_secs(31)).await,
      "deadline was not reset by client binary message"
    );
    // Now well past the reset deadline (~original-t+125) — should exit.
    assert!(
      bridge_exited_within(&mut handle, Duration::from_secs(35)).await,
      "bridge did not exit after the reset deadline"
    );
  }

  #[tokio::test(start_paused = true)]
  async fn resize_text_forwards_to_resize_channel_and_resets_deadline() {
    // The test exercises three guarantees in one shot:
    //   - deadline resets on any client text frame (including resize)
    //   - the resize JSON itself is parsed, never written to stdin
    //   - parsed cols/rows are forwarded to the backend's resize channel
    let (socket, in_tx, _out_rx) = new_mock_socket();
    let written: Arc<Mutex<Vec<u8>>> = Default::default();
    let console_in = Box::new(CaptureWriter(written.clone()));
    let console_out = Box::new(PendingReader);
    let (resize_tx, mut resize_rx) = mpsc::channel(8);

    let mut handle = tokio::spawn(async move {
      run_console_bridge(
        socket,
        console_in,
        console_out,
        resize_tx,
        Duration::from_secs(60),
      )
      .await;
    });

    tokio::time::sleep(Duration::from_secs(59)).await;
    in_tx
      .send(Ok(Message::Text(
        r#"{"type":"resize","cols":120,"rows":40}"#.into(),
      )))
      .unwrap();
    tokio::task::yield_now().await;

    // Bridge still alive past the original deadline.
    assert!(
      !bridge_exited_within(&mut handle, Duration::from_secs(30)).await,
      "deadline was not reset by resize message"
    );
    // The resize JSON must not have been written to stdin.
    assert!(
      written.lock().unwrap().is_empty(),
      "resize text frame was forwarded to console stdin (should be parsed)"
    );
    // The parsed size must have been forwarded to the resize channel.
    let size = resize_rx.try_recv().expect(
      "resize message should have been forwarded to the resize channel",
    );
    assert_eq!(size.width, 120);
    assert_eq!(size.height, 40);

    handle.abort();
  }

  #[tokio::test(start_paused = true)]
  async fn client_close_exits_loop_immediately() {
    let (socket, in_tx, _out_rx) = new_mock_socket();
    let console_in = Box::new(tokio::io::sink());
    let console_out = Box::new(PendingReader);
    let (resize_tx, _resize_rx) = mpsc::channel(8);

    let mut handle = tokio::spawn(async move {
      run_console_bridge(
        socket,
        console_in,
        console_out,
        resize_tx,
        Duration::from_secs(3600),
      )
      .await;
    });

    in_tx.send(Ok(Message::Close(None))).unwrap();
    assert!(
      bridge_exited_within(&mut handle, Duration::from_secs(1)).await,
      "bridge did not exit on Close frame"
    );
  }

  #[tokio::test(start_paused = true)]
  async fn server_to_client_data_does_not_reset_deadline() {
    // Pin the *current* behaviour: server-side data flowing through
    // the bridge does NOT reset the inactivity deadline. The deadline
    // tracks CLIENT inactivity (no keystrokes), so an idle user
    // watching scrolling logs will still time out. If that policy
    // changes, this test makes the decision explicit.
    use tokio::io::AsyncReadExt;

    let (socket, _in_tx, mut out_rx) = new_mock_socket();
    let console_in = Box::new(tokio::io::sink());
    // A finite chunk of server bytes followed by Pending forever, so
    // the bridge forwards real data once but is never broken by EOF.
    // If the deadline DID reset on server data, the bridge would
    // stay alive past 60s; we assert the opposite below.
    let console_out =
      Box::new(std::io::Cursor::new(b"chunk".to_vec()).chain(PendingReader));
    let (resize_tx, _resize_rx) = mpsc::channel(8);

    let mut handle = tokio::spawn(async move {
      run_console_bridge(
        socket,
        console_in,
        console_out,
        resize_tx,
        Duration::from_secs(60),
      )
      .await;
    });

    // Drain whatever the bridge forwards so its `send` doesn't block.
    tokio::spawn(async move { while out_rx.recv().await.is_some() {} });

    // After ~65s of paused time with no CLIENT traffic, the deadline
    // (set at t=0) should have fired even though server data was
    // forwarded early on.
    assert!(
      bridge_exited_within(&mut handle, Duration::from_secs(65)).await,
      "server-to-client data should NOT keep the deadline alive"
    );
  }
}
