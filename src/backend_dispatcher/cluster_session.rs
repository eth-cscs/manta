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
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session: types::bos::session::BosSession,
  ) -> Result<BosSession, Error> {
    match self {
      CSM(b) => {
        b.post_template_session(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session,
        )
        .await
      }
      OCHAMI(b) => {
        b.post_template_session(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session,
        )
        .await
      }
    }
  }
}
