//! Cross-resource analyses that fan IMS / CFS / BSS fetches and link
//! the results in a pure helper.
//!
//! - [`get_image_analysis`] + [`build_cache`] — image-centric flat
//!   projection; one row per IMS image with a `safe_to_delete` verdict
//!   derived from BSS boot-parameter references. See [`BackendSummary`].
//! - [`build_configuration_analysis`] — pure linker that derives a
//!   `safe_to_delete` verdict per CFS configuration from CFS components
//!   and (optionally) BSS-referenced images. Called from
//!   `service::configuration::get_configurations_with_analysis` (the
//!   components-only variant served at `/configurations`).

use std::collections::{HashMap, HashSet};

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::component::Component as CfsComponent;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
pub use manta_shared::types::api::analysis::BackendSummary;
pub use manta_shared::types::api::configuration_analysis::ConfigurationAnalysis;

/// Pure linker.
pub fn build_cache(
  boot_params: Vec<BootParameters>,
  images: Vec<Image>,
) -> Vec<BackendSummary> {
  // Set of image ids that BSS boot-parameter records currently point
  // at. An image referenced by BSS is the boot image for at least one
  // node, so deleting it would break that node's next boot.
  let bss_boot_image_ids: HashSet<String> = boot_params
    .iter()
    .filter_map(BootParameters::try_get_boot_image_id)
    .collect();

  let mut rows: Vec<BackendSummary> = images
    .into_iter()
    .filter_map(|img| {
      let id = img.id?;
      let safe_to_delete = !bss_boot_image_ids.contains(&id);
      Some(BackendSummary {
        image_id: id,
        name: img.name,
        image_created: img.created,
        configuration_name: img.configuration,
        safe_to_delete,
      })
    })
    .collect();

  // Primary: image_created ascending (oldest first). Secondary: image_id
  // ascending for a deterministic tie-break (and as the only ordering when
  // created is missing on both sides). Images without a created timestamp
  // sink to the bottom.
  rows.sort_by(|a, b| {
    use std::cmp::Ordering;
    match (a.image_created.as_ref(), b.image_created.as_ref()) {
      (Some(ac), Some(bc)) => {
        ac.cmp(bc).then_with(|| a.image_id.cmp(&b.image_id))
      }
      (Some(_), None) => Ordering::Less,
      (None, Some(_)) => Ordering::Greater,
      (None, None) => a.image_id.cmp(&b.image_id),
    }
  });
  rows
}

/// Sequence the two upstream fetches and run the pure linker.
///
/// Sequenced rather than concurrent: `get_all_bootparameters` returns
/// a cluster-scale list, and fanning two heavy fetches at the same
/// upstream is the shape that produced upstream connection-resets on
/// the configuration variant.
///
/// # Errors
///
/// - Backend errors from `get_all_bootparameters` (BSS upstream
///   failure, auth, etc.) propagate as-is.
/// - Backend / IMS errors from
///   [`crate::service::image::get_images`] propagate as-is.
pub async fn get_image_analysis(
  infra: &InfraContext<'_>,
  token: &str,
) -> Result<Vec<BackendSummary>, Error> {
  tracing::info!("Building image analysis");

  let images_params = crate::service::image::GetImagesParams {
    id: None,
    pattern: None,
    limit: None,
  };

  let boot_params = infra.backend.get_all_bootparameters(token).await?;
  let images =
    crate::service::image::get_images(infra, token, &images_params).await?;

  Ok(build_cache(boot_params, images))
}

/// Pure linker for the configuration-deletion-safety analysis.
///
/// A configuration is flagged unsafe to delete if either:
/// 1. some CFS component lists it as `desired_config`, or
/// 2. some IMS image built from it is the boot image of any BSS
///    boot-parameter record.
///
/// The output is one row per configuration in `configs`, sorted by
/// `last_updated` ascending (oldest first); ties on the timestamp
/// break by `name` ascending.
pub fn build_configuration_analysis(
  mut configs: Vec<CfsConfigurationResponse>,
  components: Vec<CfsComponent>,
  boot_params: Vec<BootParameters>,
  images: Vec<Image>,
) -> Vec<ConfigurationAnalysis> {
  // Configs that are some component's desired_config.
  let mut unsafe_configs: HashSet<String> = components
    .iter()
    .filter_map(|c| c.desired_config.clone())
    .collect();

  // Image_id -> configuration name used to build it.
  let image_id_to_config: HashMap<String, String> = images
    .into_iter()
    .filter_map(|img| match (img.id, img.configuration) {
      (Some(id), Some(cfg)) => Some((id, cfg)),
      _ => None,
    })
    .collect();

  // Add configs that produced any BSS-referenced image.
  for bp in &boot_params {
    if let Some(image_id) = bp.try_get_boot_image_id()
      && let Some(cfg) = image_id_to_config.get(&image_id)
    {
      unsafe_configs.insert(cfg.clone());
    }
  }

  configs.sort_by(|a, b| {
    a.last_updated
      .cmp(&b.last_updated)
      .then_with(|| a.name.cmp(&b.name))
  });

  configs
    .into_iter()
    .map(|c| {
      let safe_to_delete = !unsafe_configs.contains(&c.name);
      ConfigurationAnalysis {
        configuration: c,
        safe_to_delete,
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn image(
    id: &str,
    name: &str,
    config: Option<&str>,
    created: Option<&str>,
  ) -> Image {
    Image {
      id: Some(id.to_string()),
      name: name.to_string(),
      created: created.map(String::from),
      link: None,
      arch: None,
      metadata: None,
      groups: None,
      base: None,
      configuration: config.map(String::from),
    }
  }

  fn config(name: &str, last_updated: &str) -> CfsConfigurationResponse {
    CfsConfigurationResponse {
      name: name.to_string(),
      last_updated: last_updated.to_string(),
      layers: vec![],
      additional_inventory: None,
    }
  }

  fn component(id: &str, desired_config: Option<&str>) -> CfsComponent {
    CfsComponent {
      id: Some(id.to_string()),
      state: None,
      desired_config: desired_config.map(String::from),
      error_count: None,
      retry_policy: None,
      enabled: None,
      configuration_status: None,
      tags: None,
      logs: None,
    }
  }

  /// BSS boot-parameter record with a kernel S3 path that points at
  /// `image_id`. `try_get_boot_image_id` parses `root` (CN) or
  /// `metal.server` (NCN) from `params`; we set `root` here.
  fn boot_param_for_image(image_id: &str) -> BootParameters {
    BootParameters {
      hosts: vec![],
      macs: None,
      nids: None,
      params: format!("root=s3://boot-images/{image_id}/rootfs"),
      kernel: format!("s3://boot-images/{image_id}/kernel"),
      initrd: format!("s3://boot-images/{image_id}/initrd"),
      cloud_init: None,
    }
  }

  // image_id + name + configuration_name come from Image directly.
  #[test]
  fn anchors_one_row_per_image_with_built_with_configuration() {
    let rows = build_cache(
      vec![],
      vec![
        image("img-1", "ncn-1.6-base", Some("ncn-1.6"), None),
        image("img-2", "compute-1.5", Some("compute-1.5"), None),
      ],
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].image_id, "img-1");
    assert_eq!(rows[0].name, "ncn-1.6-base");
    assert_eq!(rows[0].configuration_name.as_deref(), Some("ncn-1.6"));
    assert_eq!(rows[1].image_id, "img-2");
  }

  // Orphan image: nothing references it. Row exists, `safe_to_delete`
  // is true, every Option column is None.
  #[test]
  fn orphan_image_is_safe_to_delete() {
    let rows = build_cache(vec![], vec![image("img-1", "orphan", None, None)]);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.image_id, "img-1");
    assert!(row.image_created.is_none());
    assert!(row.configuration_name.is_none());
    assert!(row.safe_to_delete);
  }

  // When no image has a created timestamp, sort falls back to image_id
  // ascending so output stays deterministic across runs.
  #[test]
  fn rows_with_no_created_timestamp_fall_back_to_image_id_asc() {
    let rows = build_cache(
      vec![],
      vec![
        image("img-z", "z", None, None),
        image("img-a", "a", None, None),
        image("img-m", "m", None, None),
      ],
    );
    let ids: Vec<&str> = rows.iter().map(|r| r.image_id.as_str()).collect();
    assert_eq!(ids, vec!["img-a", "img-m", "img-z"]);
  }

  // Primary sort: image_created ascending (oldest first). Images without
  // a created timestamp sink to the bottom; ties on created (or both None)
  // break by image_id ascending.
  #[test]
  fn rows_are_sorted_by_image_created_ascending() {
    let rows = build_cache(
      vec![],
      vec![
        image("img-old", "old", None, Some("2024-01-01T00:00:00Z")),
        image("img-newest", "newest", None, Some("2026-06-02T00:00:00Z")),
        image("img-undated-z", "undated-z", None, None),
        image("img-middle", "middle", None, Some("2026-06-01T00:00:00Z")),
        image("img-undated-a", "undated-a", None, None),
      ],
    );
    let ids: Vec<&str> = rows.iter().map(|r| r.image_id.as_str()).collect();
    assert_eq!(
      ids,
      vec![
        "img-old",       // 2024-01-01
        "img-middle",    // 2026-06-01
        "img-newest",    // 2026-06-02
        "img-undated-a", // None, id asc tie-break
        "img-undated-z", // None, id asc tie-break
      ]
    );
  }

  #[test]
  fn empty_input_yields_empty_cache() {
    let rows = build_cache(vec![], vec![]);
    assert!(rows.is_empty());
  }

  // Image referenced as the boot image of a BSS record is unsafe to
  // delete; the unreferenced sibling stays safe.
  #[test]
  fn bss_referenced_image_is_unsafe_to_delete() {
    let rows = build_cache(
      vec![boot_param_for_image("img-booted")],
      vec![
        image("img-booted", "in-use", None, None),
        image("img-orphan", "spare", None, None),
      ],
    );
    let booted = rows.iter().find(|r| r.image_id == "img-booted").unwrap();
    let orphan = rows.iter().find(|r| r.image_id == "img-orphan").unwrap();
    assert!(!booted.safe_to_delete);
    assert!(orphan.safe_to_delete);
  }

  // ------------------------------------------------------------------
  // build_configuration_analysis
  // ------------------------------------------------------------------

  #[test]
  fn configuration_analysis_orphan_config_is_safe_to_delete() {
    let rows = build_configuration_analysis(
      vec![config("orphan", "2025-01-01T00:00:00Z")],
      vec![],
      vec![],
      vec![],
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].configuration.name, "orphan");
    assert_eq!(rows[0].configuration.last_updated, "2025-01-01T00:00:00Z");
    assert!(rows[0].safe_to_delete);
  }

  #[test]
  fn configuration_analysis_desired_by_component_is_unsafe() {
    let rows = build_configuration_analysis(
      vec![
        config("desired", "2025-01-01T00:00:00Z"),
        config("nobody-cares", "2025-01-02T00:00:00Z"),
      ],
      vec![component("x1000c0s0b0n0", Some("desired"))],
      vec![],
      vec![],
    );
    let desired = rows
      .iter()
      .find(|r| r.configuration.name == "desired")
      .unwrap();
    let other = rows
      .iter()
      .find(|r| r.configuration.name == "nobody-cares")
      .unwrap();
    assert!(!desired.safe_to_delete);
    assert!(other.safe_to_delete);
  }

  #[test]
  fn configuration_analysis_bss_referenced_image_makes_config_unsafe() {
    let rows = build_configuration_analysis(
      vec![
        config("boot-config", "2025-01-01T00:00:00Z"),
        config("nobody-cares", "2025-01-02T00:00:00Z"),
      ],
      vec![],
      vec![boot_param_for_image("img-bsst")],
      vec![image("img-bsst", "boot-img", Some("boot-config"), None)],
    );
    let boot = rows
      .iter()
      .find(|r| r.configuration.name == "boot-config")
      .unwrap();
    let other = rows
      .iter()
      .find(|r| r.configuration.name == "nobody-cares")
      .unwrap();
    assert!(!boot.safe_to_delete);
    assert!(other.safe_to_delete);
  }

  #[test]
  fn configuration_analysis_bss_pointing_at_unknown_image_does_not_flag() {
    // BSS references an image the IMS listing does not return.
    // Without that image we can't resolve to a configuration, so
    // every config stays safe.
    let rows = build_configuration_analysis(
      vec![config("c1", "2025-01-01T00:00:00Z")],
      vec![],
      vec![boot_param_for_image("missing-image")],
      vec![],
    );
    assert!(rows[0].safe_to_delete);
  }

  #[test]
  fn configuration_analysis_rows_sorted_by_last_updated_asc_then_name() {
    let rows = build_configuration_analysis(
      vec![
        config("z", "2025-06-01T00:00:00Z"),
        config("a", "2024-01-01T00:00:00Z"),
        config("b", "2025-06-01T00:00:00Z"),
      ],
      vec![],
      vec![],
      vec![],
    );
    let names: Vec<&str> =
      rows.iter().map(|r| r.configuration.name.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "z"]); // oldest first; ties by name asc
  }

  #[test]
  fn configuration_analysis_components_without_desired_config_are_ignored() {
    let rows = build_configuration_analysis(
      vec![config("c1", "2025-01-01T00:00:00Z")],
      vec![component("x1000c0s0b0n0", None)],
      vec![],
      vec![],
    );
    assert!(rows[0].safe_to_delete);
  }
}
