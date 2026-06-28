//! The `manta apply sat-file` command and its supporting modules.
//!
//! End-to-end flow:
//!
//! 1. **Render** the SAT-file template by layering the values file
//!    and any `--var key=value` overrides through Jinja2, producing
//!    a YAML string.
//! 2. **Parse** the YAML into a `serde_json::Value` (the CLI keeps
//!    SAT untyped — csm-rs owns the schema).
//! 3. **Plan**: walk the parsed `Value` and produce an ordered
//!    `Vec<SatElement>` (configurations → images → session
//!    templates), honouring the `--image-only` /
//!    `--sessiontemplate-only` filters and validating in-file
//!    cross-references.
//! 4. **Preview + confirm**: render a one-screen summary of every
//!    element and ask the operator for confirmation (`--assume-yes`
//!    bypasses).
//! 5. **Dispatch**: walk the plan element-by-element, POSTing each
//!    to its per-element server endpoint. For `Image` elements the
//!    dispatcher hands off to [`image_pipeline`], which drives the
//!    multi-step CFS-session lifecycle (create → monitor → stamp)
//!    so the operator sees progress instead of blocking on one long
//!    server call.
//!
//! Module map:
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
//! - [`image_pipeline`] — per-image orchestrator the dispatcher calls
//!   for every `SatElement::Image`: drives the three HTTP steps
//!   (create CFS session → monitor → stamp) so the operator can
//!   observe the image build instead of blocking on one long server
//!   call.

pub mod dispatch;
pub mod exec;
pub mod image_pipeline;
pub mod plan;
pub mod render;
// -- TESTS --
#[cfg(test)]
pub mod tests;
