//! [`ConsoleTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Opens a PTY-style attachment to a CSM `cray-console-operator`
//! pod (per-node) or the CFS session pod (per-session) via the
//! Kubernetes exec subprotocol. Ochami's `impl ConsoleTrait for
//! Ochami {}` uses the trait default, so both methods on the Ochami
//! branch return [`Error::Message`] ("Attach to … console command
//! not implemented for this backend").

use super::*;

impl ConsoleTrait for StaticBackendDispatcher {
  type T = Box<dyn AsyncWrite + Unpin + Send>;
  type U = Box<dyn AsyncRead + Unpin + Send>;

  async fn attach_to_node_console(
    &self,
    token: &str,
    site_name: &str,
    xname: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Self::T, Self::U), Error> {
    dispatch!(
      self,
      attach_to_node_console,
      token,
      site_name,
      xname,
      width,
      height,
      k8s
    )
  }

  async fn attach_to_session_console(
    &self,
    token: &str,
    site_name: &str,
    session_name: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Self::T, Self::U), Error> {
    dispatch!(
      self,
      attach_to_session_console,
      token,
      site_name,
      session_name,
      width,
      height,
      k8s
    )
  }
}
