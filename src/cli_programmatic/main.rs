use clap::{App, Arg, SubCommand};

pub fn get_matches() {
    let cli_root = command!()
        .arg_required_else_help(true) 
        .subcommand(
            Command::new("get")
                .arg_required_else_help(true)
                .about("Get information from Shasta system")
                .subcommand(
                    Command::new("session")
                        .arg_required_else_help(true)
                        .about("Get information from Shasta CFS session")
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-n --name <VALUE> "session name"))
                        .arg(arg!(-m --most_recent <VALUE> "most recent (equivalent to --limit 1)"))
                        .arg(
                            arg!(-l --limit <VALUE> "number of CFS sessions to show on screen")
                                .value_parser(value_parser!(u8).range(1..)),
                        )
                        .group(ArgGroup::new("cluster_or_session").args(["cluster", "name"]))
                        .group(ArgGroup::new("session_limit").args(["most_recent", "limit"]))
                )
                .subcommand(
                    Command::new("configuration")
                        .arg_required_else_help(true)
                        .about("Get information from Shasta CFS configuration")
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-n --name <VALUE> "configuration name"))
                        .arg(arg!(-m --most_recent <VALUE> "most recent (equivalent to --limit 1)"))
                        .arg(
                            arg!(-l --limit <VALUE> "number of CFS configurations to show on screen")
                                .value_parser(value_parser!(u8).range(1..)),
                        )
                        .group(ArgGroup::new("cluster_or_configuration").args(["cluster", "name"]))
                        .group(ArgGroup::new("configuration_limit").args(["most_recent", "limit"]))
                )
                .subcommand(
                    Command::new("template")
                        .arg_required_else_help(true)
                        .about("Get information from Shasta BOS template")
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-n --name <VALUE> "template name"))
                        .arg(arg!(-m --most_recent <VALUE> "most recent (equivalent to --limit 1)"))
                        .arg(
                            arg!(-l --limit <VALUE> "number of BOS templates to show on screen")
                                .value_parser(value_parser!(u8).range(1..)),
                        )
                        .group(ArgGroup::new("cluster_or_template").args(["cluster", "name"]))
                        .group(ArgGroup::new("template_limit").args(["most_recent", "limit"]))
                )
                .subcommand(
                    Command::new("node")
                        .arg_required_else_help(true)
                        .about("Get members of a cluster")
                        .arg(arg!(<VALUE> "cluster name"))
                )
                .subcommand(
                    Command::new("cluster")
                    .arg_required_else_help(true)
                    .about("Get cluster details")
                    .arg(arg!(<VALUE> "cluster name"))
                )
        )
        .subcommand(
            Command::new("apply")
                .arg_required_else_help(true)
                .about("Make changes to Shasta cluster/nodes")
                .subcommand(
                    Command::new("session")
                        .about("apply session about")
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-n --name <VALUE> "session name"))
                        .group(ArgGroup::new("session").args(["cluster", "name"])),
                )
                .subcommand(
            Command::new("node")
                .arg_required_else_help(true)
                .about("Make changes to nodes")
                .subcommand(
                    Command::new("on")
                        .about("Start a node")
                        .arg(arg!(-r --reason <VALUE> "reason to power on"))
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-x --xnames <VALUE> "nodes xname"))
                        .group(ArgGroup::new("cluster_or_xnames").args(["cluster", "xnames"]))
                    .subcommand(Command::new("off")
                        .arg_required_else_help(true)
                        .about("Shutdown a node")
                        .arg(arg!(-r --reason <VALUE> "reason to power shutdown"))
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-x --xnames <VALUE> "nodes xname"))
                        .arg(arg!(-f --force "force"))
                        .group(ArgGroup::new("cluster_or_xnames").args(["cluster", "xnames"]))
                    )
                    .subcommand(Command::new("reset")
                        .arg_required_else_help(true)
                        .about("Restart a node")
                        .arg(arg!(-r --reason <VALUE> "reason to restart"))
                        .arg(arg!(-c --cluster <VALUE> "cluster name"))
                        .arg(arg!(-x --xnames <VALUE> "nodes xname"))
                        .arg(arg!(-f --force "force"))
                        .group(ArgGroup::new("cluster_or_xnames").args(["cluster", "xnames"]))
                    )
                )
            )
        )
        .subcommand(
            Command::new("log")
                .arg_required_else_help(true)
                .about("log about")
                .arg(arg!(-s --session_name <VALUE> "session name"))
                .arg(arg!(-l --layer_id <VALUE> "layer id").required(false)),
        )
        .subcommand(
            Command::new("console")
                .arg_required_else_help(true)
                .about("Access node's console")
                .arg(arg!(<VALUE> "node xname"))
        )
        .get_matches();
}

pub fn process_command() {
    if let Some(cli_get) = cli_root.subcommand_matches("get") {
        println!("this is a GET");
        if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration") {
            println!("this is a GET CONFIGURATION");

                    configuration_name = configuration.name;
                    cluster_name = configuration.cluster_name;
                    most_recent = configuration.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = configuration.limit_number;
                    }

                    // Get CFS configurations
                    let cfs_configurations = shasta_cfs_configuration::http_client::get(
                        &shasta_token,
                        &shasta_base_url,
                        &cluster_name,
                        &configuration_name,
                        &limit_number,
                    )
                    .await?;

                    if cfs_configurations.is_empty() {
                        println!("No CFS configuration found!");
                        return Ok(());
                    } else if cfs_configurations.len() == 1 {
                        let most_recent_cfs_configuration = &cfs_configurations[0];

                        let mut layers: Vec<manta_cfs_configuration::Layer> = vec![];
                        for layer in most_recent_cfs_configuration["layers"].as_array().unwrap() {
                            let gitea_commit_details = gitea::http_client::get_commit_details(
                                layer["cloneUrl"].as_str().unwrap(),
                                layer["commit"].as_str().unwrap(),
                                &gitea_token,
                            )
                            .await?;

                            layers.push(manta_cfs_configuration::Layer::new(
                                layer["name"].as_str().unwrap(),
                                layer["cloneUrl"]
                                    .as_str()
                                    .unwrap()
                                    .trim_start_matches("https://api-gw-service-nmn.local/vcs/")
                                    .trim_end_matches(".git"),
                                layer["commit"].as_str().unwrap(),
                                gitea_commit_details["commit"]["committer"]["name"]
                                    .as_str()
                                    .unwrap(),
                                gitea_commit_details["commit"]["committer"]["date"]
                                    .as_str()
                                    .unwrap(),
                            ));
                        }

                        manta_cfs_configuration::print_table(
                            manta::cfs::configuration::Configuration::new(
                                most_recent_cfs_configuration["name"].as_str().unwrap(),
                                most_recent_cfs_configuration["lastUpdated"]
                                    .as_str()
                                    .unwrap(),
                                layers,
                            ),
                        );
                    } else {
                        shasta_cfs_configuration::utils::print_table(cfs_configurations);
                    }
 
        }
        if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
            println!("this is a GET SESSION");
                    session_name = session.name;
                    cluster_name = session.cluster_name;
                    most_recent = session.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = session.limit_number;
                    }

                    let cfs_sessions = shasta_cfs_session::http_client::get(
                        &shasta_token,
                        &shasta_base_url,
                        &cluster_name,
                        &session_name,
                        &limit_number,
                    )
                    .await?;

                    if cfs_sessions.is_empty() {
                        log::info!("No CFS session found!");
                        return Ok(());
                    } else {
                        shasta_cfs_session::utils::print_table(cfs_sessions);
                    }
                }
         }
        if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
            println!("this is a GET TEMPLATE");
                     template_name = template.name;
                    cluster_name = template.cluster_name;
                    most_recent = template.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = template.limit_number;
                    }

                    let bos_templates = bos_template::http_client::get(
                        &shasta_token,
                        &shasta_base_url,
                        &cluster_name,
                        &template_name,
                        &limit_number,
                    )
                    .await?;

                    if bos_templates.is_empty() {
                        log::info!("No BOS template found!");
                        return Ok(());
                    } else {
                        bos_template::utils::print_table(bos_templates);
                    }
        }
        if let Some(cli_get_node) = cli_get.subcommand_matches("node") {
            println!("this is a GET NODE");
                      let cluster_name = node.cluster_name;

                    let nodes = shasta::hsm::http_client::get_hsm_groups(
                        &shasta_token,
                        &shasta_base_url,
                        cluster_name,
                    )
                    .await?;

                    if nodes.is_empty() {
                        log::info!("No nodes found!");
                        return Ok(());
                    } else {
                        shasta::hsm::utils::print_table(nodes);
                    }
       }
        if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
            println!("this is a GET CLUSTER");
                    let cluster_name = get_cluster_args.cluster_name;

                    let clusters =
                        cluster_ops::get_details(&shasta_token, &shasta_base_url, &cluster_name)
                            .await;

                    // println!("{:#?}", clusters);

                    for cluster in clusters {
                        println!("************************* CLUSTER *************************");

                        println!(" * HSM group label: {}", cluster.hsm_group_label);
                        println!(" * CFS configuration details:");
                        println!(
                            "   - name: {}",
                            cluster.most_recent_cfs_configuration_name_created["name"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - last updated: {}",
                            cluster.most_recent_cfs_configuration_name_created["lastUpdated"]
                                .as_str()
                                .unwrap_or_default()
                        );

                        let mut i = 0;
                        for layer in cluster.most_recent_cfs_configuration_name_created["layers"]
                            .as_array()
                            .unwrap()
                        {
                            println!("   + Layer {}", i);
                            println!(
                                "     - name: {}",
                                layer["name"].as_str().unwrap_or_default()
                            );
                            println!(
                                "     - url: {}",
                                layer["cloneUrl"].as_str().unwrap_or_default()
                            );
                            println!(
                                "     - commit: {}",
                                layer["commit"].as_str().unwrap_or_default()
                            );
                            println!(
                                "     - playbook: {}",
                                layer["playbook"].as_str().unwrap_or_default()
                            );
                            i += 1;
                        }

                        println!(" * CFS session details:");
                        println!(
                            "   - Name: {}",
                            cluster.most_recent_cfs_session_name_created["name"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - Configuration name: {}",
                            cluster.most_recent_cfs_session_name_created["configuration"]["name"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - Target: {}",
                            cluster.most_recent_cfs_session_name_created["target"]["definition"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!("   + Ansible details:");
                        println!(
                            "     - name: {}",
                            cluster.most_recent_cfs_session_name_created["ansible"]["config"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - limit: {}",
                            cluster.most_recent_cfs_session_name_created["ansible"]["limit"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!("   + Status:");
                        println!(
                            "     - status: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["status"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - succeeded: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["succeeded"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - job: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["job"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - start: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["startTime"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - tags: {}",
                            cluster.most_recent_cfs_session_name_created["tags"]
                        );

                        println!(" * members: {}", nodes::nodes_to_string(&cluster.members));
                    }
    }

    if let Some(get_apply) = cli_root.subcommand_matches("apply") {
        println!("this is an APPLY");
        if let Some(cli_apply_session) = get_apply.subcommand_matches("session") {
            println!("this is an APPLY SESSION");
                    let cfs_session_name = create_cfs_session_from_repo::run(
                        &apply_session_params.session_name,
                        apply_session_params.repo_path,
                        gitea_token,
                        shasta_token,
                        shasta_base_url,
                        apply_session_params.ansible_limit,
                        apply_session_params.ansible_verbosity,
                    )
                    .await;

                    if apply_session_params.watch_logs {
                        log::info!("Fetching logs ...");
                        shasta_cfs_session_logs::client::session_logs(
                            cfs_session_name.unwrap().as_str(),
                            None,
                        )
                        .await?;
                    }
        }
        if let Some(cli_apply_node) = get_apply.subcommand_matches("node") {
            println!("this is an APPLY NODE");
            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                println!("this is an APPLY NODE ON");
                            let xnames;
                            if main_apply_node_on_subcommand.xnames.is_some() {
                                // user provides a list of xnames
                                xnames = main_apply_node_on_subcommand
                                    .xnames
                                    .unwrap()
                                    .split(',')
                                    .map(|xname| String::from(xname.trim()))
                                    .collect();
                            } else {
                                // user provides a cluster name
                                let hsm_groups = shasta::hsm::http_client::get_hsm_groups(
                                    &shasta_token,
                                    &shasta_base_url,
                                    main_apply_node_on_subcommand.cluster_name,
                                )
                                .await?;
                                xnames = hsm_groups[0]["members"]["ids"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|xname_value| String::from(xname_value.as_str().unwrap()))
                                    .collect();
                            }
                            log::info!("Servers to turn on: {:?}", xnames);
                            capmc::http_client::node_power_on::post(
                                shasta_token.to_string(),
                                main_apply_node_on_subcommand.reason,
                                xnames,
                                false,
                            )
                            .await?; // TODO: idk why power on does not seems to work when forced
             }
            if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
                println!("this is an APPLY NODE OFF");
                            let xnames;
                            if main_apply_node_off_subcommand.xnames.is_some() {
                                // user provides a list of xnames
                                xnames = main_apply_node_off_subcommand
                                    .xnames
                                    .unwrap()
                                    .split(',')
                                    .map(|xname| String::from(xname.trim()))
                                    .collect();
                            } else {
                                // user provides a cluster name
                                let hsm_groups = shasta::hsm::http_client::get_hsm_groups(
                                    &shasta_token,
                                    &shasta_base_url,
                                    main_apply_node_off_subcommand.cluster_name,
                                )
                                .await?;
                                xnames = hsm_groups[0]["members"]["ids"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|xname_value| String::from(xname_value.as_str().unwrap()))
                                    .collect();
                            }
                            log::info!("Servers to turn off: {:?}", xnames);
                            capmc::http_client::node_power_off::post(
                                shasta_token.to_string(),
                                main_apply_node_off_subcommand.reason,
                                xnames,
                                main_apply_node_off_subcommand.force,
                            )
                            .await?;
            }
            if let Some(cli_apply_node_reset) = cli_apply_node.subcommand_matches("reset") {
                println!("this is an APPLY NODE RESET");
                            let xnames;
                            if main_apply_node_reset_subcommand.xnames.is_some() {
                                // user provides a list of xnames
                                xnames = main_apply_node_reset_subcommand
                                    .xnames
                                    .unwrap()
                                    .split(',')
                                    .map(|xname| String::from(xname.trim()))
                                    .collect();
                            } else {
                                // user provides a cluster name
                                let hsm_groups = shasta::hsm::http_client::get_hsm_groups(
                                    &shasta_token,
                                    &shasta_base_url,
                                    main_apply_node_reset_subcommand.cluster_name,
                                )
                                .await?;
                                xnames = hsm_groups[0]["members"]["ids"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|xname_value| String::from(xname_value.as_str().unwrap()))
                                    .collect();
                            }
                            log::info!("Servers to reboot: {:?}", xnames);
                            capmc::http_client::node_restart::post(
                                shasta_token.to_string(),
                                main_apply_node_reset_subcommand.reason,
                                xnames,
                                main_apply_node_reset_subcommand.force,
                            )
                            .await?;
            }
        }
    }

    if let Some(get_log) = cli_root.subcommand_matches("log") {
        println!("this is a LOG");
            logging_session_name = log_cmd.session_name;
            layer_id = log_cmd.layer_id;
            shasta_cfs_session_logs::client::session_logs_proxy(
                &shasta_token,
                &shasta_base_url,
                &None,
                &Some(logging_session_name),
                layer_id,
            )
            .await?;
     }

    if let Some(get_console) = cli_root.subcommand_matches("console") {
        println!("this is a CONSOLE");
            xname = console_cmd.xname;

            connect_to_console(&xname).await?;
     }
    
}
