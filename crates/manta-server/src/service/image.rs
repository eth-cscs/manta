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
//! Read-only listing ([`get_images`]) validates the requested
//! `pattern` glob and `since`/`until` range, filters on both, selects
//! the newest `limit` images, and returns them oldest-first (newest
//! last). IMS takes no query parameters of its own, so every filter
//! runs here.

use std::cmp::Reverse;

use chrono::NaiveDateTime;
use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;
use manta_shared::common::parse_ims_timestamp;

use crate::server::common::app_context::InfraContext;
use crate::service::boot_parameters::get_restricted_boot_parameters;
use crate::service::configuration::validate_date_range;
pub use manta_shared::types::api::image::GetImagesParams;

/// Fetch IMS images from the backend, oldest-first (newest last).
///
/// Filters server-side by `params.pattern` (glob syntax, matched
/// against `image.name`) and by the `params.since` / `params.until`
/// creation-date bounds, then selects the newest `params.limit` images
/// and returns them oldest-first for display. Selection happens before
/// the display flip, so `limit = 1` yields the single newest image (see
/// `sort_and_cap`).
///
/// Both filters run here rather than at the backend because IMS
/// accepts no query parameters at all — `ImsTrait::get_images` takes
/// only an optional image id.
///
/// An invalid glob (unbalanced bracket, malformed range, …) or a
/// `since` later than `until` returns [`Error::BadRequest`]; the
/// caller's handler layer maps that to HTTP 400.
pub async fn get_images(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetImagesParams,
) -> Result<Vec<Image>, Error> {
  validate_date_range(params.since, params.until)?;

  let mut image_vec = infra
    .backend
    .get_images(token, params.id.as_deref())
    .await?;

  image_vec = apply_pattern_filter(image_vec, params.pattern.as_deref())?;
  image_vec = apply_date_filter(image_vec, params.since, params.until);

  Ok(sort_and_cap(image_vec, params.limit))
}

/// Pure helper retaining only images created within `[since, until]`.
///
/// Both bounds are inclusive and independently optional; `None` for
/// both is a no-op pass-through.
///
/// An image whose `created` is absent or unparseable is **dropped**
/// whenever either bound is set: it cannot be shown to satisfy
/// "created after X", so including it would misreport the filter the
/// caller asked for. This matches how `manta get images` already
/// treats an unknown `safe_to_delete` verdict under
/// `--only-safe-to-delete`.
///
/// A zoned `created` is normalised to naive local time (see
/// [`parse_ims_timestamp`]) before comparison, using **this process's**
/// timezone — i.e. the server's. The CLI renders the Creation time
/// column the same way but in the *client's* timezone, so for an
/// offset-bearing `created` a displayed time can differ from the value
/// filtered against when client and server timezones disagree. The
/// server-side comparison is authoritative; the two agree on which
/// timestamps are readable, not necessarily on their wall-clock value.
fn apply_date_filter(
  image_vec: Vec<Image>,
  since: Option<NaiveDateTime>,
  until: Option<NaiveDateTime>,
) -> Vec<Image> {
  if since.is_none() && until.is_none() {
    return image_vec;
  }

  image_vec
    .into_iter()
    .filter(|img| {
      let Some(created) = img.created.as_deref().and_then(parse_ims_timestamp)
      else {
        return false;
      };
      since.is_none_or(|s| created >= s) && until.is_none_or(|u| created <= u)
    })
    .collect()
}

/// Select the newest `limit` images and return them oldest-first
/// (newest last).
///
/// Three steps whose order is the whole contract:
/// 1. sort newest-first, so
/// 2. `truncate(limit)` keeps the newest N — capping the backend's
///    arbitrary order instead would make `limit = 1` return an
///    arbitrary image rather than the most recent one; then
/// 3. `reverse()` flips the kept images to oldest-first for display, so
///    the newest lands at the bottom of the listing.
///
/// Selection (newest N) and display order (oldest-first) are decoupled
/// on purpose: `--limit` / `--most-recent` still pick the most recent
/// images, they're just printed with the newest at the end — matching
/// `get configurations` / `get sessions`, whose csm-rs listings are
/// already ascending. Keeping all three steps in one pure function
/// makes the contract testable without standing up a backend mock.
///
/// The sort key is the *parsed* timestamp ([`parse_ims_timestamp`]),
/// not the raw string: `created` arrives in a shape CSM does not
/// guarantee, and a lexicographic order only matches chronology when
/// every value shares one shape (mixed zoned/naive break it). `Reverse`
/// gives the newest-first working order and sends absent/unparseable
/// dates (`None`) to the end — so after the final `reverse` they sit at
/// the *top*, treated as oldest, as `get sessions` treats undated rows.
/// `sort_by_cached_key` parses each `created` once — not `O(n log n)`
/// times as a comparator would — and is stable, so equal or unplaceable
/// images keep the backend's order.
fn sort_and_cap(mut image_vec: Vec<Image>, limit: Option<u8>) -> Vec<Image> {
  image_vec.sort_by_cached_key(|image| {
    Reverse(image.created.as_deref().and_then(parse_ims_timestamp))
  });

  if let Some(limit) = limit {
    image_vec.truncate(limit as usize);
  }

  image_vec.reverse();

  image_vec
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
  //! Unit tests for the pure helpers behind `get_images`:
  //! `apply_pattern_filter` (pattern compilation, name matching, and
  //! the BadRequest path on invalid globs), `apply_date_filter`
  //! (inclusive since/until bounds and unusable dates), and
  //! `sort_and_cap` (most-recent-first ordering, and the cap applying
  //! only after it).
  //!
  //! These call the same functions `get_images` calls, so the
  //! order-of-operations contract is genuinely covered here rather
  //! than restated.

  use super::{apply_date_filter, apply_pattern_filter, sort_and_cap};
  use chrono::NaiveDateTime;
  use manta_backend_dispatcher::error::Error;
  use manta_backend_dispatcher::types::ims::Image;

  fn image(name: &str) -> Image {
    Image {
      name: name.to_string(),
      ..Default::default()
    }
  }

  fn image_created(name: &str, created: Option<&str>) -> Image {
    Image {
      name: name.to_string(),
      created: created.map(str::to_string),
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

  #[test]
  fn images_are_ordered_oldest_first() {
    let input = vec![
      image_created("middle", Some("2026-03-01T00:00:00")),
      image_created("oldest", Some("2026-01-01T00:00:00")),
      image_created("newest", Some("2026-06-01T00:00:00")),
    ];
    let out = sort_and_cap(input, None);
    let names: Vec<&str> = out.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(
      names,
      ["oldest", "middle", "newest"],
      "listing is oldest-first so the newest image lands at the bottom"
    );
  }

  #[test]
  fn limit_one_keeps_the_newest_not_the_first() {
    // Regression: the cap used to be applied *before* the sort, so
    // `--most-recent` returned whatever the backend happened to list
    // first. Selection is still "newest N" even though display is now
    // oldest-first — with limit 1 the single row must be the newest.
    // "newest" is deliberately first in backend order to catch a naive
    // "take the front" cap.
    let input = vec![
      image_created("newest", Some("2026-06-01T00:00:00")),
      image_created("middle", Some("2026-03-01T00:00:00")),
      image_created("oldest", Some("2026-01-01T00:00:00")),
    ];
    let out = sort_and_cap(input, Some(1));
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "newest");
  }

  #[test]
  fn limit_keeps_the_newest_n_shown_oldest_first() {
    // `--limit 2` selects the two newest, then displays them
    // oldest-first: "middle" above "newest", and "oldest" dropped.
    let input = vec![
      image_created("oldest", Some("2026-01-01T00:00:00")),
      image_created("newest", Some("2026-06-01T00:00:00")),
      image_created("middle", Some("2026-03-01T00:00:00")),
    ];
    let out = sort_and_cap(input, Some(2));
    let names: Vec<&str> = out.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["middle", "newest"]);
  }

  #[test]
  fn zoned_timestamps_take_part_in_the_ordering() {
    // A zoned `created` must be parsed and placed on the timeline, not
    // written off as unparseable. The months-wide gaps keep this
    // independent of the runner's timezone: `parse_ims_timestamp`
    // normalises to local naive time, so a zoned value shifts by at
    // most ±14h — nowhere near enough to escape the surrounding pair.
    let input = vec![
      image_created("zoned", Some("2026-06-04T12:30:00+00:00")),
      image_created("newest", Some("2026-12-01T00:00:00")),
      image_created("oldest", Some("2026-01-01T00:00:00")),
    ];
    let out = sort_and_cap(input, None);
    let names: Vec<&str> = out.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["oldest", "zoned", "newest"]);
  }

  fn ts(raw: &str) -> NaiveDateTime {
    raw.parse().expect("test timestamp is well-formed")
  }

  fn dated_fixture() -> Vec<Image> {
    vec![
      image_created("jan", Some("2026-01-15T00:00:00")),
      image_created("mar", Some("2026-03-15T00:00:00")),
      image_created("jun", Some("2026-06-15T00:00:00")),
    ]
  }

  fn names(image_vec: &[Image]) -> Vec<&str> {
    image_vec.iter().map(|i| i.name.as_str()).collect()
  }

  #[test]
  fn no_bounds_is_a_no_op() {
    let out = apply_date_filter(dated_fixture(), None, None);
    assert_eq!(names(&out), ["jan", "mar", "jun"]);
  }

  #[test]
  fn since_keeps_only_images_at_or_after_the_bound() {
    let out =
      apply_date_filter(dated_fixture(), Some(ts("2026-03-01T00:00:00")), None);
    assert_eq!(names(&out), ["mar", "jun"]);
  }

  #[test]
  fn until_keeps_only_images_at_or_before_the_bound() {
    let out =
      apply_date_filter(dated_fixture(), None, Some(ts("2026-03-31T00:00:00")));
    assert_eq!(names(&out), ["jan", "mar"]);
  }

  #[test]
  fn both_bounds_select_the_window() {
    let out = apply_date_filter(
      dated_fixture(),
      Some(ts("2026-02-01T00:00:00")),
      Some(ts("2026-04-01T00:00:00")),
    );
    assert_eq!(names(&out), ["mar"]);
  }

  #[test]
  fn bounds_are_inclusive_on_both_ends() {
    // An image created exactly on the bound is kept — `--since X
    // --until X` must not return an empty list for an image at X.
    let exact = ts("2026-03-15T00:00:00");
    let out = apply_date_filter(dated_fixture(), Some(exact), Some(exact));
    assert_eq!(names(&out), ["mar"]);
  }

  #[test]
  fn images_with_unusable_dates_are_dropped_when_filtering() {
    let input = vec![
      image_created("dated", Some("2026-03-15T00:00:00")),
      image_created("no-date", None),
      image_created("bad-date", Some("not-a-real-date")),
    ];
    let out = apply_date_filter(input, Some(ts("2026-01-01T00:00:00")), None);
    assert_eq!(
      names(&out),
      ["dated"],
      "an image with no usable creation date cannot satisfy a date bound"
    );
  }

  #[test]
  fn images_with_unusable_dates_survive_when_not_filtering() {
    // The drop above is a consequence of filtering, not a general
    // rule: an unfiltered listing must still show them.
    let input = vec![image_created("no-date", None)];
    let out = apply_date_filter(input, None, None);
    assert_eq!(names(&out), ["no-date"]);
  }

  #[test]
  fn images_without_a_parseable_date_sort_first() {
    // Unplaceable images (missing or unparseable `created`) are treated
    // as oldest, so in the oldest-first listing they sit at the top,
    // above every real date — the newest-last row stays at the bottom.
    // Also pins that ordering is by *parsed* value, not raw string:
    // "not-a-real-date" starts with 'n', so a naive string sort could
    // misplace it relative to the "2026-..." timestamp.
    let input = vec![
      image_created("no-date", None),
      image_created("bad-date", Some("not-a-real-date")),
      image_created("dated", Some("2026-01-01T00:00:00")),
    ];
    let out = sort_and_cap(input, None);
    assert_eq!(
      out.last().unwrap().name,
      "dated",
      "the only real date is the newest, so it lands at the bottom"
    );
    // The two undated rows lead. `sort_by_cached_key` is stable, so
    // before the display flip they held backend order (no-date,
    // bad-date); the final `reverse` flips the whole list, undated rows
    // included, so they appear reversed here.
    let head: Vec<&str> = out[..2].iter().map(|i| i.name.as_str()).collect();
    assert_eq!(head, ["bad-date", "no-date"]);
  }
}
