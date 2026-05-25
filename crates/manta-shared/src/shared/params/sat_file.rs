//! Parameters for `POST /sat-file`.

/// Parameters for applying a SAT file.
///
/// The CLI now renders Jinja2, parses the SAT YAML, and applies the
/// `image_only` / `session_template_only` filters locally before the
/// request reaches the server, so only the post-processed YAML and the
/// apply-time flags travel on the wire.
pub struct ApplySatFileParams<'a> {
  /// Final, fully-rendered SAT YAML — Jinja2 already evaluated and
  /// `image_only` / `session_template_only` filters already applied
  /// client-side.
  pub sat_yaml: &'a str,
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
  /// Overwrite existing CFS configurations or IMS images instead
  /// of erroring on conflict.
  pub overwrite: bool,
  /// Render and validate the SAT file without creating any
  /// resources.
  pub dry_run: bool,
}
