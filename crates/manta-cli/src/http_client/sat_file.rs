//! SAT file apply endpoints.
//!
//! One method per SAT element kind plus the image-build sub-flow:
//!
//! - [`MantaClient::apply_sat_configuration`] →
//!   `POST /sat-file/configurations`
//! - [`MantaClient::create_image_cfs_session`] →
//!   `POST /sat-file/images/cfs-session` (start the CFS session)
//! - [`MantaClient::stamp_image_from_cfs_session`] →
//!   `POST /sat-file/images/stamp` (after the CLI has driven the
//!   session to terminal-complete via the existing session endpoints)
//! - [`MantaClient::apply_sat_session_template`] →
//!   `POST /sat-file/session-templates`
//!
//! The monolithic `POST /sat-file/images` is not represented here:
//! the CLI no longer uses it, the new image-build pipeline replaces
//! it. The server still exposes it for external callers.

use std::collections::HashMap;

use manta_shared::types::dto::{CfsSessionGetResponse, Image};
pub use manta_shared::types::wire::sat_file::{
  CreateImageCfsSessionRequest, PostSatConfigurationRequest,
  PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
  StampImageFromSessionRequest,
};
use serde_json::Value;

use super::MantaClient;

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

  /// `POST /api/v1/sat-file/images/cfs-session` — translate one SAT
  /// `images[]` entry into a CFS session and create it. Returns the
  /// freshly-created session resource — caller must drive it to
  /// completion before calling [`Self::stamp_image_from_cfs_session`].
  pub async fn create_image_cfs_session(
    &self,
    token: &str,
    req: &CreateImageCfsSessionRequest,
  ) -> anyhow::Result<CfsSessionGetResponse> {
    self
      .post_json(token, "/sat-file/images/cfs-session", req)
      .await
  }

  /// `POST /api/v1/sat-file/images/stamp` — given the name of a
  /// terminal-complete CFS session, the server derives
  /// `manta.image_session.{base,groups,configuration}` from it and
  /// PATCHes them onto the produced IMS image. Returns the patched
  /// image. Errors when the session has no `result_id`.
  pub async fn stamp_image_from_cfs_session(
    &self,
    token: &str,
    cfs_session_name: &str,
  ) -> anyhow::Result<Image> {
    let body = serde_json::json!({ "cfs_session_name": cfs_session_name });
    self.post_json(token, "/sat-file/images/stamp", &body).await
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
