use crate::cli::commands;
use crate::common::{config::types::MantaConfiguration, kafka::Kafka};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;
use config::Config;

pub async fn handle_apply(
  cli_apply: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  vault_base_url: &str,
  gitea_base_url: &str,
  settings_hsm_group_name_opt: Option<&String>,
  k8s_api_url: &str,
  kafka_audit_opt: Option<&Kafka>,
  settings: &Config,
  configuration: &MantaConfiguration,
) -> Result<(), Error> {
  if let Some(cli_apply_hw) = cli_apply.subcommand_matches("hardware") {
    if let Some(cli_apply_hw_cluster) =
      cli_apply_hw.subcommand_matches("cluster")
    {
      commands::apply_hw_cluster::exec(
        cli_apply_hw_cluster,
        backend.clone(),
        site_name,
        settings_hsm_group_name_opt,
      )
      .await?
    }
  } else if let Some(cli_apply_session) =
    cli_apply.subcommand_matches("session")
  {
    commands::apply_session::exec(
      cli_apply_session,
      backend.clone(),
      site_name,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      gitea_base_url,
      settings_hsm_group_name_opt,
      kafka_audit_opt,
      configuration,
    )
    .await?
  } else if let Some(cli_apply_sat_file) =
    cli_apply.subcommand_matches("sat-file")
  {
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    let cli_value_vec_opt: Option<Vec<String>> =
      cli_apply_sat_file.get_many("values").map(|value_vec| {
        value_vec
          .map(|value: &String| value.replace("__DATE__", &timestamp))
          .collect()
      });

    let cli_values_file_content_opt: Option<String> = cli_apply_sat_file
      .get_one("values-file")
      .and_then(|values_file_path: &std::path::PathBuf| {
        std::fs::read_to_string(values_file_path).ok().map(
          |cli_value_file: String| {
            cli_value_file.replace("__DATE__", &timestamp)
          },
        )
      });

    let sat_file_content: String = std::fs::read_to_string(
      cli_apply_sat_file
        .get_one::<std::path::PathBuf>("sat-template-file")
        .expect("ERROR: SAT file not found. Exit"),
    )
    .expect("ERROR: reading SAT file template. Exit");

    let ansible_passthrough_env: Option<String> =
      settings.get("ansible-passthrough").ok();
    let ansible_passthrough_cli_arg = cli_apply_sat_file
      .get_one::<String>("ansible-passthrough")
      .cloned();
    let ansible_passthrough =
      ansible_passthrough_env.or(ansible_passthrough_cli_arg);
    let ansible_verbosity: Option<u8> = cli_apply_sat_file
      .get_one::<String>("ansible-verbosity")
      .map(|ansible_verbosity| ansible_verbosity.parse::<u8>().unwrap());

    let overwrite: bool =
      cli_apply_sat_file.get_flag("overwrite-configuration");

    let prehook: Option<&String> = cli_apply_sat_file.get_one("pre-hook");
    let posthook: Option<&String> = cli_apply_sat_file.get_one("post-hook");
    let reboot: bool = cli_apply_sat_file.get_flag("reboot");

    let watch_logs: bool = cli_apply_sat_file.get_flag("watch-logs");
    let timestamps: bool = cli_apply_sat_file.get_flag("timestamps");

    let assume_yes: bool = cli_apply_sat_file.get_flag("assume-yes");

    let dry_run: bool = cli_apply_sat_file.get_flag("dry-run");

    let values_file_content_opt = cli_values_file_content_opt.as_deref();

    let values_cli_opt: Option<&[String]> = cli_value_vec_opt.as_deref();

    let site = configuration
      .sites
      .get(&configuration.site.clone())
      .ok_or_else(|| Error::msg("Site not valid"))?;

    let k8s_details = site
      .k8s
      .as_ref()
      .ok_or_else(|| Error::msg("K8s details not found in configuration"))?;

    commands::apply_sat_file::command::exec(
      &backend,
      &site_name,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      k8s_api_url,
      sat_file_content.as_str(),
      values_file_content_opt,
      values_cli_opt,
      ansible_verbosity,
      ansible_passthrough.as_deref(),
      gitea_base_url,
      reboot,
      watch_logs,
      timestamps,
      prehook.map(String::as_str),
      posthook.map(String::as_str),
      cli_apply_sat_file.get_flag("image-only"),
      cli_apply_sat_file.get_flag("sessiontemplate-only"),
      true,
      overwrite,
      dry_run,
      assume_yes,
      k8s_details,
    )
    .await?;
  } else if let Some(cli_apply_template) =
    cli_apply.subcommand_matches("template")
  {
    let bos_session_name_opt: Option<&String> =
      cli_apply_template.get_one("name");
    let bos_sessiontemplate_name: &String = cli_apply_template
      .get_one("template")
      .expect("ERROR - template name is mandatory");
    let limit: &String = cli_apply_template
      .get_one("limit")
      .expect("ERROR - limit is mandatory");
    let bos_session_operation: &String = cli_apply_template
      .get_one("operation")
      .expect("ERROR - operation is mandatory");

    let include_disabled: bool = *cli_apply_template
      .get_one("include-disabled")
      .expect("ERROR - include disabled must have a value");

    let assume_yes: bool = cli_apply_template.get_flag("assume-yes");
    let dry_run: bool = cli_apply_template.get_flag("dry-run");

    commands::apply_template::exec(
      &backend,
      site_name,
      shasta_base_url,
      shasta_root_cert,
      bos_session_name_opt.map(String::as_str),
      &bos_sessiontemplate_name,
      &bos_session_operation,
      limit,
      include_disabled,
      assume_yes,
      dry_run,
    )
    .await?;
  } else if let Some(cli_apply_ephemeral_environment) =
    cli_apply.subcommand_matches("ephemeral-environment")
  {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
      return Err(Error::msg(
        "This command needs to run in interactive mode. Exit",
      ));
    }

    commands::apply_ephemeral_env::exec(
      &backend,
      site_name,
      shasta_base_url,
      shasta_root_cert,
      cli_apply_ephemeral_environment
        .get_one::<String>("image-id")
        .unwrap(),
    )
    .await?;
  } else if let Some(cli_apply_kernel_parameters) =
    cli_apply.subcommand_matches("kernel-parameters")
  {
    let hsm_group_name_arg_opt =
      cli_apply_kernel_parameters.get_one::<String>("hsm-group");

    // Logic for nodes fetching moved to command
    let nodes_opt: Option<&String> = if hsm_group_name_arg_opt.is_none() {
      cli_apply_kernel_parameters.get_one::<String>("nodes")
    } else {
      None
    };

    let dryrun = cli_apply_kernel_parameters.get_flag("dry-run");

    let kernel_parameters = cli_apply_kernel_parameters
      .get_one::<String>("VALUE")
      .unwrap(); // clap should validate the argument

    let assume_yes: bool = cli_apply_kernel_parameters.get_flag("assume-yes");
    let do_not_reboot: bool =
      cli_apply_kernel_parameters.get_flag("do-not-reboot");

    let result = commands::apply_kernel_parameters::exec(
      backend.clone(),
      site_name,
      kernel_parameters,
      nodes_opt.map(|x| x.as_str()),
      hsm_group_name_arg_opt.map(|x| x.as_str()),
      settings_hsm_group_name_opt,
      assume_yes,
      do_not_reboot,
      kafka_audit_opt,
      dryrun,
    )
    .await;

    match result {
      Ok(_) => {}
      Err(error) => eprintln!("{}", error),
    }
  } else if let Some(cli_apply_boot) = cli_apply.subcommand_matches("boot") {
    if let Some(cli_apply_boot_nodes) =
      cli_apply_boot.subcommand_matches("nodes")
    {
      let hosts_string: &str = cli_apply_boot_nodes
        .get_one::<String>("VALUE")
        .expect("The 'xnames' argument must have values");

      let new_boot_image_id_opt: Option<&String> =
        cli_apply_boot_nodes.get_one("boot-image");

      if let Some(new_boot_image_id) = new_boot_image_id_opt {
        if uuid::Uuid::parse_str(new_boot_image_id).is_err() {
          return Err(Error::msg("ERROR - image id is not an UUID"));
        }
      }

      let new_boot_image_configuration_opt: Option<&String> =
        cli_apply_boot_nodes.get_one("boot-image-configuration");

      let new_runtime_configuration_opt: Option<&String> =
        cli_apply_boot_nodes.get_one("runtime-configuration");

      let new_kernel_parameters_opt: Option<&String> =
        cli_apply_boot_nodes.get_one::<String>("kernel-parameters");

      let assume_yes = cli_apply_boot_nodes.get_flag("assume-yes");

      let do_not_reboot = cli_apply_boot_nodes.get_flag("do-not-reboot");

      let dry_run = cli_apply_boot_nodes.get_flag("dry-run");

      let result = commands::apply_boot_node::exec(
        &backend,
        site_name,
        shasta_base_url,
        shasta_root_cert,
        new_boot_image_id_opt.map(String::as_str),
        new_boot_image_configuration_opt.map(String::as_str),
        new_runtime_configuration_opt.map(String::as_str),
        new_kernel_parameters_opt.map(String::as_str),
        hosts_string,
        assume_yes,
        do_not_reboot,
        dry_run,
        kafka_audit_opt,
      )
      .await;

      match result {
        Ok(_) => {}
        Err(error) => eprintln!("{}", error),
      }
    } else if let Some(cli_apply_boot_cluster) =
      cli_apply_boot.subcommand_matches("cluster")
    {
      let hsm_group_name_arg: &String = cli_apply_boot_cluster
        .get_one("CLUSTER_NAME")
        .expect("ERROR - cluster name must be provided");

      let new_boot_image_id_opt: Option<&String> =
        cli_apply_boot_cluster.get_one("boot-image");

      let new_boot_image_configuration_opt: Option<&String> =
        cli_apply_boot_cluster.get_one("boot-image-configuration");

      let new_runtime_configuration_opt: Option<&String> =
        cli_apply_boot_cluster.get_one("runtime-configuration");

      let new_kernel_parameters_opt: Option<&String> =
        cli_apply_boot_cluster.get_one("kernel-parameters");

      let assume_yes = cli_apply_boot_cluster.get_flag("assume-yes");

      let do_not_reboot = cli_apply_boot_cluster.get_flag("do-not-reboot");

      let dry_run = cli_apply_boot_cluster.get_flag("dry-run");

      commands::apply_boot_cluster::exec(
        &backend,
        site_name,
        shasta_base_url,
        shasta_root_cert,
        new_boot_image_id_opt.map(String::as_str),
        new_boot_image_configuration_opt.map(String::as_str),
        new_runtime_configuration_opt.map(String::as_str),
        new_kernel_parameters_opt.map(String::as_str),
        hsm_group_name_arg,
        assume_yes,
        do_not_reboot,
        dry_run,
        kafka_audit_opt,
      )
      .await;
    }
  }

  Ok(())
}
