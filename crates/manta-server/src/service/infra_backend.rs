//! Dispatcher-meta methods on `InfraContext`.
//!
//! `InfraContext<'a>` itself is defined in
//! [`crate::server::common::app_context`]; this file extends it with
//! the two helpers every service-layer module needs that aren't
//! per-domain trait methods on the backend dispatcher:
//!
//! - [`InfraContext::backend_kind`] — stable label for tracing /
//!   diagnostics, no I/O.
//! - [`InfraContext::backend_clone`] — owned copy for use inside
//!   `'static`-bound spawned tasks (`tokio::spawn`, `JoinSet::spawn`).
//!
//! Earlier revisions of this module hosted per-domain wrappers
//! (auth/bos/bss/cfs/delete_configurations/hsm/ims/migrate/pcs/redfish/
//! sat) that re-exported each backend trait method on `InfraContext`.
//! Those were removed; service code now imports the trait directly
//! and calls `infra.backend.<method>(...)`. The result is fewer
//! abstraction layers between a service function and the backend
//! contract it's actually targeting.

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Stable label for the active backend (`csm`, `ochami`, ...).
  ///
  /// Cheap, infallible — forwards to
  /// [`crate::dispatcher::StaticBackendDispatcher::backend_kind`].
  /// Used as a structured `tracing` field across the service and
  /// backend_dispatcher layers.
  pub fn backend_kind(&self) -> &'static str {
    self.backend.backend_kind()
  }

  /// Return an owned clone of the backend dispatcher.
  ///
  /// `InfraContext<'a>` is borrowed, so it can't cross into a
  /// `'static`-bound spawned future (`tokio::spawn`, `JoinSet::spawn`).
  /// Service code that fans out per-node work needs an owned dispatcher
  /// inside each spawned task — that's what this returns. Cloning is
  /// cheap because `StaticBackendDispatcher` is `Arc`-shaped under the
  /// hood.
  pub fn backend_clone(&self) -> crate::dispatcher::StaticBackendDispatcher {
    self.backend.clone()
  }
}
