//! Dispatcher-meta methods on `InfraContext`.
//!
//! The per-domain wrapper methods that used to live in sibling files
//! (auth/bos/bss/cfs/delete_configurations/hsm/ims/migrate/pcs/redfish/
//! sat) have been removed; service code now calls
//! `infra.backend.<method>(...)` directly via the corresponding trait
//! imports. Only the truly dispatcher-level helpers remain here:
//! `backend_kind` and `backend_clone`.

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Stable label for the active backend (`csm`, `ochami`, ...).
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
