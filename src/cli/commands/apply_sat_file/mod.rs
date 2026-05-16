//! Re-exports the `manta apply sat-file` command.
//!
//! The SAT YAML deserialization types and the Jinja2 renderer used to live in
//! a `utils` submodule here, but they're shared with the server-side
//! `service::sat_file`, so they now live in `crate::common::sat_file`.

pub mod command;
// -- TESTS --
#[cfg(test)]
pub mod tests;
