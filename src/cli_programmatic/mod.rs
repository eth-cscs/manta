use std::collections::HashSet;

use crate::cluster_ops::ClusterDetails;
use crate::shasta::nodes;
use crate::{
    cluster_ops, create_cfs_session_from_repo, gitea, manta, shasta, shasta_cfs_session_logs,
};
use clap::{arg, command, value_parser, ArgAction, Command};

use clap::{ArgGroup, ArgMatches};

use crate::manta::cfs::configuration as manta_cfs_configuration;
use crate::node_console::connect_to_console;

use crate::shasta::{
    bos_template, capmc,
    cfs::{configuration as shasta_cfs_configuration, session as shasta_cfs_session},
};

use crate::node_ops;

pub fn subcommand_get_cfs_session(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_session = Command::new("session");

    get_cfs_session = get_cfs_session.arg(arg!(-n --name <VALUE> "session name"));
    get_cfs_session = get_cfs_session.arg(
        arg!(-m --most_recent <VALUE> "most recent (equivalent to --limit 1)")
            .action(ArgAction::SetTrue),
    );
    get_cfs_session = get_cfs_session.arg(
        arg!(-l --limit <VALUE> "number of CFS sessions to show on screen")
            .value_parser(value_parser!(u8).range(1..)),
    );

    let about_msg = "Get information from Shasta CFS session";

    match hsm_group {
        None => {
            get_cfs_session = get_cfs_session
                .arg(arg!(-c --cluster <VALUE> "cluster name"))
                .group(ArgGroup::new("cluster_or_session").args(["cluster", "name"]))
                .about(about_msg)
        }
        Some(hsm_group_value) => {
            get_cfs_session =
                get_cfs_session.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    get_cfs_session =
        get_cfs_session.group(ArgGroup::new("session_limit").args(["most_recent", "limit"]));

    get_cfs_session
}

pub fn subcommand_get_cfs_configuration(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_configuration = Command::new("configuration");
    get_cfs_configuration =
        get_cfs_configuration.about("Get information from Shasta CFS configuration");

    get_cfs_configuration = get_cfs_configuration.arg(arg!(-n --name <VALUE> "configuration name"));
    get_cfs_configuration = get_cfs_configuration.arg(
        arg!(-m --most_recent <VALUE> "most recent (equivalent to --limit 1)")
            .action(ArgAction::SetTrue),
    );
    get_cfs_configuration = get_cfs_configuration.arg(
        arg!(-l --limit <VALUE> "number of CFS configurations to show on screen")
            .value_parser(value_parser!(u8).range(1..)),
    );

    let about_msg = "Get information from Shasta BOS template";

    match hsm_group {
        None => {
            get_cfs_configuration = get_cfs_configuration
                .arg(arg!(-c --cluster <VALUE> "cluster name"))
                .group(ArgGroup::new("cluster_or_configuration").args(["cluster", "name"]))
                .about(about_msg)
        }
        Some(hsm_group_value) => {
            get_cfs_configuration = get_cfs_configuration
                .about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    get_cfs_configuration = get_cfs_configuration
        .group(ArgGroup::new("configuration_limit").args(["most_recent", "limit"]));

    get_cfs_configuration
}

pub fn subcommand_get_bos_template(hsm_group: Option<&String>) -> Command {
    let mut get_bos_template = Command::new("template");

    get_bos_template = get_bos_template.arg(arg!(-n --name <VALUE> "template name"));
    get_bos_template = get_bos_template.arg(
        arg!(-m --most_recent <VALUE> "most recent (equivalent to --limit 1)")
            .action(ArgAction::SetTrue),
    );
    get_bos_template = get_bos_template.arg(
        arg!(-l --limit <VALUE> "number of BOS templates to show on screen")
            .value_parser(value_parser!(u8).range(1..)),
    );

    let about_msg = "Get information from Shasta BOS template";

    match hsm_group {
        None => {
            get_bos_template = get_bos_template
                .arg(arg!(-c --cluster <VALUE> "cluster name"))
                .group(ArgGroup::new("cluster_or_template").args(["cluster", "name"]))
                .about(about_msg)
        }
        Some(hsm_group_value) => {
            get_bos_template =
                get_bos_template.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    // get_bos_template =
    //     get_bos_template.group(ArgGroup::new("template-limit").args(["most_recent", "limit"]));

    get_bos_template
}

pub fn subcommand_get_node(hsm_group: Option<&String>) -> Command {
    let mut get_node = Command::new("node");
    get_node = get_node.arg_required_else_help(true);

    let about_msg = "Get members of a cluster";

    match hsm_group {
        None => get_node = get_node.arg(arg!(<VALUE> "cluster name")).about(about_msg),
        Some(hsm_group_value) => {
            get_node = get_node.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    get_node
}

pub fn subcommand_get_cluster(hsm_group: Option<&String>) -> Command {
    let mut get_cluster = Command::new("cluster");

    let about_msg = "Get cluster details";

    match hsm_group {
        None => {
            get_cluster = get_cluster.arg_required_else_help(true);
            get_cluster = get_cluster
                .arg(arg!(<VALUE> "cluster name"))
                .about(about_msg)
        }
        Some(hsm_group_value) => {
            get_cluster = get_cluster.arg_required_else_help(false);
            get_cluster =
                get_cluster.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value));
        }
    }

    get_cluster
}

pub fn subcommand_get(hsm_group: Option<&String>) -> Command {
    let get = Command::new("get")
        .arg_required_else_help(true)
        .about("Get information from Shasta system")
        .subcommand(subcommand_get_cfs_session(hsm_group))
        .subcommand(subcommand_get_cfs_configuration(hsm_group))
        .subcommand(subcommand_get_bos_template(hsm_group))
        .subcommand(subcommand_get_node(hsm_group))
        .subcommand(subcommand_get_cluster(hsm_group));

    get
}

pub fn subcommand_apply_session(hsm_group: Option<&String>) -> Command {
    let mut apply_session = Command::new("session")
        .arg_required_else_help(true)
        .arg(arg!(-n --name <VALUE> "Session name").required(true))
        .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image")
            .required(true))
        .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS sesion")
            .required(true))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to aee container running ansible scripts"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4", "5"])
            .num_args(1)
            .require_equals(true)
            .default_value("2")
            .default_missing_value("2"));

    let about_msg = "Create a CFS configuration and a session against hsm_group/nodes";

    match hsm_group {
        None => apply_session = apply_session.about(about_msg),
        Some(hsm_group_value) => {
            apply_session =
                apply_session.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    apply_session
}

pub fn subcommand_apply_node_on(hsm_group: Option<&String>) -> Command {
    let mut apply_node_on = Command::new("on")
        .arg_required_else_help(true)
        .arg(arg!(-r --reason <VALUE> "reason to power on"))
        .arg(arg!(-x --xnames <VALUE> "nodes xname"));

    let about_msg = "Start a node";

    match hsm_group {
        None => apply_node_on = apply_node_on.about(about_msg),
        Some(hsm_group_value) => {
            apply_node_on =
                apply_node_on.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    apply_node_on
}

pub fn subcommand_apply_node_off(hsm_group: Option<&String>) -> Command {
    let mut apply_node_off = Command::new("off")
        .arg_required_else_help(true)
        .arg(arg!(-x --xnames <VALUE> "nodes xname"))
        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
        .arg(arg!(-r --reason <VALUE> "reason to power off"))
        .group(ArgGroup::new("cluster_or_xnames").args(["cluster", "xnames"]));

    let about_msg = "Shutdown a node";

    match hsm_group {
        None => {
            apply_node_off = apply_node_off
                .arg(arg!(-c --cluster <VALUE> "cluster name"))
                .about(about_msg)
        }
        Some(hsm_group_value) => {
            apply_node_off =
                apply_node_off.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    apply_node_off
}

pub fn subcommand_apply_node_reset(hsm_group: Option<&String>) -> Command {
    let mut apply_node_reset = Command::new("reset")
        .arg_required_else_help(true)
        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
        .arg(arg!(-r --reason <VALUE> "reason to reset"))
        .arg(arg!(-x --xnames <VALUE> "nodes xname"))
        .group(ArgGroup::new("cluster_or_xnames").args(["cluster", "xnames"]));

    let about_msg = "Shutdown a node";

    match hsm_group {
        None => {
            apply_node_reset = apply_node_reset
                .arg(arg!(-c --cluster <VALUE> "cluster name"))
                .about(about_msg)
        }
        Some(hsm_group_value) => {
            apply_node_reset =
                apply_node_reset.about(format!("{}\nCLUSTER NAME: {}", about_msg, hsm_group_value))
        }
    }

    apply_node_reset
}

pub fn get_matches(hsm_group: Option<&String>) -> ArgMatches {
    command!()
        .arg_required_else_help(true)
        .subcommand(subcommand_get(hsm_group))
        .subcommand(
            Command::new("apply")
                .arg_required_else_help(true)
                .about("Make changes to Shasta cluster/nodes")
                .subcommand(subcommand_apply_session(hsm_group))
                .subcommand(
                    Command::new("node")
                        .arg_required_else_help(true)
                        .about("Make changes to nodes")
                        .subcommand(subcommand_apply_node_on(hsm_group))
                        .subcommand(subcommand_apply_node_off(hsm_group))
                        .subcommand(subcommand_apply_node_reset(hsm_group)),
                ),
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
                .arg(arg!(<VALUE> "node xname")),
        )
        .get_matches()
}

pub async fn process_command(
    cli_root: ArgMatches,
    shasta_token: String,
    shasta_base_url: String,
    gitea_token: String,
    hsm_group: Option<&String>,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
    let cluster_name;
    let most_recent;
    let configuration_name;
    let session_name;
    let limit_number;
    let template_name;
    let logging_session_name;
    // let xname;
    let layer_id;

    if let Some(cli_get) = cli_root.subcommand_matches("get") {
        if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration") {
            configuration_name = cli_get_configuration.get_one::<String>("name");

            cluster_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_configuration.get_one::<String>("cluster_name"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };

            most_recent = cli_get_configuration.get_one::<bool>("most_recent");

            if most_recent.is_some() {
                limit_number = Some(&1);
            } else {
                limit_number = cli_get_configuration.get_one::<u8>("limit_number");
            }

            // Get CFS configurations
            let cfs_configurations = shasta_cfs_configuration::http_client::get(
                &shasta_token,
                &shasta_base_url,
                cluster_name,
                configuration_name,
                limit_number,
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
        } else if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
            session_name = cli_get_session.get_one::<String>("name");

            cluster_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_session.get_one::<String>("cluster_name"),
                Some(hsm_group_val) => Some(&hsm_group_val),
            };

            most_recent = cli_get_session.get_one::<bool>("most_recent");

            if most_recent.is_some() {
                limit_number = Some(&1);
            } else {
                limit_number = cli_get_session.get_one::<u8>("limit_number");
            }

            let cfs_sessions = shasta_cfs_session::http_client::get(
                &shasta_token,
                &shasta_base_url,
                cluster_name,
                session_name,
                limit_number,
            )
            .await?;

            if cfs_sessions.is_empty() {
                log::info!("No CFS session found!");
                return Ok(());
            } else {
                shasta_cfs_session::utils::print_table(cfs_sessions);
            }
        } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
            template_name = cli_get_template.get_one::<String>("name");

            cluster_name = match &hsm_group {
                None => cli_get_template.get_one::<String>("cluster_name"),
                Some(hsm_group_val) => Some(&hsm_group_val),
            };

            most_recent = cli_get_template.get_one::<bool>("most_recent");

            if most_recent.is_some() {
                limit_number = Some(&1);
            } else {
                limit_number = cli_get_template.get_one::<u8>("limit_number");
            }

            let bos_templates = bos_template::http_client::get(
                &shasta_token,
                &shasta_base_url,
                cluster_name,
                template_name,
                limit_number,
            )
            .await?;

            if bos_templates.is_empty() {
                log::info!("No BOS template found!");
                return Ok(());
            } else {
                bos_template::utils::print_table(bos_templates);
            }
        } else if let Some(cli_get_node) = cli_get.subcommand_matches("node") {
            cluster_name = match hsm_group {
                None => cli_get_node.get_one::<String>("cluster_name"),
                Some(_) => hsm_group,
            };

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
        } else if let Some(cli_get_cluster) = cli_get.subcommand_matches("cluster") {
            let cluster_name = match hsm_group {
                None => cli_get_cluster.get_one::<String>("cluster_name").unwrap(),
                Some(cluster_name_value) => cluster_name_value,
            };

            let clusters =
                cluster_ops::get_details(&shasta_token, &shasta_base_url, cluster_name).await;

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
                    cluster.most_recent_cfs_session_name_created["status"]["session"]["status"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - succeeded: {}",
                    cluster.most_recent_cfs_session_name_created["status"]["session"]["succeeded"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - job: {}",
                    cluster.most_recent_cfs_session_name_created["status"]["session"]["job"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - start: {}",
                    cluster.most_recent_cfs_session_name_created["status"]["session"]["startTime"]
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
    } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
        if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
            
            // Code below inspired on https://github.com/rust-lang/git2-rs/issues/561
            let included: HashSet<String>;
            let excluded: HashSet<String>;
            // Check andible limit matches the nodes in hsm_group
            let hsm_groups;

            if hsm_group.is_some() {
                hsm_groups =
                    cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group.unwrap())
                        .await;
            } else {
                hsm_groups = cluster_ops::get_details(&shasta_token, &shasta_base_url, "").await;
            }

            // Take all nodes for all clusters found and put them in a Set
            let hsm_groups_nodes = hsm_groups
                .iter()
                .flat_map(|cluster| {
                    cluster
                        .members
                        .iter()
                        .map(|xname| xname.as_str().unwrap().to_string())
                })
                .collect();

            // Get Vec with all nodes from ansible-limit param
            let ansible_limit_nodes_in_cfs_sessions: HashSet<String> = cli_apply_session
                .get_one::<String>("ansible-limit")
                .unwrap()
                .replace(" ", "") // trim xnames by removing white spaces
                .split(",")
                .map(|xname| xname.to_string())
                .collect();

            (included, excluded) = node_ops::check_hsm_group_and_ansible_limit(
                &hsm_groups_nodes,
                ansible_limit_nodes_in_cfs_sessions,
            );

            if !excluded.is_empty() {
                println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                    excluded,
                    hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                    hsm_groups_nodes);
                std::process::exit(-1);
            }

            let cfs_session_name = create_cfs_session_from_repo::run(
                cli_apply_session.get_one::<String>("name").unwrap(),
                vec![cli_apply_session
                    .get_one::<String>("repo-path")
                    .unwrap()
                    .to_string()],
                gitea_token,
                shasta_token,
                shasta_base_url,
                included.into_iter().collect::<Vec<String>>().join(","), // Convert Hashset to String with comma separator, need to convert to Vec first following https://stackoverflow.com/a/47582249/1918003
                cli_apply_session
                    .get_one::<String>("ansible-verbosity")
                    .unwrap()
                    .parse()
                    .unwrap(),
            )
            .await;

            if cli_apply_session.get_one::<bool>("watch-logs").is_some() {
                log::info!("Fetching logs ...");
                shasta_cfs_session_logs::client::session_logs(
                    cfs_session_name.unwrap().as_str(),
                    None,
                )
                .await?;
            }
        } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {

            let included: HashSet<String>;
            let excluded: HashSet<String>;

            let hsm_groups: Vec<ClusterDetails>;

            if hsm_group.is_some() {
                // hsm_group value provided
                hsm_groups =
                    cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group.unwrap())
                        .await;
            } else {
                // no hsm_group value provided
                hsm_groups = cluster_ops::get_details(&shasta_token, &shasta_base_url, "").await;
            }

            // Take all nodes for all clusters found and put them in a Set
            let hsm_groups_nodes = hsm_groups
                .iter()
                .flat_map(|cluster| {
                    cluster
                        .members
                        .iter()
                        .map(|xname| xname.as_str().unwrap().to_string())
                })
                .collect();

            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {

                // User provided list of xnames to power on
                let xnames: HashSet<String> = cli_apply_node_on
                    .get_one::<String>("xnames")
                    .unwrap()
                    .replace(" ", "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect();

                (included, excluded) =
                    node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

                if !excluded.is_empty() {
                    println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                        excluded,
                        hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                        hsm_groups_nodes);
                    std::process::exit(-1);
                }

                log::info!("Servers to power on: {:?}", included);

                capmc::http_client::node_power_on::post(
                    shasta_token.to_string(),
                    cli_apply_node_on.get_one::<String>("reason"),
                    included.into_iter().collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
                    false,
                )
                .await?;

            } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {

                // User provided list of xnames to power off
                let xnames: HashSet<String> = cli_apply_node_off
                    .get_one::<String>("xnames")
                    .unwrap()
                    .replace(" ", "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect();

                (included, excluded) =
                    node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

                if !excluded.is_empty() {
                    println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                        excluded,
                        hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                        hsm_groups_nodes);
                    std::process::exit(-1);
                }

                log::info!("Servers to power on: {:?}", included);

                capmc::http_client::node_power_on::post(
                    shasta_token.to_string(),
                    cli_apply_node_off.get_one::<String>("reason"),
                    included.into_iter().collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
                    false,
                )
                .await?;

            } else if let Some(cli_apply_node_reset) = cli_apply_node.subcommand_matches("reset") {
                
                // User provided list of xnames to power reset
                let xnames: HashSet<String> = cli_apply_node_reset
                    .get_one::<String>("xnames")
                    .unwrap()
                    .replace(" ", "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect();

                (included, excluded) =
                    node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

                if !excluded.is_empty() {
                    println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                        excluded,
                        hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                        hsm_groups_nodes);
                    std::process::exit(-1);
                }

                log::info!("Servers to power reset: {:?}", included);

                capmc::http_client::node_power_off::post(
                    shasta_token.to_string(),
                    cli_apply_node_reset.get_one::<String>("reason"),
                    included.into_iter().collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
                    false,
                )
                .await?; // TODO: idk why power on does not seems to work when forced

            }
        }
    } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
        logging_session_name = cli_log.get_one::<String>("session_name");
        layer_id = cli_log.get_one::<u8>("layer_id");
        shasta_cfs_session_logs::client::session_logs_proxy(
            &shasta_token,
            &shasta_base_url,
            None,
            logging_session_name,
            layer_id,
        )
        .await?;
    } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
        // xname = cli_console.get_one::<String>("xname");

        let included: HashSet<String>;
        let excluded: HashSet<String>;

        // User provided list of xnames to power reset
        let xnames: HashSet<String> = cli_console
            .get_one::<String>("xname")
            .unwrap()
            .replace(" ", "") // trim xnames by removing white spaces
            .split(',')
            .map(|xname| xname.to_string())
            .collect();

        let hsm_groups: Vec<ClusterDetails>;
            
        if hsm_group.is_some() {
            // hsm_group value provided
            hsm_groups =
                cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group.unwrap()).await;
        } else {
            // no hsm_group value provided
            hsm_groups = cluster_ops::get_details(&shasta_token, &shasta_base_url, "").await;
        }

        // Take all nodes for all clusters found and put them in a Set
        let hsm_groups_nodes = hsm_groups
            .iter()
            .flat_map(|cluster| {
                cluster
                    .members
                    .iter()
                    .map(|xname| xname.as_str().unwrap().to_string())
            })
            .collect();

        (included, excluded) =
            node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

        if !excluded.is_empty() {
            println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                excluded,
                hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                hsm_groups_nodes);
            std::process::exit(-1);
        }

        connect_to_console(included.iter().next().unwrap()).await?;
    }

    Ok(())
}
