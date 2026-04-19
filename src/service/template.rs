use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::bos::{
  ClusterSessionTrait, ClusterTemplateTrait,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session::Operation;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

use crate::common::app_context::InfraContext;
use crate::common::authorization::{
  get_groups_names_available, validate_target_hsm_members,
};
use crate::common::node_ops::validate_xname_format;

/// Typed parameters for fetching BOS session templates.
pub struct GetTemplateParams {
  pub name: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub limit: Option<u8>,
}

/// Fetch and filter BOS session templates from the backend.
pub async fn get_templates(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetTemplateParams,
) -> Result<Vec<BosSessionTemplate>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let hsm_member_vec = infra.backend
    .get_member_vec_from_group_name_vec(token, &target_hsm_group_vec)
    .await?;

  let limit_ref = params.limit.as_ref();

  log::info!(
    "Get BOS sessiontemplates for HSM groups: {:?}",
    target_hsm_group_vec
  );

  let mut bos_sessiontemplate_vec = infra.backend
    .get_and_filter_templates(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      &target_hsm_group_vec,
      &hsm_member_vec,
      params.name.as_deref(),
      limit_ref,
    )
    .await
    .context("Could not get BOS sessiontemplate list")?;

  bos_sessiontemplate_vec.sort_by(|a, b| a.name.cmp(&b.name));

  Ok(bos_sessiontemplate_vec)
}

/// Parameters for applying a BOS session template.
pub struct ApplyTemplateParams {
  pub bos_session_name: Option<String>,
  pub bos_sessiontemplate_name: String,
  pub bos_session_operation: String,
  pub limit: String,
  pub include_disabled: bool,
}

/// Validate template access, resolve limit targets, and build
/// a BOS session ready for creation.
///
/// Returns `(bos_session, resolved_limit_vec)`.
pub async fn validate_and_prepare_template_session(
  infra: &InfraContext<'_>,
  token: &str,
  params: &ApplyTemplateParams,
) -> Result<(BosSession, Vec<String>), Error> {
  let backend = infra.backend;

  // Fetch BOS sessiontemplate
  let bos_sessiontemplate_vec = backend
    .get_and_filter_templates(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      &[],
      &[],
      Some(&params.bos_sessiontemplate_name),
      None,
    )
    .await
    .with_context(|| {
      format!(
        "Could not fetch BOS sessiontemplate '{}'",
        params.bos_sessiontemplate_name
      )
    })?;

  let bos_sessiontemplate = if bos_sessiontemplate_vec.is_empty() {
    bail!(
      "No BOS sessiontemplate '{}' found",
      params.bos_sessiontemplate_name
    );
  } else {
    bos_sessiontemplate_vec
      .first()
      .context("BOS sessiontemplate list unexpectedly empty")?
  };

  // Validate user has access to the BOS sessiontemplate targets
  log::info!("Validate user has access to HSM group in BOS sessiontemplate");
  let target_hsm_vec = bos_sessiontemplate.get_target_hsm();
  let target_xname_vec: Vec<String> = if !target_hsm_vec.is_empty() {
    backend
      .get_member_vec_from_group_name_vec(token, &target_hsm_vec)
      .await
      .unwrap_or_default()
  } else {
    bos_sessiontemplate.get_target_xname()
  };

  validate_target_hsm_members(backend, token, &target_xname_vec).await?;

  // Validate user has access to xnames in `limit` argument
  log::info!("Validate user has access to xnames in BOS sessiontemplate");
  let limit_vec: Vec<String> =
    params.limit.split(',').map(str::to_string).collect();

  let mut xnames_to_validate_access_vec = Vec::new();

  for limit_value in &limit_vec {
    log::info!("Check if limit value '{}', is an xname", limit_value);
    if validate_xname_format(limit_value) {
      log::info!("limit value '{}' is an xname", limit_value);
      xnames_to_validate_access_vec.push(limit_value.to_string());
    } else {
      let hsm_members_vec_rslt = backend
        .get_member_vec_from_group_name_vec(
          token,
          std::slice::from_ref(limit_value),
        )
        .await;

      if let Ok(mut hsm_members_vec) = hsm_members_vec_rslt {
        log::info!(
          "Check if limit value '{}', is an HSM group name",
          limit_value
        );
        xnames_to_validate_access_vec.append(&mut hsm_members_vec);
      } else {
        bail!(
          "Value '{}' in 'limit' argument does not match \
           an xname or a HSM group name.",
          limit_value
        );
      }
    }
  }

  log::info!("Validate list of xnames translated from 'limit argument'");
  validate_target_hsm_members(
    backend,
    token,
    &xnames_to_validate_access_vec,
  )
  .await?;

  log::info!("Access to '{}' granted. Continue.", params.limit);

  // Build BOS session
  let bos_session = BosSession {
    name: params.bos_session_name.clone(),
    tenant: None,
    operation: Some(
      Operation::from_str(&params.bos_session_operation).map_err(|_| {
        Error::msg(format!(
          "Invalid BOS session operation '{}'",
          params.bos_session_operation
        ))
      })?,
    ),
    template_name: params.bos_sessiontemplate_name.clone(),
    limit: Some(limit_vec.join(",")),
    stage: Some(false),
    components: None,
    include_disabled: Some(params.include_disabled),
    status: None,
  };

  Ok((bos_session, limit_vec))
}

/// Create a BOS session via the backend.
pub async fn create_bos_session(
  infra: &InfraContext<'_>,
  token: &str,
  bos_session: BosSession,
) -> Result<BosSession, Error> {
  infra
    .backend
    .post_template_session(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      bos_session,
    )
    .await
    .context("Could not create BOS session")
}
