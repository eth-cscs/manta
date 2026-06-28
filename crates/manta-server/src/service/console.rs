//! Console-attach service wrappers.
//!
//! [`crate::server::common::app_context::InfraContext`] borrows the
//! backend and cannot outlive the handler frame. WebSocket upgrade
//! closures need owned (move-captured) state that is
//! `'static`-compatible, so these wrappers accept a cloned
//! [`crate::dispatcher::StaticBackendDispatcher`] instead of
//! `&InfraContext`.
//!
//! Handlers call [`crate::server::common::app_context::InfraContext::backend_clone`]
//! before the borrowed `infra` is dropped and move the result into the
//! WebSocket closure.
//! The closure then delegates here rather than traversing
//! `state.sites.get(&site_name).backend` directly.
//!
//! All authorization and vault/k8s URL checks must be performed in the
//! handler BEFORE the backend is cloned and the closure is spawned.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::console::{
  ConsoleAttachment, ConsoleTrait, TermSize,
};
use manta_backend_dispatcher::types::K8sDetails;

use crate::dispatcher::StaticBackendDispatcher;

/// Attach an interactive PTY console to a node.
///
/// Callers must validate group-member access for `xname` and resolve
/// vault/k8s URLs BEFORE calling this; see
/// [`crate::service::authorization::validate_user_group_members_access`].
pub async fn attach_to_node_console(
  backend: &StaticBackendDispatcher,
  token: &str,
  site_name: &str,
  xname: &str,
  term_size: TermSize,
  k8s: &K8sDetails,
) -> Result<ConsoleAttachment, Error> {
  backend
    .attach_to_node_console(token, site_name, xname, term_size, k8s)
    .await
}

/// Attach an interactive PTY console to a running CFS session pod.
///
/// Callers must validate session access and session liveness BEFORE
/// calling this; see [`crate::service::session::validate_session_access`]
/// and [`crate::service::session::validate_console_session`].
pub async fn attach_to_session_console(
  backend: &StaticBackendDispatcher,
  token: &str,
  site_name: &str,
  session_name: &str,
  term_size: TermSize,
  k8s: &K8sDetails,
) -> Result<ConsoleAttachment, Error> {
  backend
    .attach_to_session_console(token, site_name, session_name, term_size, k8s)
    .await
}
