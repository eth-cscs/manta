//! `manta console` — interactive raw-mode loop bridging local
//! stdin/stdout to the remote console streams. Shared by the `node`
//! and `target-ansible` subcommands.

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;
use futures::StreamExt;
use std::io::IsTerminal;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

/// Dispatch `manta console` subcommands (node, target-ansible).
pub async fn handle_console(
  cli_console: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_console.subcommand() {
    Some(("node", m)) => {
      if !std::io::stdout().is_terminal() {
        bail!("This command needs to run in interactive mode");
      }
      let xname = m.req_str("XNAME")?;

      let (cols, rows) = crossterm::terminal::size()?;
      let (a_input, a_output) = MantaClient::from_app_ctx(ctx, Some(&token))?
        .console_node(xname, cols, rows)
        .await?;
      let result = run_console_loop(a_input, a_output).await;
      handle_console_result(result);
    }
    Some(("target-ansible", m)) => {
      if !std::io::stdout().is_terminal() {
        bail!("This command needs to run in interactive mode");
      }
      let session_name = m.req_str("SESSION_NAME")?;

      let (cols, rows) = crossterm::terminal::size()?;
      let (a_input, a_output) = MantaClient::from_app_ctx(ctx, Some(&token))?
        .console_session(session_name, cols, rows)
        .await?;
      let result = run_console_loop(a_input, a_output).await;
      handle_console_result(result);
    }
    Some((other, _)) => bail!("Unknown 'console' subcommand: {other}"),
    None => bail!("No 'console' subcommand provided"),
  }
  Ok(())
}

/// Run an interactive console session using the given
/// input (write) and output (read) streams.
///
/// Enables raw mode, bridges stdin/stdout with the
/// remote streams, and disables raw mode on exit.
/// Callers should use [`handle_console_result`] to
/// process the return value.
pub async fn run_console_loop(
  a_input: Box<dyn AsyncWrite + Unpin>,
  a_output: Box<dyn AsyncRead + Unpin>,
) -> Result<(), anyhow::Error> {
  let mut stdin = tokio_util::io::ReaderStream::new(tokio::io::stdin());
  let mut stdout = tokio::io::stdout();

  let mut output = tokio_util::io::ReaderStream::new(a_output);
  let mut input = a_input;

  crossterm::terminal::enable_raw_mode()?;

  loop {
    tokio::select! {
        message = stdin.next() => {
            match message {
                Some(Ok(message)) => {
                    input.write_all(&message).await?;
                },
                Some(Err(message)) => {
                   crossterm::terminal::disable_raw_mode()?;
                   tracing::error!(
                       "Console stdin {:?}",
                       &message
                   );
                   break
                },
                None => {
                    crossterm::terminal::disable_raw_mode()?;
                    tracing::info!(
                        "NONE (No input): Console stdin"
                    );
                    break
                },
            }
        },

        message = output.next() => {
            match message {
                Some(Ok(message)) => {
                    stdout.write_all(&message).await?;
                    stdout.flush().await?;
                },
                Some(Err(message)) => {
                   crossterm::terminal::disable_raw_mode()?;
                   tracing::error!(
                       "Console stdout: {:?}",
                       &message
                   );
                   break
                },
                None => {
                    crossterm::terminal::disable_raw_mode()?;
                    tracing::info!("Exit console");
                    break
                },
            }
        },
    };
  }

  crossterm::terminal::disable_raw_mode()?;

  Ok(())
}

/// Handle the result of [`run_console_loop`], ensuring
/// raw mode is always disabled.
pub fn handle_console_result(result: Result<(), anyhow::Error>) {
  match result {
    Ok(()) => {
      if let Err(e) = crossterm::terminal::disable_raw_mode() {
        tracing::warn!("Failed to disable terminal raw mode: {}", e);
      }
      tracing::info!("Console closed");
    }
    Err(error) => {
      if let Err(e) = crossterm::terminal::disable_raw_mode() {
        tracing::warn!("Failed to disable terminal raw mode: {}", e);
      }
      tracing::error!("{:?}", error);
    }
  }
}
