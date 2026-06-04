//! Build the in-memory execution plan from a parsed SAT file.
//!
//! The CLI walks the rendered `serde_json::Value`, applies the
//! `--image-only` / `--sessiontemplate-only` filters (prune-by-reference
//! over the parsed value), and returns a `Vec<SatElement>` in the order
//! each artifact must be created: configurations first, then images
//! (topologically sorted so an image referenced via `base.image_ref` is
//! created before any image or session_template that references it),
//! then session_templates.
//!
//! No struct deserialisation happens here — the canonical SAT schema
//! lives in csm-rs. Field navigation is limited to `configurations`,
//! `images`, `session_templates`, `hardware`, `name`, `ref_name`,
//! `configuration`, `image_ref`, `ims`.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Result, anyhow, bail};
use serde_json::Value;

/// One unit of work in the order it must be executed. The body is the
/// raw SAT entry as a `serde_json::Value`; the canonical SAT types live
/// in csm-rs.
#[derive(Debug, Clone, PartialEq)]
pub enum SatElement {
  Configuration(Value),
  Image(Value),
  SessionTemplate(Value),
}

/// Apply the `image_only` / `session_template_only` filters in place,
/// then walk the SAT file and return the execution plan.
///
/// The input `sat_file` is mutated to reflect the filter so the caller
/// can preview the surviving sections as YAML before confirming. After
/// this call returns, `sat_file` holds only the sections and entries
/// that the plan covers.
///
/// Guarantees on the returned `Vec<SatElement>`:
///   1. All `Configuration` variants come first, in SAT-file order.
///   2. All `Image` variants come next, topologically sorted so that any
///      image whose `base.image_ref` names another image's `ref_name`
///      appears *after* that image. Ties (independent images) stay in
///      SAT-file order.
///   3. All `SessionTemplate` variants come last, in SAT-file order.
///
/// Cross-references inside the SAT file are validated up-front; on the
/// happy path the caller never sees a plan with a dangling `image_ref`.
pub fn build_plan(
  sat_file: &mut Value,
  image_only: bool,
  session_template_only: bool,
) -> Result<Vec<SatElement>> {
  prune_for_filters(sat_file, image_only, session_template_only)?;

  let mut plan: Vec<SatElement> = Vec::new();
  push_configurations(sat_file, &mut plan)?;
  let ref_names = push_images(sat_file, &mut plan)?;
  push_session_templates(sat_file, &ref_names, &mut plan)?;
  Ok(plan)
}

fn push_configurations(
  sat_file: &Value,
  plan: &mut Vec<SatElement>,
) -> Result<()> {
  let Some(cfgs) = sat_file.get("configurations") else {
    return Ok(());
  };
  let arr = cfgs
    .as_array()
    .ok_or_else(|| anyhow!("SAT file: 'configurations' is not an array"))?;
  for entry in arr {
    plan.push(SatElement::Configuration(entry.clone()));
  }
  Ok(())
}

/// Push images in topologically-sorted order and return the set of
/// labels (each image's `ref_name` if present, else its `name`) that
/// downstream `image_ref` references can resolve to. This mirrors
/// csm-rs's resolver: `ref_name_processed_hashmap` is keyed by
/// `ref_name.or(name)`.
fn push_images(
  sat_file: &Value,
  plan: &mut Vec<SatElement>,
) -> Result<HashSet<String>> {
  let Some(imgs) = sat_file.get("images") else {
    return Ok(HashSet::new());
  };
  let images = imgs
    .as_array()
    .ok_or_else(|| anyhow!("SAT file: 'images' is not an array"))?;
  let n = images.len();

  // Pre-pass: per image, capture display name (for error messages), the
  // label other entries resolve to (`ref_name.or(name)`), and the
  // `base.image_ref` dependency when present. Track duplicate explicit
  // `ref_name`s — those are unambiguous user errors. Name collisions
  // would be caught elsewhere (CSM rejects duplicate image names) and
  // aren't validated here.
  let mut names: Vec<String> = Vec::with_capacity(n);
  let mut labels: Vec<Option<String>> = Vec::with_capacity(n);
  let mut depends_on: Vec<Option<String>> = Vec::with_capacity(n);
  let mut label_to_idx: HashMap<String, usize> = HashMap::new();
  let mut ref_name_to_idx: HashMap<String, usize> = HashMap::new();

  for (i, img) in images.iter().enumerate() {
    let name = img.get("name").and_then(Value::as_str).map(str::to_string);
    let ref_name = img
      .get("ref_name")
      .and_then(Value::as_str)
      .map(str::to_string);
    let dep = img
      .get("base")
      .and_then(|b| b.get("image_ref"))
      .and_then(Value::as_str)
      .map(str::to_string);

    if let Some(rn) = &ref_name
      && let Some(prev_idx) = ref_name_to_idx.insert(rn.clone(), i)
    {
      bail!("images #{prev_idx} and #{i} both declare ref_name '{rn}'");
    }

    let label = ref_name.clone().or_else(|| name.clone());
    if let Some(lab) = &label {
      // First occurrence wins; later same-label entries would shadow but
      // are not the common case and aren't enforced here.
      label_to_idx.entry(lab.clone()).or_insert(i);
    }

    names.push(name.unwrap_or_else(|| "<missing name>".to_string()));
    labels.push(label);
    depends_on.push(dep);
  }

  // Validate every depends_on points at a label we've seen.
  for (i, dep) in depends_on.iter().enumerate() {
    if let Some(dep) = dep
      && !label_to_idx.contains_key(dep)
    {
      bail!(
        "image #{i} ('{}') has base.image_ref '{dep}', which does not match any image in this SAT file",
        names[i],
      );
    }
  }

  // Kahn's algorithm, with the ready queue seeded in SAT-file order and
  // dependents re-scanned in SAT-file order on each emit. The graph is a
  // forest (each image has at most one parent — `base.image_ref` is a
  // single field) so a simple linear sweep is enough; SAT files are
  // small (a few dozen images), no need to optimise.
  let mut in_degree: Vec<usize> = depends_on
    .iter()
    .map(|d| usize::from(d.is_some()))
    .collect();
  let mut ready: VecDeque<usize> = VecDeque::new();
  for (i, deg) in in_degree.iter().enumerate() {
    if *deg == 0 {
      ready.push_back(i);
    }
  }
  let mut emitted: Vec<bool> = vec![false; n];

  while let Some(idx) = ready.pop_front() {
    plan.push(SatElement::Image(images[idx].clone()));
    emitted[idx] = true;
    if let Some(lab) = labels[idx].clone() {
      for j in 0..n {
        if !emitted[j]
          && in_degree[j] > 0
          && depends_on[j].as_deref() == Some(lab.as_str())
        {
          in_degree[j] -= 1;
          if in_degree[j] == 0 {
            ready.push_back(j);
          }
        }
      }
    }
  }

  // Any image left unemitted means we hit a cycle. Walk depends_on
  // arrows from the first unemitted index until we revisit a node, then
  // report the cycle path.
  if let Some(start) = emitted.iter().position(|e| !*e) {
    let mut path_idx: Vec<usize> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    let mut cur = start;
    loop {
      path_idx.push(cur);
      if !seen.insert(cur) {
        break;
      }
      let Some(dep) = depends_on[cur].as_deref() else {
        break;
      };
      let Some(next) = label_to_idx.get(dep).copied() else {
        break;
      };
      cur = next;
    }
    let path_str: Vec<&str> =
      path_idx.iter().map(|i| names[*i].as_str()).collect();
    bail!(
      "cycle detected in image dependencies: {}",
      path_str.join(" → ")
    );
  }

  Ok(label_to_idx.into_keys().collect())
}

fn push_session_templates(
  sat_file: &Value,
  ref_names: &HashSet<String>,
  plan: &mut Vec<SatElement>,
) -> Result<()> {
  let Some(sts) = sat_file.get("session_templates") else {
    return Ok(());
  };
  let arr = sts
    .as_array()
    .ok_or_else(|| anyhow!("SAT file: 'session_templates' is not an array"))?;

  for (i, st) in arr.iter().enumerate() {
    if let Some(ir) = st
      .get("image")
      .and_then(|im| im.get("image_ref"))
      .and_then(Value::as_str)
      && !ref_names.contains(ir)
    {
      let name = st
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("<missing name>");
      bail!(
        "session_template #{i} ('{name}') has image.image_ref '{ir}', which does not match any image in this SAT file",
      );
    }
    plan.push(SatElement::SessionTemplate(st.clone()));
  }

  Ok(())
}

/// Prune the SAT value in place according to the `--image-only` /
/// `--sessiontemplate-only` flags. Both off → no-op.
///
/// `image_only`: drops `session_templates` and `hardware`; retains only
/// configurations referenced by the surviving images.
///
/// `session_template_only`: drops `hardware`; retains only images named
/// by some session_template (either by `image.image_ref` or by
/// `image.ims.name`) — the section itself is dropped if no image
/// survives; retains only configurations referenced by surviving images
/// or by any session_template.
fn prune_for_filters(
  sat_file: &mut Value,
  image_only: bool,
  session_template_only: bool,
) -> Result<()> {
  if !image_only && !session_template_only {
    return Ok(());
  }

  if image_only {
    let obj = sat_file
      .as_object_mut()
      .ok_or_else(|| anyhow!("SAT file root is not a YAML/JSON mapping"))?;

    if !obj.contains_key("images") {
      bail!("'images' section missing in SAT file");
    }

    obj.remove("session_templates");
    obj.remove("hardware");

    let referenced: HashSet<String> = obj
      .get("images")
      .and_then(Value::as_array)
      .map(|imgs| {
        imgs
          .iter()
          .filter_map(|img| {
            img.get("configuration")?.as_str().map(str::to_string)
          })
          .collect()
      })
      .unwrap_or_default();

    if let Some(configs) =
      obj.get_mut("configurations").and_then(Value::as_array_mut)
    {
      configs.retain(|cfg| {
        cfg
          .get("name")
          .and_then(Value::as_str)
          .is_some_and(|n| referenced.contains(n))
      });
    }
  }

  if session_template_only {
    let obj = sat_file
      .as_object_mut()
      .ok_or_else(|| anyhow!("SAT file root is not a YAML/JSON mapping"))?;

    if !obj.contains_key("session_templates") {
      bail!("'session_templates' section not defined in SAT file");
    }

    obj.remove("hardware");

    let image_keep: HashSet<String> = obj
      .get("session_templates")
      .and_then(Value::as_array)
      .map(|sts| {
        sts
          .iter()
          .filter_map(image_name_referenced_by_session_template)
          .collect()
      })
      .unwrap_or_default();

    let images_empty =
      if let Some(imgs) = obj.get_mut("images").and_then(Value::as_array_mut) {
        imgs.retain(|img| {
          img
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|n| image_keep.contains(n))
        });
        imgs.is_empty()
      } else {
        false
      };
    if images_empty {
      obj.remove("images");
    }

    let mut config_keep: HashSet<String> = HashSet::new();
    if let Some(imgs) = obj.get("images").and_then(Value::as_array) {
      for img in imgs {
        if let Some(c) = img.get("configuration").and_then(Value::as_str) {
          config_keep.insert(c.to_string());
        }
      }
    }
    if let Some(sts) = obj.get("session_templates").and_then(Value::as_array) {
      for st in sts {
        if let Some(c) = st.get("configuration").and_then(Value::as_str) {
          config_keep.insert(c.to_string());
        }
      }
    }

    if let Some(configs) =
      obj.get_mut("configurations").and_then(Value::as_array_mut)
    {
      configs.retain(|cfg| {
        cfg
          .get("name")
          .and_then(Value::as_str)
          .is_some_and(|n| config_keep.contains(n))
      });
    }
  }

  Ok(())
}

/// Extract the image name a session_template entry references, in either
/// shape: `image: { image_ref: "<name>" }` or `image: { ims: { name:
/// "<name>" } }`. Returns `None` for `image: { ims: { id: "<id>" } }`
/// (pre-built image referenced by id — nothing to filter on).
fn image_name_referenced_by_session_template(st: &Value) -> Option<String> {
  let image = st.get("image")?;
  if let Some(name) = image.get("image_ref").and_then(Value::as_str) {
    return Some(name.to_string());
  }
  image
    .get("ims")
    .and_then(|ims| ims.get("name"))
    .and_then(Value::as_str)
    .map(str::to_string)
}

#[cfg(test)]
mod tests {
  use super::{SatElement, build_plan};
  use serde_json::{Value, json};

  fn name_of(v: &Value) -> &str {
    v.get("name").and_then(Value::as_str).unwrap_or("?")
  }

  #[test]
  fn empty_file_yields_empty_plan() {
    let mut sat = json!({});
    assert!(build_plan(&mut sat, false, false).unwrap().is_empty());
  }

  #[test]
  fn only_configurations_preserves_order() {
    let mut sat = json!({
      "configurations": [
        { "name": "cfg-a" },
        { "name": "cfg-b" },
        { "name": "cfg-c" },
      ]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    assert_eq!(plan.len(), 3);
    let names: Vec<&str> = plan
      .iter()
      .map(|e| match e {
        SatElement::Configuration(v) => name_of(v),
        _ => panic!("expected Configuration variant"),
      })
      .collect();
    assert_eq!(names, vec!["cfg-a", "cfg-b", "cfg-c"]);
  }

  #[test]
  fn independent_images_keep_sat_order() {
    let mut sat = json!({
      "images": [
        { "name": "img-a" },
        { "name": "img-b" },
        { "name": "img-c" },
      ]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    let names: Vec<&str> = plan
      .iter()
      .map(|e| match e {
        SatElement::Image(v) => name_of(v),
        _ => panic!("expected Image variant"),
      })
      .collect();
    assert_eq!(names, vec!["img-a", "img-b", "img-c"]);
  }

  #[test]
  fn image_chain_is_topologically_sorted() {
    // Declared C, A, B. C depends on B, B depends on A. Expect A, B, C.
    let mut sat = json!({
      "images": [
        { "name": "C", "ref_name": "c", "base": { "image_ref": "b" } },
        { "name": "A", "ref_name": "a" },
        { "name": "B", "ref_name": "b", "base": { "image_ref": "a" } },
      ]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    let names: Vec<&str> = plan
      .iter()
      .map(|e| match e {
        SatElement::Image(v) => name_of(v),
        _ => panic!("expected Image variant"),
      })
      .collect();
    assert_eq!(names, vec!["A", "B", "C"]);
    // ref_name field still present on each image body.
    if let SatElement::Image(v) = &plan[0] {
      assert_eq!(v.get("ref_name").and_then(Value::as_str), Some("a"));
    } else {
      panic!();
    }
  }

  #[test]
  fn independent_image_between_dependents_keeps_position() {
    // [A, X, B] — B depends on A, X is independent — must stay [A, X, B].
    let mut sat = json!({
      "images": [
        { "name": "A", "ref_name": "a" },
        { "name": "X" },
        { "name": "B", "base": { "image_ref": "a" } },
      ]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    let names: Vec<&str> = plan
      .iter()
      .map(|e| match e {
        SatElement::Image(v) => name_of(v),
        _ => panic!(),
      })
      .collect();
    assert_eq!(names, vec!["A", "X", "B"]);
  }

  #[test]
  fn session_template_image_ref_matches_known_image() {
    let mut sat = json!({
      "images": [
        { "name": "img-a", "ref_name": "a" },
      ],
      "session_templates": [
        { "name": "st-1", "image": { "image_ref": "a" } },
      ]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    assert_eq!(plan.len(), 2);
    assert!(matches!(plan[0], SatElement::Image(_)));
    assert!(matches!(plan[1], SatElement::SessionTemplate(_)));
  }

  #[test]
  fn session_template_ims_name_passes_through_without_validation() {
    // No image_ref, just ims.name — we cannot validate this against the
    // SAT file, so it passes through to be checked server-side later.
    let mut sat = json!({
      "session_templates": [
        { "name": "st-1", "image": { "ims": { "name": "lives-in-csm" } } },
      ]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    assert_eq!(plan.len(), 1);
    assert!(matches!(plan[0], SatElement::SessionTemplate(_)));
  }

  #[test]
  fn duplicate_ref_name_errors() {
    let mut sat = json!({
      "images": [
        { "name": "A", "ref_name": "shared" },
        { "name": "B", "ref_name": "shared" },
      ]
    });
    let err = build_plan(&mut sat, false, false).unwrap_err().to_string();
    assert!(err.contains("ref_name 'shared'"), "got: {err}");
    assert!(err.contains("#0") && err.contains("#1"), "got: {err}");
  }

  #[test]
  fn dangling_image_ref_errors() {
    let mut sat = json!({
      "images": [
        { "name": "B", "base": { "image_ref": "nope" } },
      ]
    });
    let err = build_plan(&mut sat, false, false).unwrap_err().to_string();
    assert!(err.contains("'nope'"), "got: {err}");
    assert!(
      err.contains("does not match any image in this SAT file"),
      "got: {err}"
    );
  }

  #[test]
  fn image_cycle_errors() {
    let mut sat = json!({
      "images": [
        { "name": "A", "ref_name": "a", "base": { "image_ref": "b" } },
        { "name": "B", "ref_name": "b", "base": { "image_ref": "a" } },
      ]
    });
    let err = build_plan(&mut sat, false, false).unwrap_err().to_string();
    assert!(err.contains("cycle detected"), "got: {err}");
    assert!(err.contains("A") && err.contains("B"), "got: {err}");
  }

  #[test]
  fn dangling_session_template_image_ref_errors() {
    let mut sat = json!({
      "session_templates": [
        { "name": "st-1", "image": { "image_ref": "nope" } },
      ]
    });
    let err = build_plan(&mut sat, false, false).unwrap_err().to_string();
    assert!(err.contains("session_template #0"), "got: {err}");
    assert!(err.contains("'nope'"), "got: {err}");
  }

  #[test]
  fn images_only_file_skips_configurations_segment() {
    let mut sat = json!({
      "images": [{ "name": "only-img" }]
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    assert_eq!(plan.len(), 1);
    assert!(matches!(plan[0], SatElement::Image(_)));
  }

  #[test]
  fn non_array_configurations_errors() {
    let mut sat = json!({ "configurations": "not-an-array" });
    let err = build_plan(&mut sat, false, false).unwrap_err().to_string();
    assert!(
      err.contains("'configurations' is not an array"),
      "got: {err}"
    );
  }

  #[test]
  fn full_plan_preserves_section_order() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg-1" }, { "name": "cfg-2" }],
      "images": [
        { "name": "img-1", "ref_name": "one" },
        { "name": "img-2", "base": { "image_ref": "one" } },
      ],
      "session_templates": [
        { "name": "st-1", "image": { "image_ref": "one" } },
      ],
    });
    let plan = build_plan(&mut sat, false, false).unwrap();
    let kinds: Vec<&str> = plan
      .iter()
      .map(|e| match e {
        SatElement::Configuration(_) => "cfg",
        SatElement::Image(_) => "img",
        SatElement::SessionTemplate(_) => "st",
      })
      .collect();
    assert_eq!(kinds, vec!["cfg", "cfg", "img", "img", "st"]);
  }

  // ── --image-only / --sessiontemplate-only filter coverage ──

  #[test]
  fn image_only_drops_session_templates_and_hardware() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg-used" }, { "name": "cfg-unused" }],
      "images": [{ "name": "img1", "configuration": "cfg-used" }],
      "session_templates": [
        { "name": "st1", "image": { "image_ref": "img1" }, "configuration": "cfg-used" },
      ],
      "hardware": [{ "pattern": "x" }],
    });

    let plan = build_plan(&mut sat, true, false).unwrap();

    assert!(sat.get("session_templates").is_none());
    assert!(sat.get("hardware").is_none());
    let configs = sat.get("configurations").unwrap().as_array().unwrap();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0]["name"], "cfg-used");

    // Plan reflects the pruned value: one config + one image, no st.
    assert_eq!(plan.len(), 2);
    assert!(matches!(plan[0], SatElement::Configuration(_)));
    assert!(matches!(plan[1], SatElement::Image(_)));
  }

  #[test]
  fn session_template_only_keeps_referenced_images_and_drops_unreferenced() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg-st" }, { "name": "cfg-img-only" }],
      "images": [
        { "name": "used-image", "configuration": "cfg-img-only" },
        { "name": "unused-image" },
      ],
      "session_templates": [
        { "name": "st1", "image": { "image_ref": "used-image" }, "configuration": "cfg-st" },
      ],
    });

    let plan = build_plan(&mut sat, false, true).unwrap();

    let images = sat.get("images").unwrap().as_array().unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0]["name"], "used-image");

    // Both configurations kept: cfg-img-only via the surviving image,
    // cfg-st via the session template.
    let configs = sat.get("configurations").unwrap().as_array().unwrap();
    assert_eq!(configs.len(), 2);

    // Plan: 2 configs + 1 image + 1 session_template.
    assert_eq!(plan.len(), 4);
  }

  #[test]
  fn session_template_only_drops_images_section_when_no_match() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg-st" }],
      "images": [{ "name": "img-not-referenced" }],
      "session_templates": [
        { "name": "st1", "image": { "ims": { "id": "abc-123" } }, "configuration": "cfg-st" },
      ],
    });

    let plan = build_plan(&mut sat, false, true).unwrap();

    assert!(sat.get("images").is_none());
    let configs = sat.get("configurations").unwrap().as_array().unwrap();
    assert_eq!(configs.len(), 1);

    // Plan: 1 config + 1 session_template; no Image variants.
    assert_eq!(plan.len(), 2);
    assert!(matches!(plan[0], SatElement::Configuration(_)));
    assert!(matches!(plan[1], SatElement::SessionTemplate(_)));
  }

  #[test]
  fn session_template_only_matches_ims_name_variant() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg" }],
      "images": [{ "name": "ims-name-target" }],
      "session_templates": [
        { "name": "st1", "image": { "ims": { "name": "ims-name-target" } }, "configuration": "cfg" },
      ],
    });

    let plan = build_plan(&mut sat, false, true).unwrap();

    let images = sat.get("images").unwrap().as_array().unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0]["name"], "ims-name-target");
    assert_eq!(plan.len(), 3);
  }

  #[test]
  fn neither_flag_leaves_file_untouched() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg1" }],
      "images": [{ "name": "img1" }],
      "session_templates": [
        { "name": "st1", "image": { "ims": { "name": "img1" } }, "configuration": "cfg1" },
      ],
    });
    let before = sat.clone();

    let _ = build_plan(&mut sat, false, false).unwrap();

    assert_eq!(sat, before);
  }

  #[test]
  fn image_only_without_images_section_errors() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg1" }],
      "session_templates": [],
    });
    let err = build_plan(&mut sat, true, false).unwrap_err().to_string();
    assert!(
      err.contains("'images' section missing in SAT file"),
      "got: {err}"
    );
  }

  #[test]
  fn session_template_only_without_section_errors() {
    let mut sat = json!({
      "configurations": [{ "name": "cfg1" }],
      "images": [],
    });
    let err = build_plan(&mut sat, false, true).unwrap_err().to_string();
    assert!(
      err.contains("'session_templates' section not defined in SAT file"),
      "got: {err}"
    );
  }

  #[test]
  fn filter_errors_when_root_is_not_a_mapping() {
    let mut sat = json!([1, 2, 3]);
    let err = build_plan(&mut sat, true, false).unwrap_err().to_string();
    assert!(err.contains("not a YAML/JSON mapping"), "got: {err}");
  }
}
