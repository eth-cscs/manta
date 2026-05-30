//! Power on/off/reset endpoints.
//!
//! - `power(...)` POSTs to `/power` and returns immediately with the
//!   PCS transition start output (`{ transition_id, operation }`).
//! - `power_transition(id)` snapshots an in-flight transition via
//!   `GET /power/transitions/{id}` — used by the CLI poll loop.

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  /// `POST /api/v1/power` — start a PCS power transition and return
  /// immediately. The response carries the PCS `transition_id` the
  /// caller then polls with [`MantaClient::power_transition`].
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

  /// `GET /api/v1/power/transitions/{id}` — snapshot an in-flight
  /// power transition. Returns the full PCS `TransitionResponse`
  /// (status, task counts, per-task detail) as a JSON value.
  pub async fn power_transition(
    &self,
    token: &str,
    transition_id: &str,
  ) -> anyhow::Result<Value> {
    self
      .get_json(token, &format!("/power/transitions/{transition_id}"), &[])
      .await
  }
}
