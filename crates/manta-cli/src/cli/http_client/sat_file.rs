//! SAT file apply endpoint.

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  /// `POST /api/v1/sat-file` — apply a pre-rendered SAT file.
  ///
  /// The CLI renders Jinja2, parses, and filters the SAT YAML locally;
  /// only the post-processed YAML plus apply-time flags are forwarded.
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_sat_file(
    &self,
    token: &str,
    sat_yaml: &str,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<&str>,
    reboot: bool,
    watch_logs: bool,
    timestamps: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "sat_yaml": sat_yaml,
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
