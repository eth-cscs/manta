use crate::cli::commands::{
  delete_and_cancel_session, delete_configurations_and_derivatives,
  delete_group, delete_hw_component_cluster, delete_images,
  delete_kernel_parameters,
};
use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
  kafka::Kafka,
};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::hsm::{
  component::ComponentTrait, group::GroupTrait,
  redfish_endpoint::RedfishEndpointTrait,
};
use manta_backend_dispatcher::types::bss::BootParameters;

pub async fn handle_delete(
  cli_delete: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  settings_hsm_group_name_opt: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  if let Some(cli_delete_group) = cli_delete.subcommand_matches("group") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let label: &String = cli_delete_group
      .get_one("VALUE")
      .expect("ERROR - group name argument is mandatory");
    let force: bool = *cli_delete_group
      .get_one("force")
      .expect("The 'force' argument must have a value");
    delete_group::exec(backend, &shasta_token, label, force, kafka_audit_opt)
      .await?;
  } else if let Some(cli_delete_node) = cli_delete.subcommand_matches("node") {
    let shasta_token = get_api_token(backend, site_name).await?;
    let id: &String = cli_delete_node
      .get_one("VALUE")
      .expect("ERROR - group name argument is mandatory");
    backend.delete_node(&shasta_token, id).await?;
    println!("Node '{}' deleted", id);
  } else if let Some(cli_delete_hw_configuration) =
    cli_delete.subcommand_matches("hardware")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let dryrun = cli_delete_hw_configuration.get_flag("dry-run");
    let delete_hsm_group =
      cli_delete_hw_configuration.get_flag("delete-hsm-group");
    let target_hsm_group_name_arg_opt: Option<&String> =
      cli_delete_hw_configuration.get_one("target-cluster");
    let target_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      target_hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await?;
    let parent_hsm_group_name_arg_opt: Option<&String> =
      cli_delete_hw_configuration.get_one("parent-cluster");
    let parent_hsm_group_vec = get_groups_names_available(
      backend,
      &shasta_token,
      parent_hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await?;
    delete_hw_component_cluster::exec(
      backend,
      &shasta_token,
      target_hsm_group_vec.first().unwrap(),
      parent_hsm_group_vec.first().unwrap(),
      cli_delete_hw_configuration
        .get_one::<String>("pattern")
        .unwrap(),
      dryrun,
      delete_hsm_group,
    )
    .await?;
  } else if let Some(cli_delete_boot_parameters) =
    cli_delete.subcommand_matches("boot-parameters")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let xnames: Option<&String> = cli_delete_boot_parameters.get_one("hosts");
    let hosts: Vec<String> = xnames
      .cloned()
      .unwrap_or_default()
      .split(',')
      .map(String::from)
      .collect();
    let boot_parameters = BootParameters {
      hosts,
      macs: None,
      nids: None,
      params: "".to_string(),
      kernel: "".to_string(),
      initrd: "".to_string(),
      cloud_init: None,
    };
    let result = backend
      .delete_bootparameters(&shasta_token, &boot_parameters)
      .await;
    match result {
      Ok(_) => println!("Boot parameters deleted successfully"),
      Err(error) => eprintln!("{}", error),
    }
  } else if let Some(cli_delete_redfish_endpoint) =
    cli_delete.subcommand_matches("redfish-endpoint")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let id: &String = cli_delete_redfish_endpoint.get_one("id").expect(
      "ERROR - host argument is mandatory. Please provide the host to delete",
    );
    let result = backend.delete_redfish_endpoint(&shasta_token, id).await;
    match result {
      Ok(_) => {
        println!("Redfish endpoint for id '{}' deleted successfully", id)
      }
      Err(error) => eprintln!("{}", error),
    }
  } else if let Some(cli_delete_kernel_parameters) =
    cli_delete.subcommand_matches("kernel-parameters")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_group_name_arg_opt =
      cli_delete_kernel_parameters.get_one("hsm-group");
    let node_expression: &String = if hsm_group_name_arg_opt.is_some() {
      let hsm_group_name_vec = get_groups_names_available(
        backend,
        &shasta_token,
        hsm_group_name_arg_opt,
        settings_hsm_group_name_opt,
      )
      .await?;
      let hsm_members_rslt: Result<Vec<String>, _> = backend
        .get_member_vec_from_group_name_vec(&shasta_token, &hsm_group_name_vec)
        .await;
      match hsm_members_rslt {
        Ok(hsm_members) => &hsm_members.join(","),
        Err(e) => {
          eprintln!(
            "ERROR - could not fetch HSM groups members. Reason:\n{}",
            e.to_string()
          );
          std::process::exit(1);
        }
      }
    } else {
      cli_delete_kernel_parameters
        .get_one::<String>("nodes")
        .expect("Neither HSM group nor nodes defined")
    };
    let kernel_parameters = cli_delete_kernel_parameters
      .get_one::<String>("VALUE")
      .unwrap();
    let assume_yes: bool = cli_delete_kernel_parameters.get_flag("assume-yes");
    let do_not_reboot: bool =
      cli_delete_kernel_parameters.get_flag("do-not-reboot");
    let dryrun = cli_delete_kernel_parameters.get_flag("dry-run");
    let result = delete_kernel_parameters::exec(
      backend.clone(),
      &shasta_token,
      kernel_parameters,
      node_expression,
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
  } else if let Some(cli_delete_session) =
    cli_delete.subcommand_matches("session")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_group_available_vec = get_groups_names_available(
      backend,
      &shasta_token,
      None,
      settings_hsm_group_name_opt,
    )
    .await?;
    let session_name = cli_delete_session
      .get_one::<String>("SESSION_NAME")
      .expect("'session-name' argument must be provided");
    let assume_yes: bool = cli_delete_session.get_flag("assume-yes");
    let dry_run: bool = cli_delete_session.get_flag("dry-run");
    let result = delete_and_cancel_session::exec(
      backend.clone(),
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_available_vec,
      session_name,
      dry_run,
      assume_yes,
    )
    .await;
    if let Err(e) = result {
      eprintln!("{}", e.to_string());
      std::process::exit(1);
    }
  } else if let Some(cli_delete_configurations) =
    cli_delete.subcommand_matches("configurations")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let since_opt = if let Some(since) =
      cli_delete_configurations.get_one::<String>("since")
    {
      let date_time = chrono::NaiveDateTime::parse_from_str(
        &(since.to_string() + "T00:00:00"),
        "%Y-%m-%dT%H:%M:%S",
      )
      .unwrap();
      Some(date_time)
    } else {
      None
    };
    let until_opt = if let Some(until) =
      cli_delete_configurations.get_one::<String>("until")
    {
      let date_time = chrono::NaiveDateTime::parse_from_str(
        &(until.to_string() + "T00:00:00"),
        "%Y-%m-%dT%H:%M:%S",
      )
      .unwrap();
      Some(date_time)
    } else {
      None
    };
    let cfs_configuration_name_pattern: Option<&String> =
      cli_delete_configurations.get_one("configuration-name");
    let assume_yes = cli_delete_configurations.get_flag("assume-yes");
    if since_opt.is_some()
      && until_opt.is_some()
      && since_opt.unwrap() > until_opt.unwrap()
    {
      return Err(Error::msg(
        "ERROR - 'since' date can't be after 'until' date. Exit",
      ));
    }
    let target_hsm_group_vec =
      if let Some(settings_hsm_group_name) = settings_hsm_group_name_opt {
        vec![settings_hsm_group_name.clone()]
      } else {
        get_groups_names_available(
          backend,
          &shasta_token,
          None,
          settings_hsm_group_name_opt,
        )
        .await?
      };
    let result = delete_configurations_and_derivatives::exec(
      backend.clone(),
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      cfs_configuration_name_pattern.map(String::as_str),
      since_opt,
      until_opt,
      assume_yes,
    )
    .await;
    if let Err(e) = result {
      eprintln!("{}", e.to_string());
      std::process::exit(1);
    }
  } else if let Some(cli_delete_images) =
    cli_delete.subcommand_matches("images")
  {
    let shasta_token = get_api_token(backend, site_name).await?;
    let hsm_name_available_vec = get_groups_names_available(
      backend,
      &shasta_token,
      None,
      settings_hsm_group_name_opt,
    )
    .await?;
    let image_id_vec: Vec<&str> = cli_delete_images
      .get_one::<String>("IMAGE_LIST")
      .expect("'IMAGE_LIST' argument must be provided")
      .split(",")
      .map(|image_id_str| image_id_str.trim())
      .collect();
    let dry_run: bool = cli_delete_images.get_flag("dry-run");
    match delete_images::command::exec(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_name_available_vec,
      image_id_vec.as_slice(),
      dry_run,
    )
    .await
    {
      Ok(_) => {
        println!("Images deleted successfully");
      }
      Err(e) => {
        eprintln!("{}", e.to_string());
        std::process::exit(1);
      }
    }
  }
  Ok(())
}
