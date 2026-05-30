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
        // `ref_name.or(name)` — the label downstream refs resolve
        // against, matching csm-rs's resolver.
        let label = body
          .get("ref_name")
          .or_else(|| body.get("name"))
          .and_then(Value::as_str)
          .map(str::to_string);

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

        // Record the resolved id under the label so chained images and
        // session_templates can find it. csm-rs's dry-run path already
        // returns an id (`DRYRUN_<uuid>`); if for any reason there's
        // none, fall back to a synthetic key.
        if let Some(lab) = label {
          let id = img
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("dry-run-{lab}"));
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
