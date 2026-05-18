//! Power on/off/reset endpoint.

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  pub async fn power(
    &self,
    token: &str,
    action: &str,
    targets_expression: &str,
    target_type: &str,
    force: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "action": action,
      "targets_expression": targets_expression,
      "target_type": target_type,
      "force": force,
    });
    self.post_json(token, "/power", &body).await
  }
}
