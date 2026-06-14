//! Image-centric flat projection of CFS configurations + sessions +
//! BOS templates + IMS images. One row per IMS image, eight columns
//! per row (see [`BackendSummary`] for field semantics).
//!
//! - [`get_summary`] orchestrates four service-layer fetches
//!   concurrently and feeds them to [`build_summary`]. It is the
//!   only IO surface in this module; the linker itself is pure.
//! - [`build_summary`] takes the four resource vecs and produces a
//!   sorted `Vec<BackendSummary>`. Pure function — exercised by
//!   unit tests, called by [`get_summary`].

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
pub use manta_shared::types::api::summary::BackendSummary;

/// Pure linker.
///
/// `configs` is intentionally unused: every column on
/// `BackendSummary` comes from `Image`, `CfsSessionGetResponse`,
/// or `BosSessionTemplate`. Configurations are pulled by
/// `get_summary` because they're cheap to fetch alongside the
/// others and a future column may want them.
pub fn build_summary(
  _configs: Vec<CfsConfigurationResponse>,
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
        configuration_name: img.configuration,
        session_name,
        session_result_id,
        session_configuration_name,
        bos_sessiontemplate,
        bos_sessiontemplate_boot_image,
      })
    })
    .collect();

  rows.sort_by(|a, b| a.image_id.cmp(&b.image_id));
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

/// Orchestrator. Implementation in Task 3.
pub async fn get_summary(
  _infra: &InfraContext<'_>,
  _token: &str,
) -> Result<Vec<BackendSummary>, Error> {
  todo!("Task 3")
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

  fn image(id: &str, name: &str, config: Option<&str>) -> Image {
    Image {
      id: Some(id.to_string()),
      name: name.to_string(),
      created: None,
      link: None,
      arch: None,
      metadata: None,
      groups: None,
      base: None,
      configuration: config.map(String::from),
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
    let rows = build_summary(
      vec![],
      vec![],
      vec![],
      vec![
        image("img-1", "ncn-1.6-base", Some("ncn-1.6")),
        image("img-2", "compute-1.5", Some("compute-1.5")),
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
    let rows = build_summary(
      vec![],
      vec![session_producing("session-a", Some("ncn-1.6"), &["img-1"])],
      vec![],
      vec![image("img-1", "ncn-1.6-base", None)],
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
    let rows = build_summary(
      vec![],
      vec![
        session_producing("session-z", Some("ncn-1.6"), &["img-1"]),
        session_producing("session-a", Some("ncn-1.6"), &["img-1"]),
      ],
      vec![],
      vec![image("img-1", "ncn-1.6-base", None)],
    );
    assert_eq!(rows[0].session_name.as_deref(), Some("session-a"));
  }

  // bos_sessiontemplate + bos_sessiontemplate_boot_image set together.
  #[test]
  fn fills_bos_columns_when_a_template_boots_from_the_image() {
    let rows = build_summary(
      vec![],
      vec![],
      vec![template_booting("tmpl-x", &["img-1"])],
      vec![image("img-1", "ncn-1.6-base", None)],
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
    let rows = build_summary(
      vec![],
      vec![],
      vec![
        template_booting("tmpl-z", &["img-1"]),
        template_booting("tmpl-a", &["img-1"]),
      ],
      vec![image("img-1", "ncn-1.6-base", None)],
    );
    assert_eq!(rows[0].bos_sessiontemplate.as_deref(), Some("tmpl-a"));
  }

  // Orphan images still get a row.
  #[test]
  fn orphan_images_get_a_row_with_optional_columns_none() {
    let rows = build_summary(
      vec![],
      vec![],
      vec![],
      vec![image("img-1", "orphan", None)],
    );
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.image_id, "img-1");
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
    let rows = build_summary(
      vec![],
      vec![session_producing("session-x", None, &["nonexistent"])],
      vec![template_booting("tmpl-x", &["nonexistent"])],
      vec![],
    );
    assert!(rows.is_empty());
  }

  // Output sorted by image_id ascending.
  #[test]
  fn rows_are_sorted_by_image_id() {
    let rows = build_summary(
      vec![],
      vec![],
      vec![],
      vec![
        image("img-z", "z", None),
        image("img-a", "a", None),
        image("img-m", "m", None),
      ],
    );
    let ids: Vec<&str> =
      rows.iter().map(|r| r.image_id.as_str()).collect();
    assert_eq!(ids, vec!["img-a", "img-m", "img-z"]);
  }

  #[test]
  fn empty_input_yields_empty_summary() {
    let rows = build_summary(vec![], vec![], vec![], vec![]);
    assert!(rows.is_empty());
  }
}
