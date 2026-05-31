//! The `manta apply sat-file` command and its supporting modules.
//!
//! - [`command`] — entry point: render Jinja2, parse, filter, build
//!   the plan, preview + confirm, then dispatch.
//! - [`plan`] — walks the parsed SAT `Value` and returns
//!   `Vec<SatElement>` in execution order, applying the
//!   `--image-only` / `--sessiontemplate-only` filters and validating
//!   in-file cross-references along the way.
//! - [`dispatch`] — walks the plan element-by-element, POSTing each
//!   to the corresponding per-element server endpoint and
//!   accumulating the CLI's `ref_name → image_id` lookup.
//!
//! The Jinja2 renderer lives in `manta_shared::common::sat_file`
//! (where it's shared with the server-side test fixtures); the
//! filter and plan logic moved here from manta-shared once the CLI
//! gained ownership of the execution order.

pub mod command;
pub mod dispatch;
pub mod plan;
// -- TESTS --
#[cfg(test)]
pub mod tests;
