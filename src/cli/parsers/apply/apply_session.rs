use std::path::PathBuf;

use anyhow::Error;
use clap::ArgMatches;

use crate::{
  cli::commands::apply_session,
  common::{
    authentication::get_api_token,
    authorization::{get_groups_names_available, validate_target_hsm_members},
    config::types::MantaConfiguration,
    kafka::Kafka,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn process_subcommand(
  cli_apply_session: &ArgMatches,
  backend: StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  vault_base_url: &str,
  gitea_base_url: &str,
  settings_hsm_group_name_opt: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
  configuration: &MantaConfiguration,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  // FIXME: gitea auth token should be calculated before colling this function
  let gitea_token = crate::common::vault::http_client::fetch_shasta_vcs_token(
    &shasta_token,
    vault_base_url,
    &site_name,
  )
  .await
  .unwrap();

  let repo_path_vec: Vec<PathBuf> = cli_apply_session
    .get_many("repo-path")
    .unwrap()
    .cloned()
    .collect();

  let hsm_group_name_arg_opt: Option<&String> =
    cli_apply_session.get_one("hsm-group");

  let cfs_conf_sess_name_opt: Option<&String> =
    cli_apply_session.get_one("name");
  let playbook_file_name_opt: Option<&String> =
    cli_apply_session.get_one("playbook-name");

  let hsm_group_members_opt: Option<&str> = cli_apply_session
    .get_one("ansible-limit")
    .map(String::as_str);
  let ansible_verbosity: Option<&String> =
    cli_apply_session.get_one("ansible-verbosity");

  let ansible_passthrough: Option<&String> =
    cli_apply_session.get_one("ansible-passthrough");

  let watch_logs: bool = cli_apply_session.get_flag("watch-logs");

  let timestamps: bool = cli_apply_session.get_flag("timestamps");

  let target_hsm_group_vec = get_groups_names_available(
    &backend,
    &shasta_token,
    hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  target_hsm_group_vec.first().unwrap();

  if let Some(ansible_limit) = hsm_group_members_opt {
    validate_target_hsm_members(
      &backend,
      &shasta_token,
      &ansible_limit
        .split(',')
        .map(|xname| xname.trim().to_string())
        .collect::<Vec<String>>(),
    )
    .await?;
  }

  let site = configuration
    .sites
    .get(&configuration.site.clone())
    .unwrap();

  let _ = apply_session::exec(
    backend,
    &site_name,
    &gitea_token,
    gitea_base_url,
    &shasta_token,
    shasta_base_url,
    shasta_root_cert,
    cfs_conf_sess_name_opt.map(String::as_str),
    playbook_file_name_opt.map(String::as_str),
    hsm_group_name_arg_opt.map(String::as_str),
    &repo_path_vec,
    hsm_group_members_opt,
    ansible_verbosity.map(String::as_str),
    ansible_passthrough.map(String::as_str),
    watch_logs,
    timestamps,
    kafka_audit_opt,
    &site
      .k8s
      .as_ref()
      .expect("ERROR - k8s section not found in configuration"), // FIXME:
                                                                 // refactor this, we can't check configuration here and should be done ealier
  )
  .await?;

  Ok(())
}
