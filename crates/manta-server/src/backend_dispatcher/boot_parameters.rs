//! [`BootParametersTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the BSS (Boot Script Service) bootparameters API:
//! `GET/POST/PATCH/DELETE /apis/bss/boot/v1/bootparameters`. Both CSM
//! and Ochami implement these methods natively; no Ochami branch
//! returns the trait default.

use super::*;

impl BootParametersTrait for StaticBackendDispatcher {
  /// `GET /bootparameters` — every entry in BSS.
  async fn get_all_bootparameters(
    &self,
    auth_token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    dispatch!(self, get_all_bootparameters, auth_token)
  }

  /// `GET /bootparameters?hosts=...` — entries scoped to `nodes`.
  async fn get_bootparameters(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    dispatch!(self, get_bootparameters, auth_token, nodes)
  }

  /// `POST /bootparameters` — create a new entry.
  async fn add_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    dispatch!(self, add_bootparameters, auth_token, boot_parameters)
  }

  /// `PATCH /bootparameters` — merge `boot_parameters` into the
  /// existing entry for its hosts.
  async fn update_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    dispatch!(self, update_bootparameters, auth_token, boot_parameters)
  }

  /// `DELETE /bootparameters` — remove the entry. Returns the
  /// backend's response body verbatim.
  async fn delete_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<String, Error> {
    dispatch!(self, delete_bootparameters, auth_token, boot_parameters)
  }
}
