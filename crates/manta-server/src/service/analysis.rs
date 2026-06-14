//! Cross-resource analyses that fan IMS / CFS / BOS / BSS fetches
//! out concurrently and link the results in a pure helper.
//!
//! - [`get_cache`] + [`build_cache`] — image-centric flat projection;
//!   one row per IMS image, see [`BackendSummary`] for fields.
//! - [`get_configuration_analysis`] + [`build_configuration_analysis`]
//!   — configuration-deletion safety; one row per CFS configuration,
//!   see [`ConfigurationAnalysis`] for fields.

use std::collections::{HashMap, HashSet};

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::{
  bss::BootParametersTrait, cfs::CfsTrait,
};
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::component::Component as CfsComponent;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
pub use manta_shared::types::api::analysis::BackendSummary;
pub use manta_shared::types::api::configuration_analysis::ConfigurationAnalysis;

/// Pure linker.
pub fn build_cache(
  mut sessions: Vec<CfsSessionGetResponse>,
  mut templates: Vec<BosSessionTemplate>,
  images: Vec<Image>,
) -> Vec<BackendSummary> {
  // Stable ordering up front so the "first matching session/template
  // wins" rule is deterministic across runs.
  sessions.sort_by(|a, b| a.name.cmp(&b.name));
  templates.sort_by(|a, b| {
    a.name
      .as_deref()
      .unwrap_or("")
      .cmp(b.name.as_deref().unwrap_or(""))
  });

  // Reverse index: image_id -> (first session that produced it, that
  // session's configuration name).
  let mut session_by_image: HashMap<String, (String, Option<String>)> =
    HashMap::new();
  for s in &sessions {
    let cfg = s.configuration.as_ref().and_then(|c| c.name.clone());
    for result_id in s.get_result_id_vec() {
      session_by_image
        .entry(result_id)
        .or_insert_with(|| (s.name.clone(), cfg.clone()));
    }
  }

  // Reverse index: image_id -> first template that boots from it.
  let mut template_by_image: HashMap<String, String> = HashMap::new();
  for t in &templates {
    let Some(name) = t.name.clone() else {
      continue;
    };
    for boot_image_id in template_boot_image_ids(t) {
      template_by_image
        .entry(boot_image_id)
        .or_insert_with(|| name.clone());
    }
  }

  // One row per image with a usable id.
  let mut rows: Vec<BackendSummary> = images
    .into_iter()
    .filter_map(|img| {
      let id = img.id?;
      let (session_name, session_configuration_name) =
        match session_by_image.get(&id).cloned() {
          Some((sn, scn)) => (Some(sn), scn),
          None => (None, None),
        };
      let session_result_id = session_name.as_ref().map(|_| id.clone());
      let bos_sessiontemplate = template_by_image.get(&id).cloned();
      let bos_sessiontemplate_boot_image =
        bos_sessiontemplate.as_ref().map(|_| id.clone());
      Some(BackendSummary {
        image_id: id,
        name: img.name,
        image_created: img.created,
        configuration_name: img.configuration,
        session_name,
        session_result_id,
        session_configuration_name,
        bos_sessiontemplate,
        bos_sessiontemplate_boot_image,
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

/// Panic-safe replacement for `BosSessionTemplate::get_image_vec`,
/// which unwraps `boot_sets`. Templates with `boot_sets: None` yield
/// an empty vec instead of panicking.
fn template_boot_image_ids(t: &BosSessionTemplate) -> Vec<String> {
  t.boot_sets
    .as_ref()
    .map(|bs| {
      bs.values()
        .filter_map(|b| b.path.as_ref())
        .map(|p| {
          p.trim_start_matches("s3://boot-images/")
            .trim_end_matches("/manifest.json")
            .to_string()
        })
        .collect()
    })
    .unwrap_or_default()
}

/// Fan the three service-layer fetchers out concurrently, then run
/// the pure linker. Each fetcher applies its own group-access scope,
/// so the cache cannot return rows the caller couldn't list via
/// the per-resource endpoints.
pub async fn get_image_analysis(
  infra: &InfraContext<'_>,
  token: &str,
) -> Result<Vec<BackendSummary>, Error> {
  tracing::info!("Building backend cache");

  let sessions_params = crate::service::session::GetSessionParams {
    group: None,
    xnames: Vec::new(),
    min_age: None,
    max_age: None,
    session_type: None,
    status: None,
    name: None,
    limit: None,
  };
  let templates_params = crate::service::template::GetTemplateParams {
    name: None,
    group_name: None,
    settings_group_name: None,
    limit: None,
  };
  let images_params = crate::service::image::GetImagesParams {
    id: None,
    pattern: None,
    limit: None,
  };

  let (sessions, templates, images) = tokio::try_join!(
    crate::service::session::get_sessions(infra, token, &sessions_params),
    crate::service::template::get_templates(infra, token, &templates_params),
    crate::service::image::get_images(infra, token, &images_params),
  )?;

  Ok(build_cache(sessions, templates, images))
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
    if let Some(image_id) = bp.try_get_boot_image_id() {
      if let Some(cfg) = image_id_to_config.get(&image_id) {
        unsafe_configs.insert(cfg.clone());
      }
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
        name: c.name,
        last_updated: c.last_updated,
        safe_to_delete,
      }
    })
    .collect()
}

/// Fan the four required fetchers (configurations, CFS components,
/// BSS boot-parameters, IMS images) out concurrently and feed the
/// pure linker.
pub async fn get_configuration_analysis(
  infra: &InfraContext<'_>,
  token: &str,
) -> Result<Vec<ConfigurationAnalysis>, Error> {
  tracing::info!("Building configuration-deletion safety analysis");

  let configs_params = crate::service::configuration::GetConfigurationParams {
    name: None,
    pattern: None,
    group_name: None,
    settings_hsm_group_name: None,
    since: None,
    until: None,
    limit: None,
  };
  let images_params = crate::service::image::GetImagesParams {
    id: None,
    pattern: None,
    limit: None,
  };

  // Sequential rather than concurrent: the four upstream fetches
  // include `get_cfs_components` and `get_all_bootparameters`, both
  // of which can be very large at scale. Fanning them out with
  // `try_join!` flooded the upstream Envoy and tripped its
  // connection-termination behaviour ("upstream connect error or
  // disconnect/reset before headers"). Sequencing trades wall-clock
  // for resilience; if even the sequential calls trip the upstream,
  // the next step is retry-with-backoff at this layer or paging the
  // bulk calls in csm-rs.
  let configs = crate::service::configuration::get_configurations(
    infra,
    token,
    &configs_params,
  )
  .await?;
  let components = infra
    .backend
    .get_cfs_components(token, None, None, None)
    .await?;
  let boot_params = infra.backend.get_all_bootparameters(token).await?;
  let images =
    crate::service::image::get_images(infra, token, &images_params).await?;

  Ok(build_configuration_analysis(
    configs,
    components,
    boot_params,
    images,
  ))
}

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::bos::session_template::{
    BootSet, Cfs as BosCfs,
  };
  use manta_backend_dispatcher::types::cfs::session::{
    Artifact, Configuration as SessionConfiguration, Status as CfsStatus,
  };
  use std::collections::HashMap;

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

  fn session_producing(
    name: &str,
    cfg_name: Option<&str>,
    result_ids: &[&str],
  ) -> CfsSessionGetResponse {
    CfsSessionGetResponse {
      name: name.to_string(),
      configuration: cfg_name.map(|c| SessionConfiguration {
        name: Some(c.to_string()),
        limit: None,
      }),
      ansible: None,
      target: None,
      status: Some(CfsStatus {
        session: None,
        artifacts: Some(
          result_ids
            .iter()
            .map(|id| Artifact {
              image_id: None,
              result_id: Some(id.to_string()),
              r#type: None,
            })
            .collect(),
        ),
      }),
      tags: None,
      debug_on_failure: false,
      logs: None,
    }
  }

  fn empty_bootset() -> BootSet {
    BootSet {
      name: None,
      path: None,
      cfs: None,
      r#type: None,
      etag: None,
      kernel_parameters: None,
      node_list: None,
      node_roles_groups: None,
      node_groups: None,
      arch: None,
      rootfs_provider: None,
      rootfs_provider_passthrough: None,
    }
  }

  fn template_booting(
    name: &str,
    boot_image_ids: &[&str],
  ) -> BosSessionTemplate {
    let mut boot_sets = HashMap::new();
    for (i, id) in boot_image_ids.iter().enumerate() {
      boot_sets.insert(
        format!("set-{i}"),
        BootSet {
          path: Some(format!("s3://boot-images/{id}/manifest.json")),
          ..empty_bootset()
        },
      );
    }
    BosSessionTemplate {
      name: Some(name.to_string()),
      tenant: None,
      description: None,
      enable_cfs: None,
      cfs: Some(BosCfs {
        configuration: Some("dontcare".to_string()),
      }),
      boot_sets: Some(boot_sets),
      links: None,
    }
  }

  // image_id + name + configuration_name come from Image directly.
  #[test]
  fn anchors_one_row_per_image_with_built_with_configuration() {
    let rows = build_cache(
      vec![],
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

  // session_name + session_result_id + session_configuration_name
  // are set together from the first producing session.
  #[test]
  fn fills_session_columns_when_a_session_produced_the_image() {
    let rows = build_cache(
      vec![session_producing("session-a", Some("ncn-1.6"), &["img-1"])],
      vec![],
      vec![image("img-1", "ncn-1.6-base", None, None)],
    );
    let row = &rows[0];
    assert_eq!(row.session_name.as_deref(), Some("session-a"));
    assert_eq!(row.session_result_id.as_deref(), Some("img-1"));
    assert_eq!(row.session_configuration_name.as_deref(), Some("ncn-1.6"));
  }

  // When multiple sessions produced the same image, first in name
  // order wins.
  #[test]
  fn picks_first_producing_session_in_name_order() {
    let rows = build_cache(
      vec![
        session_producing("session-z", Some("ncn-1.6"), &["img-1"]),
        session_producing("session-a", Some("ncn-1.6"), &["img-1"]),
      ],
      vec![],
      vec![image("img-1", "ncn-1.6-base", None, None)],
    );
    assert_eq!(rows[0].session_name.as_deref(), Some("session-a"));
  }

  // bos_sessiontemplate + bos_sessiontemplate_boot_image set together.
  #[test]
  fn fills_bos_columns_when_a_template_boots_from_the_image() {
    let rows = build_cache(
      vec![],
      vec![template_booting("tmpl-x", &["img-1"])],
      vec![image("img-1", "ncn-1.6-base", None, None)],
    );
    let row = &rows[0];
    assert_eq!(row.bos_sessiontemplate.as_deref(), Some("tmpl-x"));
    assert_eq!(row.bos_sessiontemplate_boot_image.as_deref(), Some("img-1"));
  }

  // First template in name order wins.
  #[test]
  fn picks_first_booting_template_in_name_order() {
    let rows = build_cache(
      vec![],
      vec![
        template_booting("tmpl-z", &["img-1"]),
        template_booting("tmpl-a", &["img-1"]),
      ],
      vec![image("img-1", "ncn-1.6-base", None, None)],
    );
    assert_eq!(rows[0].bos_sessiontemplate.as_deref(), Some("tmpl-a"));
  }

  // Orphan images still get a row.
  #[test]
  fn orphan_images_get_a_row_with_optional_columns_none() {
    let rows =
      build_cache(vec![], vec![], vec![image("img-1", "orphan", None, None)]);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.image_id, "img-1");
    assert!(row.image_created.is_none());
    assert!(row.configuration_name.is_none());
    assert!(row.session_name.is_none());
    assert!(row.session_result_id.is_none());
    assert!(row.session_configuration_name.is_none());
    assert!(row.bos_sessiontemplate.is_none());
    assert!(row.bos_sessiontemplate_boot_image.is_none());
  }

  // Sessions / templates that name nonexistent images: dropped.
  #[test]
  fn sessions_and_templates_without_an_image_are_dropped() {
    let rows = build_cache(
      vec![session_producing("session-x", None, &["nonexistent"])],
      vec![template_booting("tmpl-x", &["nonexistent"])],
      vec![],
    );
    assert!(rows.is_empty());
  }

  // When no image has a created timestamp, sort falls back to image_id
  // ascending so output stays deterministic across runs.
  #[test]
  fn rows_with_no_created_timestamp_fall_back_to_image_id_asc() {
    let rows = build_cache(
      vec![],
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
    let rows = build_cache(vec![], vec![], vec![]);
    assert!(rows.is_empty());
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
    assert_eq!(rows[0].name, "orphan");
    assert_eq!(rows[0].last_updated, "2025-01-01T00:00:00Z");
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
    let desired = rows.iter().find(|r| r.name == "desired").unwrap();
    let other = rows.iter().find(|r| r.name == "nobody-cares").unwrap();
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
    let boot = rows.iter().find(|r| r.name == "boot-config").unwrap();
    let other = rows.iter().find(|r| r.name == "nobody-cares").unwrap();
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
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
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
