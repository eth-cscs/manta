//! Helpers used only by the CLI binary.
//!
//! - [`app_context`] — per-invocation context (config, site override,
//!   global flags) threaded through every dispatch handler.
//! - [`authentication`] — token bootstrap, refresh, and keyring
//!   persistence; talks to `manta-server`'s `/auth` endpoints.
//! - [`clap_ext`] — `ArgMatches` extension trait with type-safe
//!   accessors (`req_str`, `opt_str`, …) used by every handler.
//! - [`config`] — typed schema for `cli.toml`
//!   ([`config::CliConfiguration`]).
//! - [`confirm`] — interactive y/N prompt with `--assume-yes` bypass.
//! - [`hooks`] — pre/post-command hook execution (shell scripts the
//!   operator wires in via `cli.toml`).
//! - [`multi_line`] — terminal-friendly multi-line input helper.
//! - [`read_only`] — guard that blocks mutating verbs when the CLI is
//!   configured in read-only mode.

pub mod app_context;
pub mod authentication;
pub mod clap_ext;
pub mod config;
pub mod confirm;
pub mod hooks;
pub mod multi_line;
pub mod read_only;
