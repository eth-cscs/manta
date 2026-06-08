//! BSS boot-parameter backend methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::types::bss::BootParameters;

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// List BSS boot parameters for all nodes.
  pub async fn get_all_bootparameters(
    &self,
    token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    self.backend.get_all_bootparameters(token).await
  }

  /// Fetch BSS boot parameters for the given xnames.
  pub async fn get_bootparameters(
    &self,
    token: &str,
    xnames: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    self.backend.get_bootparameters(token, xnames).await
  }

  /// Delete BSS boot parameters by host list.
  pub async fn delete_bootparameters(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_bootparameters(token, boot_parameters)
      .await
      .map(|_| ())
  }

  /// Add (create) BSS boot parameters.
  pub async fn add_bootparameters(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    self
      .backend
      .add_bootparameters(token, boot_parameters)
      .await
  }

  /// Update existing BSS boot parameters.
  pub async fn update_bootparameters(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    self
      .backend
      .update_bootparameters(token, boot_parameters)
      .await
  }
}
