use manta_backend_dispatcher::{
  error::Error, interfaces::bss::BootParametersTrait,
  types::bss::BootParameters,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  xnames: &String,
  _nids: Option<&String>,
  _macs: Option<&String>,
  _params: Option<&String>,
  _kernel: Option<&String>,
  _initrd: Option<&String>,
) -> Result<Vec<BootParameters>, Error> {
  println!("Get boot parameters");

  let hosts: Vec<String> = xnames.split(',').map(String::from).collect();

  backend.get_bootparameters(shasta_token, &hosts).await
}
