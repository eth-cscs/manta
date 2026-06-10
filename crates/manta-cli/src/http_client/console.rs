//! HAND-ROLLED — not generated from the OpenAPI spec.
//!
//! WebSocket console endpoints for nodes and CFS sessions. Each
//! public method (`console_node`, `console_session`) builds the
//! `ws://` / `wss://` URL and delegates to `connect_console_ws`,
//! which drives the HTTP-upgrade handshake (via tungstenite), spawns
//! a bridge task, and returns a pair of async pipes for stdin/stdout.
//!
//! ## Why not auto-generated
//!
//! progenitor models endpoints as one-shot request/response pairs
//! that deserialize a JSON body. The HTTP-upgrade dance, the
//! bidirectional message loop, and the `Box<dyn AsyncWrite/AsyncRead>`
//! return shape all fall outside that model. The progenitor-generated
//! client does emit `console_node_ws` / `console_session_ws` stubs,
//! but they expect a pre-built upgrade response — they don't drive
//! the upgrade themselves. The CLI needs the bidirectional stream,
//! so we hand-roll the upgrade here and read the bearer token off
//! the wrapper struct instead of taking it as a parameter.
//!
//! Adding a new WebSocket endpoint? Hand-roll it here. Everything
//! else should be added to the server with `#[utoipa::path(...)]`
//! and consumed through the regenerated `client.openapi.*` methods.

use anyhow::Context;

use super::{MantaClient, ws_base_url};

impl MantaClient {
  /// Open a WebSocket console to a node and return async I/O streams.
  ///
  /// The returned `AsyncWrite` carries terminal stdin to the server; the
  /// returned `AsyncRead` delivers console output back to the terminal.
  pub async fn console_node(
    &self,
    xname: &str,
    cols: u16,
    rows: u16,
  ) -> anyhow::Result<(
    Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
    Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  )> {
    let url = format!(
      "{}/nodes/{}/console?cols={}&rows={}",
      ws_base_url(self.base_url()),
      xname,
      cols,
      rows,
    );
    self.connect_console_ws(&url).await
  }

  /// Open a WebSocket console to a CFS session container.
  pub async fn console_session(
    &self,
    session_name: &str,
    cols: u16,
    rows: u16,
  ) -> anyhow::Result<(
    Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
    Box<dyn tokio::io::AsyncRead + Unpin + Send>,
  )> {
    let url = format!(
      "{}/sessions/{}/console?cols={}&rows={}",
      ws_base_url(self.base_url()),
      session_name,
      cols,
      rows,
    );
    self.connect_console_ws(&url).await
  }

  /// Connect to a WebSocket URL with bearer auth and return stdin/stdout pipes.
  ///
  /// Spawns a background task that bridges between the WebSocket and two
  /// `tokio::io::duplex` pipes. The caller receives:
  /// - an `AsyncWrite` to write terminal stdin (sent as Binary WS frames)
  /// - an `AsyncRead` to read console output (received as Binary WS frames)
  async fn connect_console_ws(
    &self,
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

    let token = self.token.as_deref().context(
      "WebSocket console requires an authenticated client; construct \
       MantaClient with Some(token)",
    )?;

    let mut req = url.into_client_request().context("Invalid WebSocket URL")?;
    req.headers_mut().insert(
      "Authorization",
      HeaderValue::from_str(&format!("Bearer {token}"))
        .context("Invalid token header value")?,
    );
    req.headers_mut().insert(
      "X-Manta-Site",
      HeaderValue::from_str(self.site_name())
        .context("Invalid site-name header value")?,
    );

    let (ws_stream, _) =
      tokio_tungstenite::connect_async(req).await.map_err(|e| {
        // tungstenite reports connect-level failures as `Io(io_err)`;
        // surface those as the same "cannot reach manta server"
        // message the HTTP helpers use so the operator sees one
        // consistent error regardless of which subcommand they ran.
        let unreachable = matches!(
          &e,
          tokio_tungstenite::tungstenite::Error::Io(io_err)
            if matches!(
              io_err.kind(),
              std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::NotFound
            )
        );
        if unreachable {
          anyhow::Error::new(e).context(self.unreachable_server_msg())
        } else {
          anyhow::Error::new(e).context("WebSocket connection failed")
        }
      })?;

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
