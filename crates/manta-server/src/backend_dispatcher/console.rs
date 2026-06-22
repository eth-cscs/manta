//! `ConsoleTrait` impl for `StaticBackendDispatcher`.

use manta_backend_dispatcher::interfaces::console::{
  ConsoleAttachment, TermSize,
};

use super::*;

impl ConsoleTrait for StaticBackendDispatcher {
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
