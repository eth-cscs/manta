//! SAT file apply endpoints.
//!
//! The three `apply_sat_*` methods POST one SAT element at a time and
//! drive the CLI's plan-based dispatcher.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use super::MantaClient;

/// Request body for `POST /sat-file/images`.
#[derive(Serialize)]
pub struct ApplySatImageRequest<'a> {
  pub image: &'a Value,
  pub ref_lookup: &'a HashMap<String, String>,
  pub ansible_verbosity: Option<u8>,
  pub ansible_passthrough: Option<&'a str>,
  pub watch_logs: bool,
  pub timestamps: bool,
  pub dry_run: bool,
}

impl MantaClient {
  /// `POST /api/v1/sat-file/configurations` — apply one SAT
  /// configuration entry. Returns the created `CfsConfigurationResponse`
  /// as `Value` (the dispatcher passes it straight through to the
  /// summary). `dry_run` returns a mock response with the configuration
  /// name set.
  pub async fn apply_sat_configuration(
    &self,
    token: &str,
    configuration: &Value,
    overwrite: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "configuration": configuration,
      "overwrite": overwrite,
      "dry_run": dry_run,
    });
    self
      .post_json(token, "/sat-file/configurations", &body)
      .await
  }

  /// `POST /api/v1/sat-file/images` — apply one SAT image entry.
  /// `ref_lookup` carries the CLI's accumulated `ref_name.or(name) ->
  /// image_id` map; the backend uses it to resolve `base.image_ref`.
  /// Returns the created `Image` as `Value`.
  pub async fn apply_sat_image(
    &self,
    token: &str,
    req: &ApplySatImageRequest<'_>,
  ) -> anyhow::Result<Value> {
    self.post_json(token, "/sat-file/images", req).await
  }

  /// `POST /api/v1/sat-file/session-templates` — apply one SAT
  /// session_template entry. Returns the server's
  /// `PostSatSessionTemplateResponse` body (`{ template, session }`)
  /// as `Value`.
  pub async fn apply_sat_session_template(
    &self,
    token: &str,
    session_template: &Value,
    ref_lookup: &HashMap<String, String>,
    reboot: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "session_template": session_template,
      "ref_lookup": ref_lookup,
      "reboot": reboot,
      "dry_run": dry_run,
    });
    self
      .post_json(token, "/sat-file/session-templates", &body)
      .await
  }
}
