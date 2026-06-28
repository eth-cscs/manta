//! IMS image queries and safety-checked deletion.
//!
//! Image-deletion has two failure modes that the service layer
//! refuses to let through:
//!
//! 1. Deleting the boot image of a currently-booted node would brick
//!    the next boot. [`validate_image_deletion`] cross-references
//!    every candidate id against the BSS boot-parameter records.
//! 2. Deleting an image referenced by a node outside the caller's
//!    accessible groups would let users indirectly remove resources
//!    they don't own.
//!
//! Read-only listing ([`get_images`]) only validates the requested
//! `pattern` glob and applies the `limit` cap.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
use crate::service::boot_parameters::get_restricted_boot_parameters;
pub use manta_shared::types::api::image::GetImagesParams;

/// Fetch IMS images from the backend, sorted by creation time.
///
/// Filters server-side by `params.pattern` (glob syntax, matched
/// against `image.name`) and caps the result at `params.limit`.
///
/// An invalid glob (unbalanced bracket, malformed range, …) returns
/// [`Error::BadRequest`] with the parser's message; the caller's
/// handler layer maps that to HTTP 400.
pub async fn get_images(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetImagesParams,
) -> Result<Vec<Image>, Error> {
  let mut image_vec = infra
    .backend
    .get_images(token, params.id.as_deref())
    .await?;

  image_vec = apply_pattern_filter(image_vec, params.pattern.as_deref())?;

  if let Some(limit) = params.limit {
    image_vec.truncate(limit as usize);
  }

  image_vec.sort_by_key(|image| image.created.clone());

  Ok(image_vec)
}

/// Pure helper that retains only images whose `name` matches `pattern`
/// (glob syntax). `None` pattern is a no-op pass-through. Split out so
/// the filter can be unit-tested without standing up an
/// `InfraContext` / backend mock.
fn apply_pattern_filter(
  image_vec: Vec<Image>,
  pattern: Option<&str>,
) -> Result<Vec<Image>, Error> {
  let Some(pattern) = pattern else {
    return Ok(image_vec);
  };
  let matcher = globset::Glob::new(pattern)
    .map_err(|e| {
      Error::BadRequest(format!("invalid glob pattern '{pattern}': {e}"))
    })?
    .compile_matcher();
  Ok(
    image_vec
      .into_iter()
      .filter(|img| matcher.is_match(&img.name))
      .collect(),
  )
}

/// Refuse a planned image delete that would orphan a live boot path
/// or touch an image scoped to a group the caller can't reach.
///
/// Two checks run after access validation: any image listed in
/// `image_id_vec` that is the current boot image of an existing BSS
/// record fails with `BadRequest` (deleting it would brick the next
/// boot); any image whose boot record targets hosts outside the
/// caller's available groups fails the same way (so a user can't
/// indirectly remove an image they don't own through a shared id).
/// Pure check — no deletion happens here.
pub async fn validate_image_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  settings_group_name_opt: Option<&str>,
) -> Result<(), Error> {
  // One backend fetch + in-memory validation, replacing the prior
  // three round-trips. See `service::group::resolve_target_and_available_groups`.
  let (group_available_vec, _target_group_vec) =
    crate::service::group::resolve_target_and_available_groups(
      infra,
      token,
      settings_group_name_opt,
    )
    .await?;

  let boot_parameter_vec = infra.backend.get_all_bootparameters(token).await?;

  // Check if any requested image is used to boot nodes
  let image_used_to_boot_nodes: Vec<String> = boot_parameter_vec
    .iter()
    .filter_map(manta_backend_dispatcher::types::bss::BootParameters::try_get_boot_image_id)
    .collect();

  // `image_used_to_boot_nodes` is cluster-scale (one entry per BSS
  // record). Hash it once so the safety check across user-supplied
  // delete ids is O(D) rather than O(D·N).
  let image_used_to_boot_nodes_set: std::collections::HashSet<&str> =
    image_used_to_boot_nodes
      .iter()
      .map(String::as_str)
      .collect();
  let image_xnames_boot_map: Vec<&&str> = image_id_vec
    .iter()
    .filter(|id| image_used_to_boot_nodes_set.contains(**id))
    .collect();

  if !image_xnames_boot_map.is_empty() {
    return Err(Error::BadRequest(format!(
      "The following images could not be deleted \
       since they boot nodes.\n{}",
      image_xnames_boot_map
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
    )));
  }

  // Check restricted images
  let image_restricted_vec =
    get_restricted_image_ids(&group_available_vec, &boot_parameter_vec);

  if !image_restricted_vec.is_empty() {
    return Err(Error::BadRequest(format!(
      "The following image ids can't be deleted \
       because they are used by hosts that are not part \
       of the groups available to the user:\n{}",
      image_restricted_vec.join(", ")
    )));
  }

  Ok(())
}

/// Run [`validate_image_deletion`] then delete each image in
/// `image_id_vec`, best-effort.
///
/// Individual delete failures are logged and skipped — the function
/// keeps going so a single backend hiccup doesn't strand the rest of
/// the batch. The returned vector lists exactly the ids the backend
/// confirmed removed.
pub async fn delete_images(
  infra: &InfraContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<Vec<String>, Error> {
  validate_image_deletion(
    infra,
    token,
    image_id_vec,
    settings_hsm_group_name_opt,
  )
  .await?;

  let mut deleted = Vec::new();
  for image_id in image_id_vec {
    match infra.backend.delete_image(token, image_id).await {
      Ok(()) => {
        tracing::info!("Image {} deleted successfully", image_id);
        deleted.push((*image_id).to_string());
      }
      Err(e) => tracing::error!(
        "Failed to delete image {}: {}. Continuing",
        image_id,
        e
      ),
    }
  }

  Ok(deleted)
}

fn get_restricted_image_ids(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Vec<String> {
  get_restricted_boot_parameters(group_available_vec, boot_parameter_vec)
    .iter()
    .filter_map(manta_backend_dispatcher::types::bss::BootParameters::try_get_boot_image_id)
    .collect()
}

#[cfg(test)]
mod tests {
  //! Unit tests for the pure `apply_pattern_filter` helper. The
  //! async wrapper `get_images` adds no logic beyond glue, so testing
  //! the helper covers the behaviour: pattern compilation, name
  //! matching, and the BadRequest path on invalid globs.

  use super::apply_pattern_filter;
  use manta_backend_dispatcher::error::Error;
  use manta_backend_dispatcher::types::ims::Image;

  fn image(name: &str) -> Image {
    Image {
      name: name.to_string(),
      ..Default::default()
    }
  }

  #[test]
  fn no_pattern_returns_all_images_unchanged() {
    let input = vec![image("a"), image("b"), image("c")];
    let out = apply_pattern_filter(input.clone(), None).expect("None is no-op");
    assert_eq!(out.len(), 3);
    assert_eq!(out[0].name, "a");
    assert_eq!(out[2].name, "c");
  }

  #[test]
  fn star_glob_matches_everything() {
    let input = vec![image("compute-a"), image("login-b")];
    let out = apply_pattern_filter(input, Some("*")).expect("'*' is valid");
    assert_eq!(out.len(), 2);
  }

  #[test]
  fn prefix_star_keeps_only_matching_subset() {
    let input = vec![
      image("compute-a"),
      image("compute-b"),
      image("login-a"),
      image("storage-3"),
    ];
    let out = apply_pattern_filter(input, Some("compute-*"))
      .expect("'compute-*' valid");
    assert_eq!(out.len(), 2);
    assert!(out.iter().all(|i| i.name.starts_with("compute-")));
  }

  #[test]
  fn pattern_with_no_matches_returns_empty() {
    let input = vec![image("compute-a"), image("login-b")];
    let out = apply_pattern_filter(input, Some("nomatch-*"))
      .expect("'nomatch-*' is valid even when nothing matches");
    assert!(out.is_empty());
  }

  #[test]
  fn invalid_glob_returns_bad_request() {
    let input = vec![image("anything")];
    let err = apply_pattern_filter(input, Some("[unclosed"))
      .expect_err("'[unclosed' is malformed");
    match err {
      Error::BadRequest(msg) => {
        assert!(
          msg.contains("invalid glob pattern"),
          "error message should explain the glob is bad; got: {msg}"
        );
        assert!(
          msg.contains("'[unclosed'"),
          "error should quote the offending pattern; got: {msg}"
        );
      }
      other => panic!("expected BadRequest, got {other:?}"),
    }
  }

  #[test]
  fn question_mark_matches_single_char() {
    // Lock the globset semantics for `?`: matches exactly one
    // character. If we ever swap libraries, this test will fail
    // and force a deliberate decision rather than silent drift.
    let input = vec![
      image("a"),    // 1 char — no match (pattern needs >=2)
      image("ab"),   // 2 chars — match
      image("abc"),  // 3 chars — match
      image("abcd"), // 4 chars — no match
    ];
    let out = apply_pattern_filter(input, Some("a??")).expect("'a??' is valid");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "abc");
  }

  #[test]
  fn character_class_matches_any_listed_char() {
    let input = vec![
      image("compute-a"),
      image("compute-b"),
      image("compute-c"),
      image("compute-d"),
    ];
    let out =
      apply_pattern_filter(input, Some("compute-[abc]")).expect("class valid");
    assert_eq!(out.len(), 3);
    assert!(!out.iter().any(|i| i.name == "compute-d"));
  }
}
