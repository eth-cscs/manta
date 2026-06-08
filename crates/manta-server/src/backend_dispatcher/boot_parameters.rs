//! `BootParametersTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl BootParametersTrait for StaticBackendDispatcher {
  async fn get_all_bootparameters(
    &self,
    auth_token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    dispatch!(self, get_all_bootparameters, auth_token)
  }

  async fn get_bootparameters(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    dispatch!(self, get_bootparameters, auth_token, nodes)
  }

  async fn add_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    dispatch!(self, add_bootparameters, auth_token, boot_parameters)
  }

  async fn update_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    dispatch!(self, update_bootparameters, auth_token, boot_parameters)
  }

  async fn delete_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<String, Error> {
    dispatch!(self, delete_bootparameters, auth_token, boot_parameters)
  }
}
