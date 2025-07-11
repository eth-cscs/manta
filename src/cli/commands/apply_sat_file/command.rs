use dialoguer::theme::ColorfulTheme;
use manta_backend_dispatcher::{
  interfaces::apply_sat_file::SatTrait,
  types::{K8sAuth, K8sDetails},
};
use serde_yaml::Value;
use termion::color;

use crate::{
  cli::commands::apply_sat_file::utils,
  common::vault::http_client::fetch_shasta_k8s_secrets_from_vault,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  vault_base_url: &str,
  k8s_api_url: &str,
  sat_file_content: String,
  values_file_content_opt: Option<String>,
  values_cli_opt: Option<Vec<String>>,
  hsm_group_available_vec: &Vec<String>,
  ansible_verbosity_opt: Option<u8>,
  ansible_passthrough_opt: Option<&String>,
  gitea_base_url: &str,
  gitea_token: &str,
  do_not_reboot: bool,
  watch_logs: bool,
  prehook: Option<&String>,
  posthook: Option<&String>,
  image_only: bool,
  session_template_only: bool,
  debug_on_failure: bool,
  overwrite: bool,
  dry_run: bool,
  assume_yes: bool,
  k8s: &K8sDetails,
) {
  // Validate Pre-hook
  log::info!("Validating pre-hook script");
  if prehook.is_some() {
    match crate::common::hooks::check_hook_perms(prehook).await {
      Ok(_r) => println!(
        "Pre-hook script '{}' exists and is executable.",
        prehook.unwrap()
      ),
      Err(e) => {
        log::error!("{}. File: {}", e, &prehook.unwrap());
        std::process::exit(2);
      }
    };
  }

  // Validate Post-hook
  log::info!("Validating post-hook script");
  if posthook.is_some() {
    match crate::common::hooks::check_hook_perms(posthook).await {
      Ok(_) => println!(
        "Post-hook script '{}' exists and is executable.",
        posthook.unwrap()
      ),
      Err(e) => {
        log::error!("{}. File: {}", e, &posthook.unwrap());
        std::process::exit(2);
      }
    };
  }

  log::info!("Render SAT template file");
  let sat_template_file_yaml: Value = utils::render_jinja2_sat_file_yaml(
    &sat_file_content,
    values_file_content_opt.as_ref(),
    values_cli_opt,
  )
  .clone();

  let sat_template_file_string =
    serde_yaml::to_string(&sat_template_file_yaml).unwrap();

  let mut sat_template: utils::SatFile =
    serde_yaml::from_str(&sat_template_file_string)
      .expect("Could not parse SAT template yaml file");

  // Filter either images or session_templates section according to user request
  //
  sat_template.filter(image_only, session_template_only);

  let sat_template_file_yaml: Value =
    serde_yaml::to_value(sat_template).unwrap();

  println!(
    "{}#### SAT file content ####{}\n{}",
    color::Fg(color::Blue),
    color::Fg(color::Reset),
    serde_yaml::to_string(&sat_template_file_yaml).unwrap(),
  );

  let process_sat_file = if !assume_yes {
    dialoguer::Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt("Please check the template above and confirm to proceed.")
      .interact()
      .unwrap()
  } else {
    true
  };

  // Run/process Pre-hook
  if prehook.is_some() {
    println!("Running the pre-hook '{}'", &prehook.unwrap());
    match crate::common::hooks::run_hook(prehook).await {
      Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
      Err(_error) => {
        log::error!("{}", _error);
        std::process::exit(2);
      }
    };
  }

  if process_sat_file {
    println!("Proceed and process SAT file");
  } else {
    println!("Operation canceled by user. Exit");
    std::process::exit(0);
  }

  // Get K8s secrets
  let shasta_k8s_secrets = match &k8s.authentication {
    K8sAuth::Native {
      certificate_authority_data,
      client_certificate_data,
      client_key_data,
    } => {
      serde_json::json!({ "certificate-authority-data": certificate_authority_data, "client-certificate-data": client_certificate_data, "client-key-data": client_key_data })
    }
    K8sAuth::Vault { base_url } => {
      fetch_shasta_k8s_secrets_from_vault(&base_url, site_name, shasta_token)
        .await
        .unwrap()
    }
  };

  backend
    .apply_sat_file(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      site_name,
      k8s_api_url,
      shasta_k8s_secrets,
      sat_template_file_yaml,
      hsm_group_available_vec,
      ansible_verbosity_opt,
      ansible_passthrough_opt,
      gitea_base_url,
      gitea_token,
      do_not_reboot,
      watch_logs,
      debug_on_failure,
      overwrite,
      dry_run,
    )
    .await
    .unwrap_or_else(|e| {
      eprintln!("{}", e);
      std::process::exit(1);
    });

  // Run/process Post-hook
  if posthook.is_some() {
    println!("Running the post-hook '{}'", &posthook.unwrap());
    match crate::common::hooks::run_hook(posthook).await {
      Ok(_code) => log::debug!("Post-hook script completed ok. RT={}", _code),
      Err(_error) => {
        log::error!("{}", _error);
        std::process::exit(2);
      }
    };
  }
}
