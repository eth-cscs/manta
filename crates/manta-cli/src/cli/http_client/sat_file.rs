//! SAT file apply endpoint.

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  #[allow(clippy::too_many_arguments)]
  pub async fn apply_sat_file(
    &self,
    token: &str,
    sat_file_content: &str,
    values: Option<serde_json::Value>,
    values_file_content: Option<&str>,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<&str>,
    reboot: bool,
    watch_logs: bool,
    timestamps: bool,
    image_only: bool,
    session_template_only: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "sat_file_content": sat_file_content,
      "values": values,
      "values_file_content": values_file_content,
      "ansible_verbosity": ansible_verbosity,
      "ansible_passthrough": ansible_passthrough,
      "reboot": reboot,
      "watch_logs": watch_logs,
      "timestamps": timestamps,
      "image_only": image_only,
      "session_template_only": session_template_only,
      "overwrite": overwrite,
      "dry_run": dry_run,
    });
    self.post_json(token, "/sat-file", &body).await
  }
}
