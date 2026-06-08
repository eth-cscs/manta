//! `ClusterSessionTrait` (BOS session) impl for `StaticBackendDispatcher`.

use super::*;

impl ClusterSessionTrait for StaticBackendDispatcher {
  async fn post_template_session(
    &self,
    token: &str,
    bos_session: types::bos::session::BosSession,
  ) -> Result<BosSession, Error> {
    dispatch!(self, post_template_session, token, bos_session)
  }
}
