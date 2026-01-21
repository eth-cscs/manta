
use manta_backend_dispatcher::{
  error::Error,
  interfaces::bss::BootParametersTrait,
  types::bss::BootParameters,
};

use StaticBackendDispatcher::*;


use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl BootParametersTrait for StaticBackendDispatcher {
  async fn get_all_bootparameters(
    &self,
    auth_token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    match self {
      CSM(b) => b.get_all_bootparameters(auth_token).await,
      OCHAMI(b) => b.get_all_bootparameters(auth_token).await,
    }
  }

  async fn get_bootparameters(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    match self {
      CSM(b) => b.get_bootparameters(auth_token, nodes).await,
      OCHAMI(b) => b.get_bootparameters(auth_token, nodes).await,
    }
  }

  async fn add_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.add_bootparameters(auth_token, boot_parameters).await,
      OCHAMI(b) => b.add_bootparameters(auth_token, boot_parameters).await,
    }
  }

  async fn update_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.update_bootparameters(auth_token, boot_parameters).await,
      OCHAMI(b) => b.update_bootparameters(auth_token, boot_parameters).await,
    }
  }

  async fn delete_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<String, Error> {
    match self {
      CSM(b) => b.delete_bootparameters(auth_token, boot_parameters).await,
      OCHAMI(b) => b.delete_bootparameters(auth_token, boot_parameters).await,
    }
  }
}
