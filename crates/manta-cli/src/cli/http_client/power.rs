//! Power on/off/reset endpoints.
//!
//! - [`MantaClient::power`] POSTs to `/power` and returns
//!   immediately with the PCS transition start output
//!   (`{ "transitionID": …, "operation": … }`).
//! - [`MantaClient::power_transition`] snapshots an in-flight
//!   transition via `GET /power/transitions/{id}` — used by the
//!   CLI poll loop in `power_common::poll_until_done`.
//!
//! Both responses come back as untyped `serde_json::Value` — the
//! dispatcher/typed shapes (`TransitionStartOutput`,
//! `TransitionResponse`) live in csm-rs / manta-backend-dispatcher;
//! the CLI walks the JSON directly. Wire field names are PCS-style
//! camelCase (`transitionID`, `transitionStatus`, `taskCounts.…`).

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  /// `POST /api/v1/power` — start a PCS power transition and return
  /// immediately. Does **not** block until the transition completes.
  /// The response carries the PCS `transitionID` the caller then
  /// polls with [`MantaClient::power_transition`].
  ///
  /// `action` is the lowercase form (`"on"`, `"off"`, `"reset"`)
  /// that the server's request schema expects; the server maps
  /// `(action, force)` to the PCS wire-level operation
  /// (`"on"` / `"soft-off"` / `"force-off"` / `"soft-restart"` /
  /// `"hard-restart"`).
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
  /// (or completed) power transition. Returns the full PCS
  /// `TransitionResponse` shape as a JSON value: `transitionStatus`,
  /// `taskCounts` (`total`, `failed`, `in-progress`, `succeeded`,
  /// `new`, `un-supported`), and per-task detail in `tasks`.
  /// Termination condition for the CLI poll loop:
  /// `transitionStatus == "completed"`.
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
