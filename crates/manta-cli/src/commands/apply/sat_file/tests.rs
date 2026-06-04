//! End-to-end client-side pipeline test for `manta apply sat-file`:
//! render → parse → filter → plan, exercised against an `image_only`
//! input. Pure-renderer tests live in [`super::render`]; this file
//! holds tests that span the full module.

use crate::commands::apply::sat_file::plan::{SatElement, build_plan};
use crate::commands::apply::sat_file::render::render_jinja2_sat_file_yaml;

/// End-to-end client-side pipeline: render Jinja2, parse to
/// `serde_json::Value`, apply `image_only=true`, and confirm the
/// session_templates section was stripped and unreferenced
/// configurations were pruned before the value would be sent.
#[test]
fn test_client_side_pipeline_with_image_only_filter() {
  let sat_file_content = r#"
configurations:
- name: cfg-{{ app.version }}
  layers:
    - name: layer1
      git:
        url: https://example.com/repo.git
        branch: main
- name: cfg-unused-{{ app.version }}
  layers:
    - name: layer-unused
      git:
        url: https://example.com/repo.git
        branch: main
images:
- name: img-{{ app.version }}
  ims:
    is_recipe: false
    id: abc-123
  configuration: cfg-{{ app.version }}
session_templates:
- name: st-{{ app.version }}
  image:
    image_ref: img-{{ app.version }}
  configuration: cfg-{{ app.version }}
  bos_parameters:
    boot_sets:
      compute:
        node_groups:
          - group1
"#;
  let values_file_content = r#"
app:
  version: v1
"#;

  let rendered = render_jinja2_sat_file_yaml(
    sat_file_content,
    Some(values_file_content),
    None,
  )
  .expect("render");
  let mut sat: serde_json::Value =
    serde_yaml::from_str(&rendered).expect("parse to Value");

  let plan = build_plan(&mut sat, true, false).expect("build_plan image_only");

  assert!(
    sat.get("session_templates").is_none(),
    "image_only filter should drop session_templates"
  );
  let images = sat.get("images").unwrap().as_array().unwrap();
  assert_eq!(images.len(), 1);
  assert_eq!(images[0]["name"], "img-v1");
  // Only the referenced configuration survives the prune.
  let configs = sat.get("configurations").unwrap().as_array().unwrap();
  assert_eq!(configs.len(), 1);
  assert_eq!(configs[0]["name"], "cfg-v1");

  // Plan reflects the same: one configuration, one image, no session_template.
  assert_eq!(plan.len(), 2);
  assert!(matches!(plan[0], SatElement::Configuration(_)));
  assert!(matches!(plan[1], SatElement::Image(_)));
}
