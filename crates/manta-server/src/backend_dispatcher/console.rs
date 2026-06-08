//! `ConsoleTrait` impl for `StaticBackendDispatcher`.

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
  ) -> Result<
    (
      Box<dyn AsyncWrite + Unpin + Send>,
      Box<dyn AsyncRead + Unpin + Send>,
    ),
    Error,
  > {
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
  ) -> Result<
    (
      Box<dyn AsyncWrite + Unpin + Send>,
      Box<dyn AsyncRead + Unpin + Send>,
    ),
    Error,
  > {
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
