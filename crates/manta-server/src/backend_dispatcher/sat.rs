//! Dispatches `SatTrait` (SAT file application) methods to csm-rs or
//! ochami-rs.
//!
//! Each method just forwards through the `dispatch!` macro so the
//! `CSM` / `OCHAMI` variant's own trait impl is called. Without these
//! explicit forwards the calls would fall through to `SatTrait`'s
//! default "not implemented" impls.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::apply_sat_file::{
    ApplyConfigurationParams, ApplyImageParams, ApplySatFileParams,
    ApplySessionTemplateParams, SatTrait,
  },
  types::{
    bos::{session::BosSession, session_template::BosSessionTemplate},
    cfs::cfs_configuration_response::CfsConfigurationResponse,
    ims::Image,
  },
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl SatTrait for StaticBackendDispatcher {
  async fn apply_sat_file(
    &self,
    params: ApplySatFileParams<'_>,
  ) -> Result<
    (
      Vec<CfsConfigurationResponse>,
      Vec<Image>,
      Vec<BosSessionTemplate>,
      Vec<BosSession>,
    ),
    Error,
  > {
    dispatch!(self, apply_sat_file, params)
  }

  async fn apply_configuration(
    &self,
    params: ApplyConfigurationParams<'_>,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(self, apply_configuration, params)
  }

  async fn apply_image(
    &self,
    params: ApplyImageParams<'_>,
  ) -> Result<Image, Error> {
    dispatch!(self, apply_image, params)
  }

  async fn apply_session_template(
    &self,
    params: ApplySessionTemplateParams<'_>,
  ) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
    dispatch!(self, apply_session_template, params)
  }
}
