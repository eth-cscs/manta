//! Image-centric flat projection of CFS configurations + sessions +
//! BOS templates + IMS images. One row per IMS image, eight columns
//! per row (see [`BackendSummary`] for field semantics).
//!
//! - [`get_cache`] orchestrates four service-layer fetches
//!   concurrently and feeds them to [`build_cache`]. It is the
//!   only IO surface in this module; the linker itself is pure.
//! - [`build_cache`] takes the four resource vecs and produces a
//!   sorted `Vec<BackendSummary>`. Pure function — exercised by
//!   unit tests, called by [`get_cache`].

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
pub use manta_shared::types::api::summary::BackendSummary;

/// Pure linker.
pub fn build_cache(
  configs: Vec<CfsConfigurationResponse>,
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

  // Lookup: configuration name -> last_updated timestamp.
  let configuration_last_updated: HashMap<String, String> = configs
    .into_iter()
    .map(|c| (c.name, c.last_updated))
    .collect();

  // Reverse index: image_id -> (first session that produced it, that
  // session's configuration name, that session's start_time).
  let mut session_by_image: HashMap<
    String,
    (String, Option<String>, Option<String>),
  > = HashMap::new();
  for s in &sessions {
    let cfg = s.configuration.as_ref().and_then(|c| c.name.clone());
    let start_time = s.get_start_time();
    for result_id in s.get_result_id_vec() {
      session_by_image
        .entry(result_id)
        .or_insert_with(|| (s.name.clone(), cfg.clone(), start_time.clone()));
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
      let (session_name, session_configuration_name, session_start_time) =
        match session_by_image.get(&id).cloned() {
          Some((sn, scn, sst)) => (Some(sn), scn, sst),
          None => (None, None, None),
        };
      let session_result_id = session_name.as_ref().map(|_| id.clone());
      let bos_sessiontemplate = template_by_image.get(&id).cloned();
      let bos_sessiontemplate_boot_image =
        bos_sessiontemplate.as_ref().map(|_| id.clone());
      let configuration_last_updated = img
        .configuration
        .as_ref()
        .and_then(|name| configuration_last_updated.get(name).cloned());
      Some(BackendSummary {
        image_id: id,
        name: img.name,
        image_created: img.created,
        configuration_name: img.configuration,
        configuration_last_updated,
        session_name,
        session_result_id,
        session_configuration_name,
        session_start_time,
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
      (Some(ac), Some(bc)) => ac.cmp(bc).then_with(|| a.image_id.cmp(&b.image_id)),
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

/// Fan the four service-layer fetchers out concurrently, then run
/// the pure linker. Each fetcher applies its own group-access scope,
/// so the cache cannot return rows the caller couldn't list via
/// the per-resource endpoints.
pub async fn get_cache(
  infra: &InfraContext<'_>,
  token: &str,
) -> Result<Vec<BackendSummary>, Error> {
  tracing::info!("Building backend cache");

  let configs_params = crate::service::configuration::GetConfigurationParams {
    name: None,
    pattern: None,
    group_name: None,
    settings_hsm_group_name: None,
    since: None,
    until: None,
    limit: None,
  };
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

  let (configs, sessions, templates, images) = tokio::try_join!(
    crate::service::configuration::get_configurations(
      infra,
      token,
      &configs_params
    ),
    crate::service::session::get_sessions(infra, token, &sessions_params),
    crate::service::template::get_templates(infra, token, &templates_params),
    crate::service::image::get_images(infra, token, &images_params),
  )?;

  Ok(build_cache(configs, sessions, templates, images))
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

  fn session_producing(
    name: &str,
    cfg_name: Option<&str>,
    result_ids: &[&str],
    start_time: Option<&str>,
  ) -> CfsSessionGetResponse {
    use manta_backend_dispatcher::types::cfs::session::Session as CfsSession;
    CfsSessionGetResponse {
      name: name.to_string(),
      configuration: cfg_name.map(|c| SessionConfiguration {
        name: Some(c.to_string()),
        limit: None,
      }),
      ansible: None,
      target: None,
      status: Some(CfsStatus {
        session: start_time.map(|t| CfsSession {
          job: None,
          ims_job: None,
          completion_time: None,
          start_time: Some(t.to_string()),
          status: None,
          succeeded: None,
        }),
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

  fn template_booting(name: &str, boot_image_ids: &[&str]) -> BosSessionTemplate {
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
      vec![],
      vec![session_producing(
        "session-a",
        Some("ncn-1.6"),
        &["img-1"],
        None,
      )],
      vec![],
      vec![image("img-1", "ncn-1.6-base", None, None)],
    );
    let row = &rows[0];
    assert_eq!(row.session_name.as_deref(), Some("session-a"));
    assert_eq!(row.session_result_id.as_deref(), Some("img-1"));
    assert_eq!(
      row.session_configuration_name.as_deref(),
      Some("ncn-1.6")
    );
  }

  // When multiple sessions produced the same image, first in name
  // order wins.
  #[test]
  fn picks_first_producing_session_in_name_order() {
    let rows = build_cache(
      vec![],
      vec![
        session_producing("session-z", Some("ncn-1.6"), &["img-1"], None),
        session_producing("session-a", Some("ncn-1.6"), &["img-1"], None),
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
      vec![],
      vec![template_booting("tmpl-x", &["img-1"])],
      vec![image("img-1", "ncn-1.6-base", None, None)],
    );
    let row = &rows[0];
    assert_eq!(row.bos_sessiontemplate.as_deref(), Some("tmpl-x"));
    assert_eq!(
      row.bos_sessiontemplate_boot_image.as_deref(),
      Some("img-1")
    );
  }

  // First template in name order wins.
  #[test]
  fn picks_first_booting_template_in_name_order() {
    let rows = build_cache(
      vec![],
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
    let rows = build_cache(
      vec![],
      vec![],
      vec![],
      vec![image("img-1", "orphan", None, None)],
    );
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.image_id, "img-1");
    assert!(row.image_created.is_none());
    assert!(row.configuration_name.is_none());
    assert!(row.configuration_last_updated.is_none());
    assert!(row.session_name.is_none());
    assert!(row.session_result_id.is_none());
    assert!(row.session_configuration_name.is_none());
    assert!(row.session_start_time.is_none());
    assert!(row.bos_sessiontemplate.is_none());
    assert!(row.bos_sessiontemplate_boot_image.is_none());
  }

  // Sessions / templates that name nonexistent images: dropped.
  #[test]
  fn sessions_and_templates_without_an_image_are_dropped() {
    let rows = build_cache(
      vec![],
      vec![session_producing("session-x", None, &["nonexistent"], None)],
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
      vec![],
      vec![
        image("img-z", "z", None, None),
        image("img-a", "a", None, None),
        image("img-m", "m", None, None),
      ],
    );
    let ids: Vec<&str> =
      rows.iter().map(|r| r.image_id.as_str()).collect();
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
      vec![],
      vec![
        image("img-old", "old", None, Some("2024-01-01T00:00:00Z")),
        image("img-newest", "newest", None, Some("2026-06-02T00:00:00Z")),
        image("img-undated-z", "undated-z", None, None),
        image("img-middle", "middle", None, Some("2026-06-01T00:00:00Z")),
        image("img-undated-a", "undated-a", None, None),
      ],
    );
    let ids: Vec<&str> =
      rows.iter().map(|r| r.image_id.as_str()).collect();
    assert_eq!(
      ids,
      vec![
        "img-old",        // 2024-01-01
        "img-middle",     // 2026-06-01
        "img-newest",     // 2026-06-02
        "img-undated-a",  // None, id asc tie-break
        "img-undated-z",  // None, id asc tie-break
      ]
    );
  }

  #[test]
  fn empty_input_yields_empty_cache() {
    let rows = build_cache(vec![], vec![], vec![], vec![]);
    assert!(rows.is_empty());
  }

  // configuration_last_updated is looked up from the configs list by
  // the image's built-with configuration name.
  #[test]
  fn fills_configuration_last_updated_from_configs_lookup() {
    let rows = build_cache(
      vec![
        config("ncn-1.6", "2026-06-01T00:00:00Z"),
        config("other", "1999-01-01T00:00:00Z"),
      ],
      vec![],
      vec![],
      vec![
        image("img-1", "ncn-1.6-base", Some("ncn-1.6"), None),
        image("img-2", "no-config", None, None),
        image("img-3", "missing-config", Some("not-listed"), None),
      ],
    );
    assert_eq!(
      rows[0].configuration_last_updated.as_deref(),
      Some("2026-06-01T00:00:00Z")
    );
    assert!(rows[1].configuration_last_updated.is_none());
    assert!(rows[2].configuration_last_updated.is_none());
  }
}
