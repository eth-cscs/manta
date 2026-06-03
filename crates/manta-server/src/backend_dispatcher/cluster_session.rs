//! Dispatches `ClusterSessionTrait` (BOS session) methods to csm-rs or ochami-rs.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::bos::ClusterSessionTrait,
  types::{self, bos::session::BosSession},
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl ClusterSessionTrait for StaticBackendDispatcher {
  async fn post_template_session(
    &self,
    shasta_token: &str,
    bos_session: types::bos::session::BosSession,
  ) -> Result<BosSession, Error> {
    dispatch!(self, post_template_session, shasta_token, bos_session)
  }
}
