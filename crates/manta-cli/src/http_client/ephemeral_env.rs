//! Ephemeral env (interactive image-boot) endpoint.

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  pub async fn create_ephemeral_env(
    &self,
    token: &str,
    image_id: &str,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({ "image_id": image_id });
    self.post_json(token, "/ephemeral-env", &body).await
  }
}
