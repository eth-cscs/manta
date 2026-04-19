use crate::{
  cli::commands::console_common,
  common,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::console::ConsoleTrait, types::K8sDetails,
};

/// Open an interactive serial console to a node.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  token: &str,
  xname: &str,
  k8s: &K8sDetails,
) -> Result<(), Error> {
  // Convert user input to xname
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    token,
    xname,
    false,
  )
  .await?;

  if xname_vec.len() != 1 {
    bail!(
      "The node to operate is not \
       valid. Nothing to do",
    );
  }

  let xname = xname_vec.first().context("xname list unexpectedly empty")?;

  log::info!("xname: {}", xname);

  let (width, height) = crossterm::terminal::size()?;

  let (a_input, a_output) = backend
    .attach_to_node_console(
      token,
      site_name,
      &xname.to_string(),
      width,
      height,
      k8s,
    )
    .await?;

  let result = console_common::run_console_loop(a_input, a_output).await;

  console_common::handle_console_result(result);

  Ok(())
}
