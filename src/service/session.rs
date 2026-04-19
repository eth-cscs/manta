use anyhow::Error;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Typed parameters for fetching CFS sessions.
pub struct GetSessionParams {
  pub hsm_group: Option<String>,
  pub xnames: Vec<String>,
  pub min_age: Option<String>,
  pub max_age: Option<String>,
  pub session_type: Option<String>,
  pub status: Option<String>,
  pub name: Option<String>,
  pub limit: Option<u8>,
}

/// Fetch and filter CFS sessions from the backend.
///
/// Queries the backend for sessions matching the given
/// parameters and returns the filtered results.
pub async fn get_sessions(
  backend: &StaticBackendDispatcher,
  token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  params: &GetSessionParams,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  log::info!("Get CFS sessions");

  backend
    .get_and_filter_sessions(
      token,
      shasta_base_url,
      shasta_root_cert,
      params
        .hsm_group
        .as_ref()
        .map(|v| vec![v.clone()])
        .unwrap_or_default(),
      params.xnames.iter().map(String::as_str).collect(),
      params.min_age.as_ref(),
      params.max_age.as_ref(),
      params.session_type.as_ref(),
      params.status.as_ref(),
      params.name.as_ref(),
      params.limit.as_ref(),
      None,
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))
}
