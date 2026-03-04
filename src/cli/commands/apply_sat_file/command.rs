use anyhow::{Context, Error, bail};

use crate::common::{self, app_context::AppContext};

use manta_backend_dispatcher::{
  interfaces::apply_sat_file::SatTrait,
  types::{K8sAuth, K8sDetails},
};
use serde_yaml::Value;
use termion::color;

use crate::{
  cli::commands::apply_sat_file::utils,
  common::vault::http_client::fetch_shasta_k8s_secrets_from_vault,
};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  vault_base_url: &str,
  k8s_api_url: &str,
  sat_file_content: &str,
  values_file_content_opt: Option<&str>,
  values_cli_opt: Option<&[String]>,
  ansible_verbosity_opt: Option<u8>,
  ansible_passthrough_opt: Option<&str>,
  reboot: bool,
  watch_logs: bool,
  timestamps: bool,
  prehook_opt: Option<&str>,
  posthook_opt: Option<&str>,
  image_only: bool,
  session_template_only: bool,
  debug_on_failure: bool,
  overwrite: bool,
  dry_run: bool,
  assume_yes: bool,
  k8s: &K8sDetails,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let shasta_base_url = ctx.shasta_base_url;
  let shasta_root_cert = ctx.shasta_root_cert;
  let gitea_base_url = ctx.gitea_base_url;

  let shasta_token =
    crate::common::authentication::get_api_token(backend, site_name).await?;

  let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
    &shasta_token,
    vault_base_url,
    site_name,
  )
  .await?;

  let hsm_group_available_vec =
    backend.get_group_name_available(&shasta_token).await?;

  // Validate Pre-hook
  log::info!("Validating pre-hook script");
  if let Some(prehook) = prehook_opt {
    match crate::common::hooks::check_hook_perms(prehook_opt).await {
      Ok(_r) => println!(
        "Pre-hook script '{}' exists \
         and is executable.",
        prehook
      ),
      Err(e) => {
        bail!("{}. File: {}", e, &prehook);
      }
    };
  }

  // Validate Post-hook
  log::info!("Validating post-hook script");
  if let Some(posthook) = posthook_opt {
    match crate::common::hooks::check_hook_perms(posthook_opt).await {
      Ok(_) => println!(
        "Post-hook script '{}' exists \
         and is executable.",
        posthook
      ),
      Err(e) => {
        bail!("{}. File: {}", e, &posthook);
      }
    };
  }

  log::info!("Render SAT template file");
  let sat_template_file_yaml: Value = utils::render_jinja2_sat_file_yaml(
    sat_file_content,
    values_file_content_opt,
    values_cli_opt,
  )?;

  let sat_template_file_string = serde_yaml::to_string(&sat_template_file_yaml)
    .context(
      "Failed to serialize SAT template file \
         to YAML string",
    )?;

  let mut sat_template: utils::SatFile =
    serde_yaml::from_str(&sat_template_file_string).map_err(|e| {
      Error::msg(format!(
        "Could not parse SAT template yaml \
           file. Error:\n{e}"
      ))
    })?;

  // Filter either images or session_templates
  // section according to user request
  sat_template.filter(image_only, session_template_only)?;

  let sat_template_file_yaml: Value = serde_yaml::to_value(sat_template)
    .context(
      "Failed to convert SAT template to \
       YAML value",
    )?;

  println!(
    "{}#### SAT file content ####{}\n{}",
    color::Fg(color::Blue),
    color::Fg(color::Reset),
    serde_yaml::to_string(&sat_template_file_yaml).context(
      "Failed to serialize SAT template to \
         YAML for display",
    )?,
  );

  if !common::user_interaction::confirm(
    "Please check the template above and \
     confirm to proceed.",
    assume_yes,
  ) {
    bail!("Operation canceled by user");
  }

  // Confirm reboot if session_templates are to be
  // applied
  if sat_template_file_yaml.get("session_templates").is_some()
    && reboot
    && !common::user_interaction::confirm(
      "This operation will reboot nodes. \
       Please confirm to proceed.",
      assume_yes,
    )
  {
    println!("Operation canceled by user");
    return Ok(());
  }

  // Run/process Pre-hook
  if let Some(prehook) = prehook_opt {
    println!("Running the pre-hook '{}'", &prehook);
    let code = crate::common::hooks::run_hook(prehook_opt).await?;

    log::debug!("Pre-hook script completed ok. RT={}", code);
  }

  // Get K8s secrets
  let shasta_k8s_secrets = match &k8s.authentication {
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
      vault_base_url,
      site_name,
      k8s_api_url,
      shasta_k8s_secrets,
      sat_template_file_yaml,
      &hsm_group_available_vec,
      ansible_verbosity_opt,
      ansible_passthrough_opt,
      gitea_base_url,
      &gitea_token,
      reboot,
      watch_logs,
      timestamps,
      debug_on_failure,
      overwrite,
      dry_run,
    )
    .await?;

  // Run/process Post-hook
  if let Some(posthook) = posthook_opt {
    println!("Running the post-hook '{}'", &posthook);
    let code = crate::common::hooks::run_hook(posthook_opt).await?;

    log::debug!("Post-hook script completed ok. RT={}", code);
  }

  Ok(())
}
