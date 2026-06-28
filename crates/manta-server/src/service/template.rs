//! BOS session template queries and BOS session creation with
//! access validation.
//!
//! BOS session creation runs a two-step
//! [`validate_and_prepare_template_session`] + [`create_bos_session`]
//! flow so authorization (which fans across template targets and the
//! `limit` argument) is separate from the actual `post_template_session`
//! call. The split also keeps each side individually unit-testable.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bos::{
  ClusterSessionTrait, ClusterTemplateTrait,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::bos::session::BosSession;
use manta_backend_dispatcher::types::bos::session::Operation;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::{
  validate_user_group_members_access, validate_user_group_vec_access,
};
use crate::service::node_ops::validate_xname_format;
pub use manta_shared::types::api::template::{
  ApplyTemplateParams, GetTemplateParams,
};

/// List BOS session templates visible to the caller.
///
/// When `params.group_name` is unset the lookup spans every HSM
/// group the token already grants access to. The backend filters
/// templates whose targets intersect the resolved group set (and
/// their member xnames), so the response stays scoped to what the
/// caller could see by other means. Results are sorted by template
/// name for stable output.
pub async fn get_templates(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetTemplateParams,
) -> Result<Vec<BosSessionTemplate>, Error> {
  // Get list of target groups the user is asking for
  let target_group_vec: Vec<String> = if let Some(group) = &params.group_name {
    vec![group.clone()]
  } else {
    infra
      .backend
      .get_group_available(token)
      .await?
      .iter()
      .map(|group| group.label.clone())
      .collect()
  };

  // Validate groups and get list of groups available
  validate_user_group_vec_access(infra, token, &target_group_vec).await?;

  let hsm_member_vec = infra
    .backend
    .get_member_vec_from_group_name_vec(token, &target_group_vec)
    .await?;

  let limit_ref = params.limit.as_ref();

  tracing::info!(
    "Get BOS sessiontemplates for HSM groups: {:?}",
    target_group_vec
  );

  let mut bos_sessiontemplate_vec = infra
    .backend
    .get_and_filter_templates(
      token,
      &target_group_vec,
      &hsm_member_vec,
      params.name.as_deref(),
      limit_ref,
    )
    .await?;

  bos_sessiontemplate_vec.sort_by(|a, b| a.name.cmp(&b.name));

  Ok(bos_sessiontemplate_vec)
}

/// Build the [`BosSession`] that
/// [`create_bos_session`] will submit, after validating every
/// xname/group the operation will touch.
///
/// Authorization runs in two passes: first against the template's
/// own targets (group members or explicit xnames), then against each
/// comma-separated entry of `params.limit`, which may itself be an
/// xname or a group label. An unrecognised limit value yields
/// `BadRequest`; a missing template yields `NotFound`. The returned
/// `Vec<String>` is the split limit list, useful when the caller
/// wants to display the resolved targets before creation.
pub async fn validate_and_prepare_template_session(
  infra: &InfraContext<'_>,
  token: &str,
  params: &ApplyTemplateParams,
) -> Result<(BosSession, Vec<String>), Error> {
  // Fetch BOS sessiontemplate
  let bos_sessiontemplate_vec = infra
    .backend
    .get_and_filter_templates(
      token,
      &[],
      &[],
      Some(&params.bos_sessiontemplate_name),
      None,
    )
    .await?;

  let bos_sessiontemplate = if bos_sessiontemplate_vec.is_empty() {
    return Err(Error::NotFound(format!(
      "No BOS sessiontemplate '{}' found",
      params.bos_sessiontemplate_name
    )));
  } else {
    bos_sessiontemplate_vec.first().ok_or_else(|| {
      Error::NotFound("BOS sessiontemplate list unexpectedly empty".to_string())
    })?
  };

  // Validate user has access to the BOS sessiontemplate targets
  tracing::info!(
    "Validate user has access to HSM group in BOS sessiontemplate"
  );
  let target_hsm_vec = bos_sessiontemplate.get_target_hsm();
  let target_xname_vec: Vec<String> = if !target_hsm_vec.is_empty() {
    infra
      .backend
      .get_member_vec_from_group_name_vec(token, &target_hsm_vec)
      .await
      .unwrap_or_default()
  } else {
    bos_sessiontemplate.get_target_xname()
  };

  validate_user_group_members_access(infra, token, &target_xname_vec).await?;

  // Validate user has access to xnames in `limit` argument
  tracing::info!("Validate user has access to xnames in BOS sessiontemplate");
  let limit_vec: Vec<String> =
    params.limit.split(',').map(str::to_string).collect();

  let mut xnames_to_validate_access_vec = Vec::new();

  for limit_value in &limit_vec {
    tracing::info!("Check if limit value '{}', is an xname", limit_value);
    if validate_xname_format(limit_value) {
      tracing::info!("limit value '{}' is an xname", limit_value);
      xnames_to_validate_access_vec.push(limit_value.clone());
    } else {
      let hsm_members_vec_rslt = infra
        .backend
        .get_member_vec_from_group_name_vec(
          token,
          std::slice::from_ref(limit_value),
        )
        .await;

      if let Ok(mut hsm_members_vec) = hsm_members_vec_rslt {
        tracing::info!(
          "Check if limit value '{}', is an HSM group name",
          limit_value
        );
        xnames_to_validate_access_vec.append(&mut hsm_members_vec);
      } else {
        return Err(Error::BadRequest(format!(
          "Value '{limit_value}' in 'limit' argument does not match \
           an xname or a HSM group name."
        )));
      }
    }
  }

  tracing::info!("Validate list of xnames translated from 'limit argument'");
  validate_user_group_members_access(
    infra,
    token,
    &xnames_to_validate_access_vec,
  )
  .await?;

  tracing::info!("Access to '{}' granted. Continue.", params.limit);

  // Build BOS session
  let bos_session = BosSession {
    name: params.bos_session_name.clone(),
    tenant: None,
    operation: Some(
      Operation::from_str(&params.bos_session_operation).map_err(|_| {
        Error::BadRequest(format!(
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

/// Submit a [`BosSession`] previously built by
/// [`validate_and_prepare_template_session`].
///
/// This is a thin wrapper kept so the handler stays a one-liner and
/// the validate / create steps remain separate testable units.
pub async fn create_bos_session(
  infra: &InfraContext<'_>,
  token: &str,
  bos_session: BosSession,
) -> Result<BosSession, Error> {
  infra
    .backend
    .post_template_session(token, bos_session)
    .await
}
