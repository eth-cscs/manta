//! [`ClusterSessionTrait`] (BOS session) impl for
//! [`StaticBackendDispatcher`].
//!
//! Forwards to `POST /apis/bos/v2/sessions`. Ochami uses the trait
//! default and returns [`Error::Message`] ("not implemented for this
//! backend").

use super::*;

impl ClusterSessionTrait for StaticBackendDispatcher {
  /// Submit a BOS session derived from an existing template. Returns
  /// the persisted [`BosSession`] (the backend assigns the id).
  async fn post_template_session(
    &self,
    token: &str,
    bos_session: types::bos::session::BosSession,
  ) -> Result<BosSession, Error> {
    dispatch!(self, post_template_session, token, bos_session)
  }
}
