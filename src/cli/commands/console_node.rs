use crate::{
  cli::commands::console_common,
  common::{self, authentication::get_api_token},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::console::ConsoleTrait, types::K8sDetails,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  xname: &str,
  k8s: &K8sDetails,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  // Convert user input to xname
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
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
      &shasta_token,
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
