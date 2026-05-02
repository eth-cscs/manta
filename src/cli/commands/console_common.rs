use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

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
