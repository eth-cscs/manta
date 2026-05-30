//! Walk the `Vec<SatElement>` execution plan and POST each element to
//! the manta server's per-element SAT endpoints.
//!
//! Element order, dependency resolution, and `ref_name → image_id`
//! accumulation are the CLI's responsibility; the server is a thin
//! shell that forwards each element to csm-rs's per-element creator.
//! See `plan::build_plan` for the ordering guarantees.

use std::collections::HashMap;

use serde_json::Value;

use super::{command::SatApplyOptions, plan::SatElement};
use crate::cli::http_client::MantaClient;

/// Dispatch every element in the plan in order. For each `Image`, the
/// response's `id` (or, on dry-run, a synthetic key) is recorded under
/// `image.ref_name.or(image.name)` so subsequent images and
/// session_templates can resolve their `image_ref` references.
///
/// Returns a `Value` shaped exactly like the legacy `POST /sat-file`
/// response — `{ configurations, images, session_templates,
/// bos_sessions }` — so the existing `print_with_data` helper produces
/// the same user-visible output.
pub async fn dispatch_plan(
  client: &MantaClient,
  token: &str,
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
          .apply_sat_configuration(token, &body, opts.overwrite, opts.dry_run)
          .await?;
        configurations.push(cfg);
      }
      SatElement::Image(body) => {
        let label = image_label(&body);

        let img = client
          .apply_sat_image(
            token,
            &body,
            &ref_lookup,
            opts.ansible_verbosity_opt,
            opts.ansible_passthrough_opt,
            opts.watch_logs,
            opts.timestamps,
            opts.dry_run,
          )
          .await?;

        if let Some(lab) = label {
          let id = resolve_image_id(&img, &lab);
          ref_lookup.insert(lab, id);
        }

        images.push(img);
      }
      SatElement::SessionTemplate(body) => {
        let resp = client
          .apply_sat_session_template(
            token,
            &body,
            &ref_lookup,
            opts.reboot,
            opts.dry_run,
          )
          .await?;
        // Server response shape: { template, session? }.
        let mut obj = match resp {
          Value::Object(o) => o,
          other => anyhow::bail!(
            "session_template response was not an object: {other}"
          ),
        };
        if let Some(tpl) = obj.remove("template") {
          session_templates.push(tpl);
        }
        if let Some(Value::Object(_)) = obj.get("session") {
          if let Some(s) = obj.remove("session") {
            bos_sessions.push(s);
          }
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
}
