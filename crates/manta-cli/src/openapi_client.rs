//! Typed manta-server API client generated at build time from
//! `crates/manta-cli/openapi.json` by progenitor (see `build.rs`).
//!
//! Refresh the spec after handler or schema changes:
//!
//!     cargo run -p manta-server -- --emit-openapi > crates/manta-cli/openapi.json
//!
//! The generated code lives in `$OUT_DIR/openapi_client.rs` so it
//! stays out of the source tree and out of `cargo publish` packaging.

include!(concat!(env!("OUT_DIR"), "/openapi_client.rs"));
