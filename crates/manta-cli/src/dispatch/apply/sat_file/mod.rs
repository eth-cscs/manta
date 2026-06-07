//! The `manta apply sat-file` command and its supporting modules.
//!
//! - [`render`] — Jinja2 rendering: layer the values file + `--var`
//!   overrides, then render the SAT template to a YAML string.
//! - [`exec`] — entry point: render, parse, filter, build the plan,
//!   preview + confirm, then dispatch.
//! - [`plan`] — walks the parsed SAT `Value` and returns
//!   `Vec<SatElement>` in execution order, applying the
//!   `--image-only` / `--sessiontemplate-only` filters and validating
//!   in-file cross-references along the way.
//! - [`dispatch`] — walks the plan element-by-element, POSTing each
//!   to the corresponding per-element server endpoint and
//!   accumulating the CLI's `ref_name → image_id` lookup.

pub mod dispatch;
pub mod exec;
pub mod image_pipeline;
pub mod plan;
pub mod render;
// -- TESTS --
#[cfg(test)]
pub mod tests;
