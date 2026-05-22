//! Parameters for `POST /sat-file`.

/// Parameters for applying a SAT file.
//
// Seven bools — `reboot`, `watch_logs`, `timestamps`, `image_only`,
// `session_template_only`, `overwrite`, `dry_run`. `image_only` and
// `session_template_only` are mutually exclusive and could be a
// 3-variant enum, but doing so would break the HTTP request body and
// the CLI flag surface; the rest are independent. Tracked as a future
// API refactor — for now silence the `struct_excessive_bools` lint.
#[allow(clippy::struct_excessive_bools)]
pub struct ApplySatFileParams<'a> {
  /// Raw YAML body of the SAT file. May contain Jinja2 syntax that
  /// the service layer renders against `values` and
  /// `values_file_content` before parsing.
  pub sat_file_content: &'a str,
  /// Inline JSON object of Jinja2 variable overrides. Merged on top
  /// of `values_file_content` when both are supplied.
  pub values: Option<&'a serde_json::Value>,
  /// YAML body of a separate values file, supplying the lower-priority
  /// half of the Jinja2 variable set.
  pub values_file_content: Option<&'a str>,
  /// Ansible verbosity level (0–4) passed to any CFS sessions
  /// created by this SAT file.
  pub ansible_verbosity: Option<u8>,
  /// Extra arguments forwarded verbatim to `ansible-playbook`.
  pub ansible_passthrough: Option<&'a str>,
  /// When true, reboot affected nodes after the session templates
  /// are applied.
  pub reboot: bool,
  /// When true, stream CFS session logs to the caller as part of
  /// the response.
  pub watch_logs: bool,
  /// When true, prefix each streamed log line with its timestamp.
  pub timestamps: bool,
  /// Process only the `images` section of the SAT file; skip
  /// session templates.
  pub image_only: bool,
  /// Process only the `session_templates` section; skip image
  /// builds.
  pub session_template_only: bool,
  /// Overwrite existing CFS configurations or IMS images instead
  /// of erroring on conflict.
  pub overwrite: bool,
  /// Render and validate the SAT file without creating any
  /// resources.
  pub dry_run: bool,
}
