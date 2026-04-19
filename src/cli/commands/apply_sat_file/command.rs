use anyhow::{Context, Error, bail};

use crate::common::{self, app_context::AppContext};

use crossterm::style::Stylize;
use manta_backend_dispatcher::{
  interfaces::apply_sat_file::SatTrait,
  types::{K8sAuth, K8sDetails},
};
use serde_yaml::Value;

use crate::{
  cli::commands::apply_sat_file::utils,
  common::vault::http_client::fetch_shasta_k8s_secrets_from_vault,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

/// Options for applying a SAT file.
///
/// Bundles the many parameters needed by [`exec`] into a
/// single struct, improving call-site readability.
pub struct SatApplyOptions<'a> {
  pub vault_base_url: &'a str,
  pub k8s_api_url: &'a str,
  pub sat_file_content: &'a str,
  pub values_file_content_opt: Option<&'a str>,
  pub values_cli_opt: Option<&'a [String]>,
  pub ansible_verbosity_opt: Option<u8>,
  pub ansible_passthrough_opt: Option<&'a str>,
  pub reboot: bool,
  pub watch_logs: bool,
  pub timestamps: bool,
  pub prehook_opt: Option<&'a str>,
  pub posthook_opt: Option<&'a str>,
  pub image_only: bool,
  pub session_template_only: bool,
  pub debug_on_failure: bool,
  pub overwrite: bool,
  pub dry_run: bool,
  pub assume_yes: bool,
  pub k8s: &'a K8sDetails,
}

/// Validate that a hook script exists and is executable.
fn validate_hook(
  hook_opt: Option<&str>,
  label: &str,
) -> Result<(), Error> {
  if let Some(hook) = hook_opt {
    crate::common::hooks::check_hook_perms(hook_opt)
      .map_err(|e| anyhow::anyhow!("{}. File: {}", e, hook))?;
    println!(
      "{}-hook script '{}' exists and is executable.",
      label, hook
    );
  }
  Ok(())
}

/// Run a hook script if one was provided.
fn run_hook_if_present(
  hook_opt: Option<&str>,
  label: &str,
) -> Result<(), Error> {
  if let Some(hook) = hook_opt {
    println!("Running the {}-hook '{}'", label, hook);
    let code = crate::common::hooks::run_hook(hook_opt)?;
    log::debug!("{}-hook script completed ok. RT={}", label, code);
  }
  Ok(())
}

/// Process and apply a SAT file to the system.
pub async fn exec(
  ctx: &AppContext<'_>,
  opts: &SatApplyOptions<'_>,
) -> Result<(), Error> {
  let backend = ctx.infra.backend;
  let site_name = ctx.infra.site_name;
  let shasta_base_url = ctx.infra.shasta_base_url;
  let shasta_root_cert = ctx.infra.shasta_root_cert;
  let gitea_base_url = ctx.infra.gitea_base_url;

  let shasta_token =
    crate::common::authentication::get_api_token(backend, site_name).await?;

  let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
    &shasta_token,
    opts.vault_base_url,
    site_name,
  )
  .await?;

  let hsm_group_available_vec =
    backend.get_group_name_available(&shasta_token).await?;

  // Validate hooks
  validate_hook(opts.prehook_opt, "Pre")?;
  validate_hook(opts.posthook_opt, "Post")?;

  log::info!("Render SAT template file");
  let sat_template_file_yaml: Value = utils::render_jinja2_sat_file_yaml(
    opts.sat_file_content,
    opts.values_file_content_opt,
    opts.values_cli_opt,
  )?;

  let sat_template_file_string = serde_yaml::to_string(&sat_template_file_yaml)
    .context(
      "Failed to serialize SAT template file \
         to YAML string",
    )?;

  let mut sat_template: utils::SatFile =
    serde_yaml::from_str(&sat_template_file_string).context(
      "Could not parse SAT template yaml \
         file",
    )?;

  // Filter either images or session_templates
  // section according to user request
  sat_template.filter(opts.image_only, opts.session_template_only)?;

  let sat_template_file_yaml: Value = serde_yaml::to_value(sat_template)
    .context(
      "Failed to convert SAT template to \
       YAML value",
    )?;

  println!(
    "{}\n{}",
    "#### SAT file content ####".blue(),
    serde_yaml::to_string(&sat_template_file_yaml).context(
      "Failed to serialize SAT template to \
         YAML for display",
    )?,
  );

  if !common::user_interaction::confirm(
    "Please check the template above and \
     confirm to proceed.",
    opts.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  // Confirm reboot if session_templates are to be
  // applied
  if sat_template_file_yaml.get("session_templates").is_some()
    && opts.reboot
    && !common::user_interaction::confirm(
      "This operation will reboot nodes. \
       Please confirm to proceed.",
      opts.assume_yes,
    )
  {
    println!("Operation cancelled by user");
    return Ok(());
  }

  // Run/process Pre-hook
  run_hook_if_present(opts.prehook_opt, "pre")?;

  // Get K8s secrets
  let shasta_k8s_secrets = match &opts.k8s.authentication {
    K8sAuth::Native {
      certificate_authority_data,
      client_certificate_data,
      client_key_data,
    } => {
      serde_json::json!({
        "certificate-authority-data":
          certificate_authority_data,
        "client-certificate-data":
          client_certificate_data,
        "client-key-data":
          client_key_data
      })
    }
    K8sAuth::Vault { base_url } => {
      fetch_shasta_k8s_secrets_from_vault(base_url, site_name, &shasta_token)
        .await
        .context("Failed to fetch K8s secrets from Vault")?
    }
  };

  backend
    .apply_sat_file(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      opts.vault_base_url,
      site_name,
      opts.k8s_api_url,
      shasta_k8s_secrets,
      sat_template_file_yaml,
      &hsm_group_available_vec,
      opts.ansible_verbosity_opt,
      opts.ansible_passthrough_opt,
      gitea_base_url,
      &gitea_token,
      opts.reboot,
      opts.watch_logs,
      opts.timestamps,
      opts.debug_on_failure,
      opts.overwrite,
      opts.dry_run,
    )
    .await?;

  // Run/process Post-hook
  run_hook_if_present(opts.posthook_opt, "post")?;

  Ok(())
}
