//! [`ConsoleTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Opens a PTY-style attachment to a CSM `cray-console-operator`
//! pod (per-node) or the CFS session pod (per-session) via the
//! Kubernetes exec subprotocol. Ochami's `impl ConsoleTrait for
//! Ochami {}` uses the trait default, so both methods on the Ochami
//! branch return [`Error::Message`] ("Attach to … console command
//! not implemented for this backend").

use manta_backend_dispatcher::interfaces::console::{
  ConsoleAttachment, TermSize,
};

use super::*;

impl ConsoleTrait for StaticBackendDispatcher {
  /// Attach to `xname`'s console. Returns the
  /// stdin/stdout/resize handle; the caller drives the lifetime by
  /// dropping the channels.
  async fn attach_to_node_console(
    &self,
    token: &str,
    site_name: &str,
    xname: &str,
    initial_size: TermSize,
    k8s: &K8sDetails,
  ) -> Result<ConsoleAttachment, Error> {
    dispatch!(
      self,
      attach_to_node_console,
      token,
      site_name,
      xname,
      initial_size,
      k8s
    )
  }

  /// Attach to the Ansible-container console of a running CFS
  /// session, identified by `session_name`. Same handle shape as
  /// [`attach_to_node_console`](Self::attach_to_node_console).
  async fn attach_to_session_console(
    &self,
    token: &str,
    site_name: &str,
    session_name: &str,
    initial_size: TermSize,
    k8s: &K8sDetails,
  ) -> Result<ConsoleAttachment, Error> {
    dispatch!(
      self,
      attach_to_session_console,
      token,
      site_name,
      session_name,
      initial_size,
      k8s
    )
  }
}
