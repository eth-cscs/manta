
use manta_backend_dispatcher::{
  error::Error,
  interfaces::console::ConsoleTrait,
  types::K8sDetails,
};

use StaticBackendDispatcher::*;
use tokio::io::{AsyncRead, AsyncWrite};


use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl ConsoleTrait for StaticBackendDispatcher {
  type T = Box<dyn AsyncWrite + Unpin>;
  type U = Box<dyn AsyncRead + Unpin>;

  async fn attach_to_node_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    xname: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Box<dyn AsyncWrite + Unpin>, Box<dyn AsyncRead + Unpin>), Error>
  {
    match self {
      CSM(b) => {
        b.attach_to_node_console(
          shasta_token,
          site_name,
          xname,
          width,
          height,
          k8s,
        )
        .await
      }
      OCHAMI(b) => {
        b.attach_to_node_console(
          shasta_token,
          site_name,
          xname,
          width,
          height,
          k8s,
        )
        .await
      }
    }
  }

  async fn attach_to_session_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    session_name: &str,
    width: u16,
    height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Box<dyn AsyncWrite + Unpin>, Box<dyn AsyncRead + Unpin>), Error>
  {
    match self {
      CSM(b) => {
        b.attach_to_session_console(
          shasta_token,
          site_name,
          session_name,
          width,
          height,
          k8s,
        )
        .await
      }
      OCHAMI(b) => {
        b.attach_to_session_console(
          shasta_token,
          site_name,
          session_name,
          width,
          height,
          k8s,
        )
        .await
      }
    }
  }
}
