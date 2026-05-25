//! Dispatches `SatTrait` (SAT file application) methods to csm-rs or
//! ochami-rs.
//!
//! Both backend variants return the same tuple
//! `(Vec<CfsConfigurationResponse>, Vec<Image>, Vec<BosSessionTemplate>,
//! Vec<BosSession>)` so the dispatch is a straight `match`/`.await`.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::apply_sat_file::{ApplySatFileParams, SatTrait},
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
}
