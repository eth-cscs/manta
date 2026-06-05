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
//! storage. CRUD maps to three helpers:
//!
//! | Op     | Helper             |
//! |--------|--------------------|
//! | Create | `apply`            |
//! | Read   | `read`             |
//! | Update | `apply` (full replace) |
//! | Delete | `clear`            |

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

/// CFS-derived metadata returned by [`read`].
///
/// Holds only the three values read back from the IMS image's metadata
/// entries. Not stored anywhere; exists so [`read`] can return a named
/// shape instead of a `(String, Vec<String>, String)` tuple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSessionMetadata {
  /// Source/base image id (`manta.image_session.base`).
  pub base: String,
  /// HSM group names (`manta.image_session.groups`, JSON-decoded).
  pub groups: Vec<String>,
  /// CFS configuration name (`manta.image_session.configuration`).
  pub configuration: String,
}

/// Write the CFS-derived provenance metadata onto `ims`.
///
/// Call this after the CFS session that produced `ims` has finished
/// successfully. Idempotent: calling twice with the same `cfs` is a
/// no-op; calling with a different `cfs` overwrites all three keys.
///
/// Initializes `ims.metadata` to `Some(HashMap::new())` first if it was
/// `None`, so the writes are never silently dropped.
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

  let groups = cfs
    .get_target_hsm()
    .unwrap_or_default();

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

/// Read the CFS-derived provenance metadata back from an IMS image.
///
/// `id` must match `ims.id`; this is a guard so callers can't pass a
/// mismatched image by accident.
///
/// # Errors
///
/// - [`MantaError::NotFound`] if `ims.id` is `None` or does not equal `id`.
/// - [`MantaError::MissingField`] if any of the three
///   `manta.image_session.*` keys is absent from `ims.metadata`.
/// - [`MantaError::SerdeError`] if the `groups` value is not a valid
///   JSON array of strings.
pub fn read(id: &str, ims: &Image) -> Result<ImageSessionMetadata, MantaError> {
  check_id(id, ims)?;

  let metadata = ims
    .metadata
    .as_ref()
    .ok_or_else(|| MantaError::MissingField(META_BASE.into()))?;

  let base = metadata
    .get(META_BASE)
    .cloned()
    .ok_or_else(|| MantaError::MissingField(META_BASE.into()))?;
  let groups_raw = metadata
    .get(META_GROUPS)
    .ok_or_else(|| MantaError::MissingField(META_GROUPS.into()))?;
  let configuration = metadata
    .get(META_CONFIGURATION)
    .cloned()
    .ok_or_else(|| MantaError::MissingField(META_CONFIGURATION.into()))?;

  let groups: Vec<String> = serde_json::from_str(groups_raw)?;

  Ok(ImageSessionMetadata {
    base,
    groups,
    configuration,
  })
}

/// Remove the three `manta.image_session.*` keys from `ims.metadata`.
///
/// Leaves any other metadata entries untouched. No-op (returns `Ok`)
/// if the keys aren't present.
///
/// # Errors
///
/// - [`MantaError::NotFound`] if `ims.id` is `None` or does not equal `id`.
pub fn clear(id: &str, ims: &mut Image) -> Result<(), MantaError> {
  check_id(id, ims)?;

  if let Some(metadata) = ims.metadata.as_mut() {
    metadata.remove(META_BASE);
    metadata.remove(META_GROUPS);
    metadata.remove(META_CONFIGURATION);
  }

  Ok(())
}

fn check_id(id: &str, ims: &Image) -> Result<(), MantaError> {
  match ims.id.as_deref() {
    Some(actual) if actual == id => Ok(()),
    _ => Err(MantaError::NotFound(format!("image id {id}"))),
  }
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
  fn apply_then_read_roundtrip() {
    let cfs = sample_cfs(Some("cfg-1"), Some("base-1"), vec!["g1", "g2"]);
    let mut ims = sample_image(Some("img-1"));

    apply(&cfs, &mut ims).unwrap();
    let got = read("img-1", &ims).unwrap();

    assert_eq!(
      got,
      ImageSessionMetadata {
        base: "base-1".into(),
        groups: vec!["g1".into(), "g2".into()],
        configuration: "cfg-1".into(),
      }
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

    let got = read("img-1", &ims).unwrap();
    assert_eq!(got.base, "base-2");
    assert_eq!(got.groups, vec!["g2".to_string(), "g3".to_string()]);
    assert_eq!(got.configuration, "cfg-2");
  }

  #[test]
  fn read_not_found_on_id_mismatch() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();

    let err = read("img-2", &ims).unwrap_err();
    assert!(matches!(err, MantaError::NotFound(_)), "got {err:?}");
  }

  #[test]
  fn read_not_found_when_image_has_no_id() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    ims.id = None;

    let err = read("img-1", &ims).unwrap_err();
    assert!(matches!(err, MantaError::NotFound(_)), "got {err:?}");
  }

  #[test]
  fn read_missing_field_when_key_absent() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    ims.metadata.as_mut().unwrap().remove(META_GROUPS);

    let err = read("img-1", &ims).unwrap_err();
    match err {
      MantaError::MissingField(k) => assert_eq!(k, META_GROUPS),
      other => panic!("expected MissingField, got {other:?}"),
    }
  }

  #[test]
  fn read_serde_error_on_malformed_groups() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["g"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    ims
      .metadata
      .as_mut()
      .unwrap()
      .insert(META_GROUPS.into(), "not json".into());

    let err = read("img-1", &ims).unwrap_err();
    assert!(matches!(err, MantaError::SerdeError(_)), "got {err:?}");
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
  fn groups_roundtrip_empty() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec![]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    assert!(read("img-1", &ims).unwrap().groups.is_empty());
  }

  #[test]
  fn groups_roundtrip_single() {
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["only"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    assert_eq!(read("img-1", &ims).unwrap().groups, vec!["only".to_string()]);
  }

  #[test]
  fn groups_roundtrip_special_chars() {
    // HSM names won't have these in practice, but the JSON encoding
    // tolerates them; verifying so a future malformed group name
    // doesn't silently corrupt the metadata.
    let cfs = sample_cfs(Some("c"), Some("b"), vec!["a,b", "with\"quote"]);
    let mut ims = sample_image(Some("img-1"));
    apply(&cfs, &mut ims).unwrap();
    assert_eq!(
      read("img-1", &ims).unwrap().groups,
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
