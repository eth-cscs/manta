//! Backend-dispatcher methods grouped on `InfraContext`.
//!
//! Service code should call `infra.method(token, ...)` instead of
//! reaching into `infra.backend` and re-passing `shasta_base_url` /
//! `shasta_root_cert` at each call site. This module owns the URL/cert
//! unpacking so the rest of the service layer never sees them.
//!
//! Per-domain methods live in sibling files (`auth.rs`, `bos.rs`,
//! `bss.rs`, `cfs.rs`, `delete_configurations.rs`, `hsm.rs`, `ims.rs`,
//! `migrate.rs`, `pcs.rs`, `redfish.rs`, `sat.rs`); each is a single
//! `impl InfraContext<'_>` block. This file owns the dispatcher-meta
//! methods that don't belong to any one backend domain.

mod auth;
mod bos;
mod bss;
mod cfs;
mod delete_configurations;
mod hsm;
mod ims;
mod migrate;
mod pcs;
mod redfish;
mod sat;

pub use delete_configurations::DeletionCandidates;

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
  ///
  /// This is the only sanctioned way for code outside this module to
  /// touch the dispatcher; everything else should call the named
  /// `InfraContext::<verb>` methods so per-call site doesn't re-pass
  /// `shasta_base_url` / `shasta_root_cert`.
  pub fn backend_clone(&self) -> crate::dispatcher::StaticBackendDispatcher {
    self.backend.clone()
  }
}
