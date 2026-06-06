//! Provenance metadata attached to an IMS image after a successful CFS
//! session.
//!
//! An IMS image is produced as the output of a CFS session. This module
//! writes three CFS-derived facts onto the resulting `Image`'s
//! `metadata` HashMap so the image is self-describing:
//!
//! - `manta.image_session.base` — the source/base image id the new
//!   image was built on top of (from `cfs.target.image_map[0].source_id`)
//! - `manta.image_session.groups` — JSON-encoded array of HSM group
//!   names the image targets (from `cfs.target.groups[*].name`)
//! - `manta.image_session.configuration` — the CFS configuration name
//!   that was applied (from `cfs.configuration.name`)
//!
//! The IMS-side facts (`id`, `created`, `name`, `arch`) are not
//! duplicated here — they already live on `Image` directly.
//!
//! There is intentionally no managed struct: `Image.metadata` is the
//! storage, and callers that want a single field do their own
//! `metadata.get(META_*)` lookup (with `serde_json::from_str` on the
//! groups value). Two helpers cover the writes:
//!
//! | Op     | Helper                 |
//! |--------|------------------------|
//! | Create | `apply`                |
//! | Update | `apply` (full replace) |
//! | Delete | `clear`                |

use std::collections::HashMap;

use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;

use crate::common::error::MantaError;

/// Metadata key for the base (source) image id the IMS image was built on.
pub const META_BASE: &str = "manta.image_session.base";
/// Metadata key for the JSON-encoded array of HSM group names.
pub const META_GROUPS: &str = "manta.image_session.groups";
/// Metadata key for the CFS configuration name that produced the image.
pub const META_CONFIGURATION: &str = "manta.image_session.configuration";

/// Write the CFS-derived provenance metadata onto `ims`.
///
/// Call this after the CFS session that produced `ims` has finished
/// successfully. Idempotent: calling twice with the same `cfs` is a
/// no-op; calling with a different `cfs` overwrites all three keys.
///
/// Initializes `ims.metadata` to `Some(HashMap::new())` first if it was
/// `None`, so the writes are never silently dropped.
///
/// The mutation is in-memory only. Callers that need the stamped
/// metadata to survive the request must PATCH `ims` back to IMS
/// themselves (manta-server's `InfraContext::apply_image` does this
/// via `InfraContext::update_image`).
///
/// # Errors
///
/// - [`MantaError::MissingField`] if `ims.id` is `None`, if
///   `cfs.configuration.name` is absent, or if `cfs.target.image_map`
///   is `None`/empty (no base image id available).
/// - [`MantaError::SerdeError`] if the `groups` JSON encoding fails
///   (in practice unreachable for `Vec<String>`).
pub fn apply(
  cfs: &CfsSessionGetResponse,
  ims: &mut Image,
) -> Result<(), MantaError> {
  if ims.id.is_none() {
    return Err(MantaError::MissingField("ims.id".into()));
  }

  let base = cfs
    .target
    .as_ref()
    .and_then(|t| t.image_map.as_ref())
    .and_then(|m| m.first())
    .map(|im| im.source_id.clone())
    .ok_or_else(|| {
      MantaError::MissingField("cfs.target.image_map[0].source_id".into())
    })?;

  let groups = cfs.get_target_hsm().unwrap_or_default();

  let configuration = cfs
    .get_configuration_name()
    .ok_or_else(|| MantaError::MissingField("cfs.configuration.name".into()))?;

  let groups_json = serde_json::to_string(&groups)?;

  let metadata = ims.metadata.get_or_insert_with(HashMap::new);
  metadata.insert(META_BASE.to_string(), base);
  metadata.insert(META_GROUPS.to_string(), groups_json);
  metadata.insert(META_CONFIGURATION.to_string(), configuration);

  Ok(())
}

/// Remove the three `manta.image_session.*` keys from `ims.metadata`.
///
/// Leaves any other metadata entries untouched. No-op (returns `Ok`)
/// if the keys aren't present. `id` must match `ims.id` — a guard
/// against callers handing in a mismatched image by accident.
///
/// # Errors
///
/// - [`MantaError::NotFound`] if `ims.id` is `None` or does not equal `id`.
pub fn clear(id: &str, ims: &mut Image) -> Result<(), MantaError> {
  match ims.id.as_deref() {
    Some(actual) if actual == id => {}
    _ => return Err(MantaError::NotFound(format!("image id {id}"))),
  }

  if let Some(metadata) = ims.metadata.as_mut() {
    metadata.remove(META_BASE);
    metadata.remove(META_GROUPS);
    metadata.remove(META_CONFIGURATION);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::cfs::session::{
    Configuration, Group, ImageMap, Target,
  };

  fn sample_cfs(
    config_name: Option<&str>,
    base_id: Option<&str>,
    groups: Vec<&str>,
  ) -> CfsSessionGetResponse {
    CfsSessionGetResponse {
      name: "sess".into(),
      configuration: config_name.map(|n| Configuration {
        name: Some(n.into()),
        limit: None,
      }),
      ansible: None,
      target: Some(Target {
        definition: Some("image".into()),
        groups: Some(
          groups
            .into_iter()
            .map(|g| Group {
              name: g.into(),
              members: vec![],
            })
            .collect(),
        ),
        image_map: base_id.map(|b| {
          vec![ImageMap {
            source_id: b.into(),
            result_name: "out".into(),
          }]
        }),
      }),
      status: None,
      tags: None,
      debug_on_failure: false,
      logs: None,
    }
  }

  fn sample_image(id: Option<&str>) -> Image {
    Image {
      id: id.map(str::to_owned),
      created: None,
      name: "img".into(),
      link: None,
      arch: None,
      metadata: None,
    }
  }

  #[test]
  fn apply_writes_all_three_keys() {
    let cfs = sample_cfs(Some("cfg-1"), Some("base-1"), vec!["g1", "g2"]);
    let mut ims = sample_image(Some("img-1"));

    apply(&cfs, &mut ims).unwrap();

    let md = ims.metadata.as_ref().unwrap();
    assert_eq!(md.get(META_BASE).map(String::as_str), Some("base-1"));
    assert_eq!(
      md.get(META_CONFIGURATION).map(String::as_str),
      Some("cfg-1"),
    );
    assert_eq!(
      md.get(META_GROUPS).map(String::as_str),
      Some(r#"["g1","g2"]"#),
    );
  }

  #[test]
  fn apply_twice_overwrites() {
    let mut ims = sample_image(Some("img-1"));
    apply(
      &sample_cfs(Some("cfg-1"), Some("base-1"), vec!["g1"]),
      &mut ims,
    )
    .unwrap();
    apply(
      &sample_cfs(Some("cfg-2"), Some("base-2"), vec!["g2", "g3"]),
      &mut ims,
    )
    .unwrap();

    let md = ims.metadata.as_ref().unwrap();
    assert_eq!(md.get(META_BASE).map(String::as_str), Some("base-2"));
    assert_eq!(
      md.get(META_CONFIGURATION).map(String::as_str),
      Some("cfg-2"),
    );
    assert_eq!(
      md.get(META_GROUPS).map(String::as_str),
      Some(r#"["g2","g3"]"#),
    );
  }

  #[test]
  fn clear_removes_only_image_session_keys() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    ims
      .metadata
      .as_mut()
      .unwrap()
      .insert("unrelated".into(), "keep-me".into());

    clear("img-1", &mut ims).unwrap();

    let md = ims.metadata.as_ref().unwrap();
    assert!(!md.contains_key(META_BASE));
    assert!(!md.contains_key(META_GROUPS));
    assert!(!md.contains_key(META_CONFIGURATION));
    assert_eq!(md.get("unrelated").map(String::as_str), Some("keep-me"));
  }

  #[test]
  fn clear_not_found_on_id_mismatch() {
    let mut ims = sample_image(Some("img-1"));
    let err = clear("img-2", &mut ims).unwrap_err();
    assert!(matches!(err, MantaError::NotFound(_)), "got {err:?}");
  }

  #[test]
  fn groups_empty_round_trips_through_json() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec![]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    let raw = ims.metadata.as_ref().unwrap().get(META_GROUPS).unwrap();
    let groups: Vec<String> = serde_json::from_str(raw).unwrap();
    assert!(groups.is_empty());
  }

  #[test]
  fn groups_single_round_trips_through_json() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["only"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    let raw = ims.metadata.as_ref().unwrap().get(META_GROUPS).unwrap();
    let groups: Vec<String> = serde_json::from_str(raw).unwrap();
    assert_eq!(groups, vec!["only".to_string()]);
  }

  #[test]
  fn groups_special_chars_round_trip_through_json() {
    // HSM names won't have these in practice, but the JSON encoding
    // tolerates them; verifying so a future malformed group name
    // doesn't silently corrupt the metadata.
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["a,b", "with\"quote"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    let raw = ims.metadata.as_ref().unwrap().get(META_GROUPS).unwrap();
    let groups: Vec<String> = serde_json::from_str(raw).unwrap();
    assert_eq!(
      groups,
      vec!["a,b".to_string(), "with\"quote".to_string()]
    );
  }

  #[test]
  fn apply_initializes_none_metadata() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    assert!(ims.metadata.is_none());

    apply(&cfs, &mut ims).unwrap();

    let md = ims.metadata.as_ref().expect("metadata should be Some");
    assert!(md.contains_key(META_BASE));
    assert!(md.contains_key(META_GROUPS));
    assert!(md.contains_key(META_CONFIGURATION));
  }

  #[test]
  fn apply_missing_field_when_ims_id_none() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(None);
    let err = apply(&cfs, &mut ims).unwrap_err();
    match err {
      MantaError::MissingField(f) => assert_eq!(f, "ims.id"),
      other => panic!("expected MissingField, got {other:?}"),
    }
  }

  #[test]
  fn apply_missing_field_when_image_map_empty() {
    let cfs = sample_cfs(Some("c"), None, vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    let err = apply(&cfs, &mut ims).unwrap_err();
    assert!(matches!(err, MantaError::MissingField(_)), "got {err:?}");
  }

  #[test]
  fn apply_missing_field_when_configuration_name_absent() {
    let cfs = sample_cfs(None, Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    let err = apply(&cfs, &mut ims).unwrap_err();
    match err {
      MantaError::MissingField(f) => assert_eq!(f, "cfs.configuration.name"),
      other => panic!("expected MissingField, got {other:?}"),
    }
  }
}
