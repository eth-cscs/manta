//! SAT file apply endpoint.

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  /// `POST /api/v1/sat-file` — apply a pre-rendered SAT file.
  ///
  /// The CLI renders Jinja2, parses the rendered YAML into a
  /// structured `serde_json::Value`, applies the `image_only` /
  /// `session_template_only` filters locally, and forwards the
  /// resulting value in `sat_file`. The server is a pure pass-through
  /// for the SAT content; csm-rs transcodes it during apply.
  ///
  /// The returned `Value` is the server's `PostSatFileResponse` body —
  /// a JSON object with `configurations`, `images`, `session_templates`,
  /// and `bos_sessions` arrays describing the artifacts the backend
  /// produced (or, with `dry_run`, would have produced).
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_sat_file(
    &self,
    token: &str,
    sat_file: Value,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<&str>,
    reboot: bool,
    watch_logs: bool,
    timestamps: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "sat_file": sat_file,
      "ansible_verbosity": ansible_verbosity,
      "ansible_passthrough": ansible_passthrough,
      "reboot": reboot,
      "watch_logs": watch_logs,
      "timestamps": timestamps,
      "overwrite": overwrite,
      "dry_run": dry_run,
    });
    self.post_json(token, "/sat-file", &body).await
  }
}
