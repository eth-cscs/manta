use futures::{channel::mpsc::Sender, SinkExt};

use kube::api::TerminalSize;

use tokio::signal;

pub async fn handle_terminal_size(
  mut channel: Sender<TerminalSize>,
) -> Result<(), anyhow::Error> {
  let (width, height) = crossterm::terminal::size()?;
  channel.send(TerminalSize { height, width }).await?;

  // create a stream to catch SIGWINCH signal
  let mut sig =
    signal::unix::signal(signal::unix::SignalKind::window_change())?;
  loop {
    if (sig.recv().await).is_none() {
      return Ok(());
    }

    let (width, height) = crossterm::terminal::size()?;
    channel.send(TerminalSize { height, width }).await?;
  }
}
