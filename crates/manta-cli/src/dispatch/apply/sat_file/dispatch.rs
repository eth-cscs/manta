//! Walk the `Vec<SatElement>` execution plan and POST each element to
//! the manta server's per-element SAT endpoints.
//!
//! Element order, dependency resolution, and `ref_name → image_id`
//! accumulation are the CLI's responsibility; the server is a thin
//! shell that forwards each element to csm-rs's per-element creator.
//! See `plan::build_plan` for the ordering guarantees.
//!
//! For `SatElement::Image` entries the loop delegates to
//! [`super::image_pipeline::run_image_pipeline`], which itself splits the
//! image build across three HTTP calls (create CFS session → monitor
//! → stamp) so the operator can observe progress instead of blocking
//! on one long server call. `dispatch_plan` only sees the final
//! `Image` value and records its `id` into `ref_lookup` for any
//! downstream images / session_templates that reference it.

use std::collections::HashMap;

use anyhow::Context as _;
use serde_json::Value;

use super::{
  exec::SatApplyOptions, image_pipeline::run_image_pipeline, plan::SatElement,
};
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::{
  PostSatConfigurationRequest, PostSatSessionTemplateRequest,
};

/// Dispatch every element in the plan in order. For each `Image`, the
/// resulting `Image` value (from `run_image_pipeline`) carries an `id`
/// (real, or `DRYRUN-…` synthetic in dry-run mode) which is recorded
/// under `image.ref_name.or(image.name)` so subsequent images and
/// session_templates can resolve their `image_ref` references.
///
/// Returns a `Value` with the four-list shape
/// (`{ configurations, images, session_templates, bos_sessions }`) the
/// existing `print_with_data` helper expects — the same shape the
/// whole-file `POST /sat-file` endpoint still uses for SAT files with
/// a `hardware:` section, so output looks identical to users mixing
/// the two paths.
pub async fn dispatch_plan(
  client: &MantaClient,
  plan: Vec<SatElement>,
  opts: &SatApplyOptions<'_>,
) -> anyhow::Result<Value> {
  let mut ref_lookup: HashMap<String, String> = HashMap::new();
  let mut configurations: Vec<Value> = Vec::new();
  let mut images: Vec<Value> = Vec::new();
  let mut session_templates: Vec<Value> = Vec::new();
  let mut bos_sessions: Vec<Value> = Vec::new();

  for element in plan {
    match element {
      SatElement::Configuration(body) => {
        let cfg = client
          .openapi
          .post_sat_configuration(
            client.site_name(),
            &PostSatConfigurationRequest {
              configuration: body,
              overwrite: Some(opts.overwrite),
              dry_run: Some(opts.dry_run),
            },
          )
          .await
          .into_anyhow()?;
        configurations.push(cfg);
      }
      SatElement::Image(body) => {
        let label = image_label(&body);
        let display_name = body
          .get("name")
          .and_then(Value::as_str)
          .unwrap_or("<unnamed>")
          .to_string();

        let img = run_image_pipeline(client, &body, &ref_lookup, opts)
          .await
          .with_context(|| format!("building SAT image '{display_name}'"))?;

        if let Some(lab) = label {
          let id = resolve_image_id(&img, &lab);
          ref_lookup.insert(lab, id);
        }

        images.push(img);
      }
      SatElement::SessionTemplate(body) => {
        let resp = client
          .openapi
          .post_sat_session_template(
            client.site_name(),
            &PostSatSessionTemplateRequest {
              session_template: body,
              ref_lookup: ref_lookup.clone(),
              reboot: Some(opts.reboot),
              dry_run: Some(opts.dry_run),
            },
          )
          .await
          .into_anyhow()?;
        session_templates.push(resp.template);
        if let Some(s) = resp.session
          && matches!(s, Value::Object(_))
        {
          bos_sessions.push(s);
        }
      }
    }
  }

  Ok(serde_json::json!({
    "configurations": configurations,
    "images": images,
    "session_templates": session_templates,
    "bos_sessions": bos_sessions,
  }))
}

/// The label downstream `image_ref` references resolve against —
/// `ref_name` if the image declares one, else `name`. Matches csm-rs's
/// `ref_name.or(name)` resolver.
fn image_label(body: &Value) -> Option<String> {
  body
    .get("ref_name")
    .or_else(|| body.get("name"))
    .and_then(Value::as_str)
    .map(str::to_string)
}

/// Pick the image id to record under a label in the ref_lookup map.
/// csm-rs's normal and dry-run paths both populate `id`; the synthetic
/// fallback only fires if for any reason the response has no `id`.
fn resolve_image_id(img: &Value, label: &str) -> String {
  img
    .get("id")
    .and_then(Value::as_str)
    .map(str::to_string)
    .unwrap_or_else(|| format!("dry-run-{label}"))
}

#[cfg(test)]
mod tests {
  use super::{image_label, resolve_image_id};
  use serde_json::json;

  #[test]
  fn image_label_prefers_ref_name() {
    let body = json!({ "name": "my-image", "ref_name": "base" });
    assert_eq!(image_label(&body), Some("base".to_string()));
  }

  #[test]
  fn image_label_falls_back_to_name() {
    let body = json!({ "name": "my-image" });
    assert_eq!(image_label(&body), Some("my-image".to_string()));
  }

  #[test]
  fn image_label_returns_none_when_neither_field_present() {
    let body = json!({ "configuration": "cfg-1" });
    assert_eq!(image_label(&body), None);
  }

  #[test]
  fn resolve_image_id_uses_response_id_when_present() {
    let img = json!({ "id": "abc-123", "name": "my-image" });
    assert_eq!(resolve_image_id(&img, "base"), "abc-123");
  }

  #[test]
  fn resolve_image_id_falls_back_to_synthetic_when_id_missing() {
    let img = json!({ "name": "my-image" });
    assert_eq!(resolve_image_id(&img, "base"), "dry-run-base");
  }

  /// csm-rs's dry-run path populates `id` with a `DRYRUN_<uuid>` value,
  /// so resolve_image_id should pass that through rather than synthesise.
  #[test]
  fn resolve_image_id_passes_through_dryrun_uuid_from_csm_rs() {
    let img = json!({ "id": "DRYRUN_a1b2", "name": "my-image" });
    assert_eq!(resolve_image_id(&img, "base"), "DRYRUN_a1b2");
  }

  /// In the dry-run branch the image_pipeline synthesises its return
  /// Value from the mocked CFS session: `{ id: <session.result_id>,
  /// name: <SAT image name> }`. resolve_image_id picks the synthetic
  /// id straight off that.
  #[test]
  fn resolve_image_id_reads_dryrun_id_synthesised_by_image_pipeline() {
    let img = json!({ "id": "DRYRUN-build-img-v1", "name": "img-v1" });
    assert_eq!(resolve_image_id(&img, "img-v1"), "DRYRUN-build-img-v1");
  }
}
