use crate::cli::commands::{
    delete_and_cancel_session, delete_boot_parameters,
    delete_configurations_and_derivatives, delete_group,
    delete_hw_component_cluster, delete_images, delete_kernel_parameters,
    delete_node, delete_redfish_endpoint,
};
use crate::common::kafka::Kafka;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;

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
        let label: &String = cli_delete_group
            .get_one("VALUE")
            .expect("ERROR - group name argument is mandatory");
        let force: bool = *cli_delete_group
            .get_one("force")
            .expect("The 'force' argument must have a value");
        delete_group::exec(backend, site_name, label, force, kafka_audit_opt)
            .await?;
    } else if let Some(cli_delete_node) = cli_delete.subcommand_matches("node")
    {
        let id: &String = cli_delete_node
            .get_one("VALUE")
            .expect("ERROR - group name argument is mandatory");
        delete_node::exec(backend, site_name, id).await?;
    } else if let Some(cli_delete_hw_configuration) =
        cli_delete.subcommand_matches("hardware")
    {
        let dryrun = cli_delete_hw_configuration.get_flag("dry-run");
        let delete_hsm_group =
            cli_delete_hw_configuration.get_flag("delete-hsm-group");
        let target_hsm_group_name_arg_opt: Option<&String> =
            cli_delete_hw_configuration.get_one("target-cluster");
        let parent_hsm_group_name_arg_opt: Option<&String> =
            cli_delete_hw_configuration.get_one("parent-cluster");
        let pattern = cli_delete_hw_configuration
            .get_one::<String>("pattern")
            .unwrap();

        delete_hw_component_cluster::exec(
            backend,
            site_name,
            target_hsm_group_name_arg_opt,
            settings_hsm_group_name_opt,
            parent_hsm_group_name_arg_opt,
            pattern,
            dryrun,
            delete_hsm_group,
        )
        .await?;
    } else if let Some(cli_delete_boot_parameters) =
        cli_delete.subcommand_matches("boot-parameters")
    {
        let xnames: Option<&String> =
            cli_delete_boot_parameters.get_one("hosts");
        let hosts: Vec<String> = xnames
            .cloned()
            .unwrap_or_default()
            .split(',')
            .map(String::from)
            .collect();
        delete_boot_parameters::exec(backend, site_name, hosts).await?;
    } else if let Some(cli_delete_redfish_endpoint) =
        cli_delete.subcommand_matches("redfish-endpoint")
    {
        let id: &String = cli_delete_redfish_endpoint.get_one("id").expect(
            "ERROR - host argument is mandatory. Please provide the host to delete",
        );
        delete_redfish_endpoint::exec(backend, site_name, id).await?;
    } else if let Some(cli_delete_kernel_parameters) =
        cli_delete.subcommand_matches("kernel-parameters")
    {
        let hsm_group_name_arg_opt =
            cli_delete_kernel_parameters.get_one("hsm-group");
        let nodes = cli_delete_kernel_parameters.get_one::<String>("nodes");
        let kernel_parameters = cli_delete_kernel_parameters
            .get_one::<String>("VALUE")
            .unwrap();
        let assume_yes: bool =
            cli_delete_kernel_parameters.get_flag("assume-yes");
        let do_not_reboot: bool =
            cli_delete_kernel_parameters.get_flag("do-not-reboot");
        let dryrun = cli_delete_kernel_parameters.get_flag("dry-run");

        delete_kernel_parameters::exec(
            backend.clone(),
            site_name,
            settings_hsm_group_name_opt,
            hsm_group_name_arg_opt,
            nodes,
            kernel_parameters,
            assume_yes,
            do_not_reboot,
            kafka_audit_opt,
            dryrun,
        )
        .await?;
    } else if let Some(cli_delete_session) =
        cli_delete.subcommand_matches("session")
    {
        let session_name = cli_delete_session
            .get_one::<String>("SESSION_NAME")
            .expect("'session-name' argument must be provided");
        let assume_yes: bool = cli_delete_session.get_flag("assume-yes");
        let dry_run: bool = cli_delete_session.get_flag("dry-run");

        let result = delete_and_cancel_session::exec(
            backend.clone(),
            site_name,
            shasta_base_url,
            shasta_root_cert,
            settings_hsm_group_name_opt,
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

        let result = delete_configurations_and_derivatives::exec(
            backend.clone(),
            site_name,
            shasta_base_url,
            shasta_root_cert,
            settings_hsm_group_name_opt,
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
        let image_id_vec: Vec<&str> = cli_delete_images
            .get_one::<String>("IMAGE_LIST")
            .expect("'IMAGE_LIST' argument must be provided")
            .split(",")
            .map(|image_id_str| image_id_str.trim())
            .collect();
        let dry_run: bool = cli_delete_images.get_flag("dry-run");

        match delete_images::command::exec(
            backend,
            site_name,
            shasta_base_url,
            shasta_root_cert,
            settings_hsm_group_name_opt,
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
