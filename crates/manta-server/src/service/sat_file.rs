//! SAT file apply orchestration (backend trait + HSM groups).
//!
//! Rendering (Jinja2), parsing, and `image_only` / `session_template_only`
//! filtering are performed client-side by the CLI; this layer receives
//! the already-parsed SAT file as a `serde_json::Value`, looks up the
//! caller's available HSM groups, and forwards everything to the
//! backend's `SatTrait`. The backend fetches its own Kubernetes secrets
//! from Vault internally.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    apply_sat_file::{
      ApplySatFileParams as BackendApplySatFileParams, SatTrait,
    },
    hsm::group::GroupTrait,
  },
  types::{
    bos::{session::BosSession, session_template::BosSessionTemplate},
    cfs::cfs_configuration_response::CfsConfigurationResponse,
    ims::Image,
  },
};

use crate::server::common::app_context::InfraContext;
pub use manta_shared::shared::params::sat_file::ApplySatFileParams;

/// Apply a pre-rendered SAT file via the backend.
///
/// Returns the four lists of artifacts the backend produced (or would
/// produce, in `dry_run` mode): CFS configurations, IMS images, BOS
/// session templates, and BOS sessions. The handler serialises these as
/// the JSON response body so `manta apply sat-file` can show them.
pub async fn apply_sat_file(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  params: ApplySatFileParams<'_>,
) -> Result<
  (
    Vec<CfsConfigurationResponse>,
    Vec<Image>,
    Vec<BosSessionTemplate>,
    Vec<BosSession>,
  ),
  Error,
> {
  let hsm_group_available_vec =
    infra.backend.get_group_name_available(token).await?;

  infra
    .backend
    .apply_sat_file(BackendApplySatFileParams {
      shasta_token: token,
      shasta_base_url: infra.shasta_base_url,
      shasta_root_cert: infra.shasta_root_cert,
      socks5_proxy: infra.socks5_proxy,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      sat_file: params.sat_file,
      hsm_group_available_vec: &hsm_group_available_vec,
      ansible_verbosity: params.ansible_verbosity,
      ansible_passthrough: params.ansible_passthrough,
      gitea_base_url: infra.gitea_base_url,
      gitea_token,
      reboot: params.reboot,
      watch_logs: params.watch_logs,
      timestamps: params.timestamps,
      debug_on_failure: true,
      overwrite: params.overwrite,
      dry_run: params.dry_run,
    })
    .await
}
