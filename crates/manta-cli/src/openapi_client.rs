//! ============================================================
//! AUTO-GENERATED — do not edit by hand.
//! ============================================================
//!
//! Typed manta-server API client emitted at build time by
//! `crates/manta-cli/build.rs`, which runs progenitor 0.14 against
//! `crates/manta-cli/openapi.json`.
//!
//! Source of truth for behaviour: the utoipa annotations on each
//! handler under `crates/manta-server/src/server/handlers/`. To
//! refresh the spec after a handler or schema change, re-run:
//!
//!     cargo run -p manta-server -- --emit-openapi > crates/manta-cli/openapi.json
//!
//! Then `cargo build -p manta-cli` will regenerate this module from
//! the new spec. The generated code itself lives in
//! `$OUT_DIR/openapi_client.rs` so it stays out of the source tree
//! and out of `cargo publish` packaging.
//!
//! Two endpoint families are NOT served by this generated client and
//! live as hand-rolled `impl MantaClient` methods elsewhere:
//!   - WebSocket consoles → `http_client/console.rs`
//!   - SSE log streaming  → `http_client/streaming.rs`
//! progenitor's request/response model doesn't cover WS upgrades or
//! `text/event-stream` long-poll bodies, so those endpoints stay
//! hand-written.

// `clippy::all` stays: generated code routinely trips style lints
// (uninlined format args, `::std::vec::Vec`, redundant lifetimes, …)
// that would require modifying progenitor's templates to fix.
// `dead_code` is NOT silenced — if the spec drifts and a generated
// type stops being consumed, we want the warning to surface so the
// unused schema can be trimmed from `server/api_doc.rs`.
#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/openapi_client.rs"));
