//! Library root for the `manta-server` crate.
//!
//! `manta-server` is the Axum HTTPS service that brokers between
//! `manta-cli` clients and the per-site backend implementations (CSM
//! via `csm-rs`, OpenCHAMI via `ochami-rs`). Requests arrive at the
//! [`server`] module's handlers, which delegate to the [`service`]
//! layer, which in turn dispatches to the chosen backend through
//! [`dispatcher::StaticBackendDispatcher`] and the trait impls under
//! [`backend_dispatcher`].
//!
//! # Layering
//!
//! The crate enforces a strict three-tier shape (see `CLAUDE.md` for
//! the canonical statement of the rule):
//!
//! 1. **HTTP layer** — [`server`]: Axum handlers, routing, middleware,
//!    request/response wire types. Handlers MUST only call functions
//!    in [`service`] (or shared helpers in `manta-shared`), never the
//!    backend directly.
//! 2. **Service layer** — [`service`]: pure business logic + access
//!    control, parameterised on a borrowed `InfraContext<'_>`. All
//!    fallible paths return [`manta_backend_dispatcher::error::Error`].
//! 3. **Backend layer** — [`dispatcher`] + [`backend_dispatcher`]:
//!    the `StaticBackendDispatcher` enum routes each trait method to
//!    the active CSM or OCHAMI backend.
//!
//! [`wire_conv`] sits at the service boundary and maps the
//! `MantaError` produced by `manta-shared` helpers onto the structured
//! `BackendError` the service layer uses.
//!
//! # Configuration
//!
//! Server-side configuration lives in [`config::ServerConfiguration`]
//! and is loaded once at startup from `~/.config/manta/server.toml`.
//! Each request carries an `X-Manta-Site` header that picks which
//! configured site's backend to dispatch through.
//!
//! # Entry points
//!
//! The `main.rs` binary is a thin bootstrap shim that loads config,
//! constructs the per-site backends, and hands off to the router built
//! in [`server`]. Integration tests under `crates/manta-server/tests/`
//! import the library directly via `use manta_server::...`.

// Warn (not deny) on undocumented pub items. The server isn't
// docs.rs-bound (publish = false), so the goal here is contributor
// onboarding rather than a public API contract. CI still surfaces
// these warnings via the rustdoc build step.
#![warn(missing_docs)]

pub mod backend_dispatcher;
pub mod config;
pub mod dispatcher;
pub mod server;
pub mod service;
pub mod wire_conv;
