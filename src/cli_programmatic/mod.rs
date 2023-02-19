use std::collections::HashSet;

use crate::cluster_ops::ClusterDetails;
use crate::shasta::nodes;
use crate::{
    check_nodes_have_most_recent_image, cluster_ops, create_cfs_session_from_repo, gitea, manta,
    shasta_cfs_session_logs,
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
    let mut get_cfs_session = Command::new("session")
        .aliases(["s", "se", "sess"])
        .about("Get information from Shasta CFS session")
        .arg(arg!(-n --name <VALUE> "session name"))
        .arg(arg!(-m --"most-recent" "most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "number of CFS sessions to show on screen")
                .value_parser(value_parser!(u8).range(1..)),
        );

    match hsm_group {
        None => {
            get_cfs_session = get_cfs_session.arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
        }
        //                    .group(ArgGroup::new("hsm-group_or_session").args(["hsm-group", "name"]))
        ,
        Some(_) => {}
    }

    get_cfs_session =
        get_cfs_session.group(ArgGroup::new("session_limit").args(["most-recent", "limit"]));

    get_cfs_session
}

pub fn subcommand_get_cfs_configuration(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_configuration = Command::new("configuration")
        .aliases(["c", "cfg", "conf", "config"])
        .about("Get information from Shasta CFS configuration")
        .arg(arg!(-n --name <VALUE> "configuration name"))
        .arg(arg!(-m --"most-recent" "most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "number of CFS configurations to show on screen")
                .value_parser(value_parser!(u8).range(1..)),
        );

    match hsm_group {
        None => {
            get_cfs_configuration = get_cfs_configuration
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_configuration").args(["hsm-group", "name"]))
        }
        Some(_) => {}
    }

    get_cfs_configuration = get_cfs_configuration
        .group(ArgGroup::new("configuration_limit").args(["most-recent", "limit"]));

    get_cfs_configuration
}

pub fn subcommand_get_bos_template(hsm_group: Option<&String>) -> Command {
    let mut get_bos_template = Command::new("template")
        .aliases(["t", "tplt", "templ"])
        .about("Get information from Shasta BOS template")
        .arg(arg!(-n --name <VALUE> "template name"))
        .arg(arg!(-m --"most-recent" "most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "number of BOS templates to show on screen")
                .value_parser(value_parser!(u8).range(1..)),
        );

    match hsm_group {
        None => {
            get_bos_template = get_bos_template
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_template").args(["hsm-group", "name"]))
        }
        Some(_) => {}
    }

    get_bos_template
}

pub fn subcommand_get_node(hsm_group: Option<&String>) -> Command {
    let mut get_node = Command::new("node")
        .alias("n")
        .about("Get members of a HSM group");

    match hsm_group {
        None => {
            get_node = get_node
                .arg_required_else_help(true)
                .arg(arg!(<HSMGROUP> "hsm group name"))
        }
        Some(_) => {}
    }

    get_node
}

pub fn subcommand_get_hsm_groups_details(hsm_group: Option<&String>) -> Command {
    let mut get_hsm_group = Command::new("hsm-groups")
        .aliases(["h", "hg", "hsm"])
        .about("Get HSM groups details");

    match hsm_group {
        None => {
            get_hsm_group = get_hsm_group
                .arg_required_else_help(true)
                .arg(arg!(<HSMGROUP> "hsm group name"))
        }
        Some(_) => {
            get_hsm_group = get_hsm_group.arg_required_else_help(false);
        }
    }

    get_hsm_group
}

pub fn subcommand_get(hsm_group: Option<&String>) -> Command {
    Command::new("get")
        .alias("g")
        .arg_required_else_help(true)
        .about("Get information from Shasta system")
        .subcommand(subcommand_get_cfs_session(hsm_group))
        .subcommand(subcommand_get_cfs_configuration(hsm_group))
        .subcommand(subcommand_get_bos_template(hsm_group))
        .subcommand(subcommand_get_node(hsm_group))
        .subcommand(subcommand_get_hsm_groups_details(hsm_group))
}

pub fn subcommand_apply_session(hsm_group: Option<&String>) -> Command {
    let mut apply_session = Command::new("session")
        .aliases(["s", "se", "ses", "sess"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration and a session against HSM group or xnames")
        .arg(arg!(-n --name <VALUE> "Session name").required(true))
        .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image")
            .required(true))
        // .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
        .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS session. NOTE: ansible-limit must be a subset of hsm-group if both parameters are provided"))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to aee container running ansible scripts"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4", "5"])
            .num_args(1)
            .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .group(ArgGroup::new("hsm-group_or_ansible-limit").args(["hsm-group", "ansible-limit"]).multiple(true));

    match hsm_group {
        Some(_) => {}
        None => apply_session = apply_session.arg(arg!(-H --"hsm-group" <VALUE> "hsm group name")),
    };

    apply_session
}

pub fn subcommand_apply_node_on(hsm_group: Option<&String>) -> Command {
    let mut apply_node_on = Command::new("on")
        .about("Start a node")
        .arg_required_else_help(true)
        .arg(arg!(<XNAMES> "node's xnames"))
        .arg(arg!(-r --reason <VALUE> "reason to power on"));

    match hsm_group {
        None => {
            apply_node_on = apply_node_on
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(
                    ArgGroup::new("hsm-group_or_xnames")
                        .args(["hsm-group", "XNAMES"])
                        .multiple(true),
                )
        }
        Some(_) => {}
    };

    apply_node_on
}

pub fn subcommand_apply_node_off(hsm_group: Option<&String>) -> Command {
    let mut apply_node_off = Command::new("off")
        .arg_required_else_help(true)
        .about("Shutdown a node")
        .arg(arg!(<XNAMES> "node's xnames"))
        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
        .arg(arg!(-r --reason <VALUE> "reason to power off"));

    match hsm_group {
        None => {
            apply_node_off = apply_node_off
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(
                    ArgGroup::new("hsm-group_or_xnames")
                        .args(["hsm-group", "XNAMES"])
                        .multiple(true),
                )
        }
        Some(_) => {}
    };

    apply_node_off
}

pub fn subcommand_apply_node_reset(hsm_group: Option<&String>) -> Command {
    let mut apply_node_reset = Command::new("reset")
        .aliases(["r", "res"])
        .arg_required_else_help(true)
        .about("Restart a node")
        .arg(arg!(<XNAMES> "node's xnames"))
        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
        .arg(arg!(-r --reason <VALUE> "reason to reset"));

    match hsm_group {
        None => {
            apply_node_reset = apply_node_reset
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(
                    ArgGroup::new("hsm-group_or_xnames")
                        .args(["hsm-group", "XNAMES"])
                        .multiple(true),
                )
        }
        Some(_) => {}
    };

    apply_node_reset
}

pub fn get_matches(hsm_group: Option<&String>) -> ArgMatches {
    command!()
        .arg_required_else_help(true)
        .subcommand(subcommand_get(hsm_group))
        .subcommand(
            Command::new("apply")
                .alias("a")
                .arg_required_else_help(true)
                .about("Make changes to Shasta HSM group or nodes")
                .subcommand(subcommand_apply_session(hsm_group))
                .subcommand(
                    Command::new("node")
                        .aliases(["n", "nod"])
                        .arg_required_else_help(true)
                        .about("Make changes to nodes")
                        .subcommand(subcommand_apply_node_on(hsm_group))
                        .subcommand(subcommand_apply_node_off(hsm_group))
                        .subcommand(subcommand_apply_node_reset(hsm_group)),
                ),
        )
        .subcommand(
            Command::new("log")
                .alias("l")
                .arg_required_else_help(true)
                .about("Get CFS session logs")
                .arg(arg!(<SESSION> "session name"))
                .arg(
                    arg!(-l --"layer-id" <VALUE> "layer id")
                        .required(false)
                        .value_parser(value_parser!(u8)),
                ),
        )
        .subcommand(
            Command::new("console")
                .aliases(["c", "con", "cons", "conso"])
                .arg_required_else_help(true)
                .about("Access node's console")
                .arg(arg!(<XNAME> "node xname")),
        )
        .get_matches()
}

pub async fn process_command(
    cli_root: ArgMatches,
    shasta_token: String,
    shasta_base_url: String,
    vault_base_url: String,
    gitea_token: &str,
    gitea_base_url: &str,
    hsm_group: Option<&String>,
) -> core::result::Result<(), Box<dyn std::error::Error>> {
    let hsm_group_name;
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

            hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_configuration.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };

            most_recent = cli_get_configuration.get_one::<bool>("most-recent");

            if let Some(true) = most_recent {
                limit_number = Some(&1);
            } else if let Some(false) = most_recent {
                limit_number = cli_get_configuration.get_one::<u8>("limit");
            } else {
                limit_number = None;
            }

            // Get CFS configurations
            let cfs_configurations = shasta_cfs_configuration::http_client::get(
                &shasta_token,
                &shasta_base_url,
                hsm_group_name,
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
                        gitea_token,
                    )
                    .await?;

                    layers.push(manta_cfs_configuration::Layer::new(
                        layer["name"].as_str().unwrap(),
                        layer["cloneUrl"]
                            .as_str()
                            .unwrap()
                            .trim_start_matches("https://api.cmn.alps.cscs.ch")
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

            hsm_group_name = match hsm_group {
                // ref: https://stackoverflow.com/a/32487173/1918003
                None => cli_get_session.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };

            most_recent = cli_get_session.get_one::<bool>("most-recent");

            if let Some(true) = most_recent {
                limit_number = Some(&1);
            } else if let Some(false) = most_recent {
                limit_number = cli_get_session.get_one::<u8>("limit");
            } else {
                limit_number = None;
            }

            let cfs_sessions = shasta_cfs_session::http_client::get(
                &shasta_token,
                &shasta_base_url,
                hsm_group_name,
                session_name,
                limit_number,
                None,
            )
            .await?;

            if cfs_sessions.is_empty() {
                println!("No CFS session found!");
                return Ok(());
            } else {
                shasta_cfs_session::utils::print_table(cfs_sessions);
            }
        } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
            template_name = cli_get_template.get_one::<String>("name");

            hsm_group_name = match hsm_group {
                None => cli_get_template.get_one::<String>("hsm-group"),
                Some(hsm_group_val) => Some(hsm_group_val),
            };

            most_recent = cli_get_template.get_one::<bool>("most-recent");

            if let Some(true) = most_recent {
                limit_number = Some(&1);
            } else if let Some(false) = most_recent {
                limit_number = cli_get_template.get_one::<u8>("limit");
            } else {
                limit_number = None;
            }

            let bos_templates = bos_template::http_client::get(
                &shasta_token,
                &shasta_base_url,
                hsm_group_name,
                template_name,
                limit_number,
            )
            .await?;

            if bos_templates.is_empty() {
                println!("No BOS template found!");
                return Ok(());
            } else {
                bos_template::utils::print_table(bos_templates);
            }
        } else if let Some(cli_get_node) = cli_get.subcommand_matches("node") {
            // Check of HSM group name provided y configuration file
            hsm_group_name = match hsm_group {
                None => cli_get_node.get_one::<String>("HSMGROUP"),
                Some(_) => hsm_group,
            };

            let hsm_group_resp = crate::shasta::hsm::http_client::get_hsm_group(
                &shasta_token,
                &shasta_base_url,
                hsm_group_name.unwrap(),
            )
            .await;

            // println!("hsm_groups: {:?}", hsm_groups);

            let hsm_group;

            // Exit if no hsm groups found
            if hsm_group_resp.is_err() {
                println!("No nodes found!");
                return Ok(());
            } else {
                hsm_group = hsm_group_resp.unwrap();
            }

            // Take all nodes for all hsm_groups found and put them in a Vec
            let hsm_groups_nodes: Vec<String> = hsm_group["members"]["ids"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|xname| xname.as_str().unwrap().to_string())
                .collect();

            // Get node most recent CFS session with target image
            // Get all CFS sessions matching hsm_group value
            let mut cfs_sessions = crate::shasta::cfs::session::http_client::get(
                &shasta_token,
                &shasta_base_url,
                hsm_group_name,
                None,
                None,
                Some(true),
            )
            .await
            .unwrap();

            // Sort CFS sessions by start time
            cfs_sessions.sort_by(|a, b| {
                a["status"]["session"]["completionTime"]
                    .as_str()
                    .unwrap()
                    .cmp(b["status"]["session"]["startTime"].as_str().unwrap())
            });

            // println!("cfs_sessions: {:#?}", cfs_sessions);

            // Filter CFS sessions with target = "image" and succeded = "true"
            let cfs_sessions_target_image: Vec<_> = cfs_sessions
                .iter()
                .filter(|cfs_session| {
                    cfs_session["target"]["definition"]
                        .as_str()
                        .unwrap()
                        .eq("image")
                        && cfs_session["status"]["session"]["succeeded"]
                            .as_str()
                            .unwrap()
                            .eq("true")
                })
                .collect();

            // println!("cfs_sessions_target_image: {:#?}", cfs_sessions_target_image);

            // Get most recent CFS session with target = "image" and succeded = "true"
            let cfs_session_most_recent = cfs_sessions_target_image
                [cfs_sessions_target_image.len().saturating_sub(1)..]
                .to_vec();

            // Exit if no CFS session found!
            if cfs_session_most_recent.is_empty() {
                println!("CFS session target image not found!");
                std::process::exit(0);
            }

            // Extract image id from session
            let image_id = cfs_session_most_recent.iter().next().unwrap()["status"]["artifacts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|artifact| artifact["image_id"].as_str().unwrap())
                .next()
                .unwrap()
                .to_string();

            log::info!(
                "image_id from most recent CFS session (target = image and successful = true): {}",
                image_id
            );

            // println!("Images found in CFS sessions for HSM group {}:", hsm_group_name.unwrap());
            //            for cfs_session_target_image in cfs_sessions_target_image {
            //                println!(
            //                    "start time: {}; image_id: {}",
            //                    cfs_session_target_image["status"]["session"]["startTime"],
            //                    cfs_session_target_image["status"]["artifacts"]
            //                        .as_array()
            //                        .unwrap()
            //                        .iter()
            //                        .next()
            //                        .unwrap()["image_id"]
            //                        .as_str()
            //                        .unwrap()
            //                );
            //            }

            // Check nodes have latest image in bootparam
            let nodes_status = check_nodes_have_most_recent_image::get_node_details(
                &shasta_token,
                &shasta_base_url,
                &hsm_group_name.unwrap(),
                &hsm_groups_nodes,
            )
            .await;

            // shasta::hsm::utils::print_table(hsm_groups);
            node_ops::print_table(nodes_status);
        } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
            let hsm_group_name = match hsm_group {
                None => cli_get_hsm_groups.get_one::<String>("HSMGROUP").unwrap(),
                Some(hsm_group_name_value) => hsm_group_name_value,
            };

            let hsm_groups =
                cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group_name).await;

            for hsm_group in hsm_groups {
                println!("************************* HSM GROUP *************************");

                println!(" * HSM group label: {}", hsm_group.hsm_group_label);
                println!(" * CFS configuration details:");
                println!(
                    "   - name: {}",
                    hsm_group.most_recent_cfs_configuration_name_created["name"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "   - last updated: {}",
                    hsm_group.most_recent_cfs_configuration_name_created["lastUpdated"]
                        .as_str()
                        .unwrap_or_default()
                );

                for (i, layer) in hsm_group.most_recent_cfs_configuration_name_created["layers"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
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
                }

                println!(" * CFS session details:");
                println!(
                    "   - Name: {}",
                    hsm_group.most_recent_cfs_session_name_created["name"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "   - Configuration name: {}",
                    hsm_group.most_recent_cfs_session_name_created["configuration"]["name"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "   - Target: {}",
                    hsm_group.most_recent_cfs_session_name_created["target"]["definition"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!("   + Ansible details:");
                println!(
                    "     - name: {}",
                    hsm_group.most_recent_cfs_session_name_created["ansible"]["config"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - limit: {}",
                    hsm_group.most_recent_cfs_session_name_created["ansible"]["limit"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!("   + Status:");
                println!(
                    "     - status: {}",
                    hsm_group.most_recent_cfs_session_name_created["status"]["session"]["status"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - succeeded: {}",
                    hsm_group.most_recent_cfs_session_name_created["status"]["session"]
                        ["succeeded"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - job: {}",
                    hsm_group.most_recent_cfs_session_name_created["status"]["session"]["job"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "     - start: {}",
                    hsm_group.most_recent_cfs_session_name_created["status"]["session"]
                        ["startTime"]
                        .as_str()
                        .unwrap_or_default()
                );
                println!(
                    "   - tags: {}",
                    hsm_group.most_recent_cfs_session_name_created["tags"]
                );

                println!(
                    " * members: {}",
                    nodes::nodes_to_string_format_unlimited(&hsm_group.members)
                );
            }
        }
    } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
        if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
            let included: HashSet<String>;
            let excluded: HashSet<String>;
            // Check andible limit matches the nodes in hsm_group
            let hsm_groups;

            let cfs_configuration_name;

            let hsm_groups_nodes;

            // * Validate input params
            // Neither hsm_group (both config file or cli arg) nor ansible_limit provided --> ERROR since we don't know the target nodes to apply the session to
            // NOTE: hsm group can be assigned either by config file or cli arg
            if cli_apply_session
                .get_one::<String>("ansible-limit")
                .is_none()
                && hsm_group.is_none()
                && cli_apply_session.get_one::<String>("hsm-group").is_none()
            {
                // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
                eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
                std::process::exit(-1);
            }
            // * End validation input params

            // * Parse input params
            // Parse ansible limit
            // Get ansible limit nodes from cli arg
            let ansible_limit_nodes: HashSet<String> = if cli_apply_session
                .get_one::<String>("ansible-limit")
                .is_some()
            {
                // Get HashSet with all nodes from ansible-limit param
                cli_apply_session
                    .get_one::<String>("ansible-limit")
                    .unwrap()
                    .replace(' ', "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect()
            } else {
                HashSet::new()
            };

            // Parse hsm group
            let mut hsm_group_value = None;

            // Get hsm_group from cli arg
            if cli_apply_session.get_one::<String>("hsm-group").is_some() {
                hsm_group_value = cli_apply_session.get_one::<String>("hsm-group");
            }

            // Get hsm group from config file
            if hsm_group.is_some() {
                hsm_group_value = hsm_group;
            }
            // * End Parse input params

            // * Process/validate hsm group value (and ansible limit)
            if hsm_group_value.is_some() {
                // Get all hsm groups related to hsm_group input
                hsm_groups = cluster_ops::get_details(
                    &shasta_token,
                    &shasta_base_url,
                    hsm_group_value.unwrap(),
                )
                .await;

                //cfs_configuration_name = format!("{}-{}", hsm_group_value.unwrap(), cli_apply_session.get_one::<String>("name").unwrap());
                cfs_configuration_name = cli_apply_session
                    .get_one::<String>("name")
                    .unwrap()
                    .to_string();

                // Take all nodes for all hsm_groups found and put them in a Vec
                hsm_groups_nodes = hsm_groups
                    .iter()
                    .flat_map(|hsm_group| {
                        hsm_group
                            .members
                            .iter()
                            .map(|xname| xname.as_str().unwrap().to_string())
                    })
                    .collect();

                if !ansible_limit_nodes.is_empty() {
                    // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

                    (included, excluded) = node_ops::check_hsm_group_and_ansible_limit(
                        &hsm_groups_nodes,
                        ansible_limit_nodes,
                    );

                    if !excluded.is_empty() {
                        println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                            excluded,
                            hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                            hsm_groups_nodes);
                        std::process::exit(-1);
                    }
                } else {
                    // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
                    included = hsm_groups_nodes
                }
            } else {
                // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
                cfs_configuration_name = cli_apply_session
                    .get_one::<String>("name")
                    .unwrap()
                    .to_string();
                included = ansible_limit_nodes
            }
            // * End Process/validate hsm group value (and ansible limit)

            // * Create CFS session
            let cfs_session_name = create_cfs_session_from_repo::run(
                &cfs_configuration_name,
                cli_apply_session
                    .get_many("repo-path")
                    .unwrap()
                    .cloned()
                    .collect(),
                // vec![cli_apply_session
                //     .get_one::<String>("repo-path")
                //     .unwrap()
                //     .to_string()],
                gitea_token,
                gitea_base_url,
                &shasta_token,
                &shasta_base_url,
                included.into_iter().collect::<Vec<String>>().join(","), // Convert Hashset to String with comma separator, need to convert to Vec first following https://stackoverflow.com/a/47582249/1918003
                cli_apply_session
                    .get_one::<String>("ansible-verbosity")
                    .unwrap()
                    .parse()
                    .unwrap(),
            )
            .await;

            let watch_logs = cli_apply_session.get_one::<bool>("watch-logs");

            if let Some(true) = watch_logs {
                log::info!("Fetching logs ...");
                shasta_cfs_session_logs::client::session_logs(
                    vault_base_url,
                    cfs_session_name.unwrap().as_str(),
                    None,
                )
                .await?;
            }
            // * End Create CFS session
        } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                let included: HashSet<String>;
                let excluded: HashSet<String>;
                // Check andible limit matches the nodes in hsm_group
                let hsm_groups;

                let hsm_groups_nodes;

                // * Validate input params
                // Neither hsm_group (both config file or cli arg) nor xnames provided --> ERROR since we don't know the target nodes to apply the session to
                // NOTE: hsm group can be assigned either by config file or cli arg
                if cli_apply_node_on.get_one::<String>("XNAMES").is_none()
                    && hsm_group.is_none()
                    && cli_apply_node_on.get_one::<String>("hsm-group").is_none()
                {
                    // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
                    eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
                    std::process::exit(-1);
                }
                // * End validation input params

                // * Parse input params
                // Parse xnames
                // Get xnames nodes from cli arg
                // User provided list of xnames to power on
                let xnames: HashSet<String> = cli_apply_node_on
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .replace(' ', "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect();

                // Parse hsm group
                let mut hsm_group_value = None;

                // Get hsm group from config file
                if hsm_group.is_some() {
                    hsm_group_value = hsm_group;
                }
                // * End Parse input params

                // * Process/validate hsm group value (and ansible limit)
                if hsm_group_value.is_some() {
                    // Get all hsm groups related to hsm_group input
                    hsm_groups = cluster_ops::get_details(
                        &shasta_token,
                        &shasta_base_url,
                        hsm_group_value.unwrap(),
                    )
                    .await;

                    // Take all nodes for all hsm_groups found and put them in a Vec
                    hsm_groups_nodes = hsm_groups
                        .iter()
                        .flat_map(|hsm_group| {
                            hsm_group
                                .members
                                .iter()
                                .map(|xname| xname.as_str().unwrap().to_string())
                        })
                        .collect();

                    if !xnames.is_empty() {
                        // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

                        (included, excluded) =
                            node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

                        if !excluded.is_empty() {
                            println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                                excluded,
                                hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                                hsm_groups_nodes);
                            std::process::exit(-1);
                        }
                    } else {
                        // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
                        included = hsm_groups_nodes
                    }
                } else {
                    // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
                    included = xnames
                }
                // * End Process/validate hsm group value (and ansible limit)

                log::info!("Servers to power on: {:?}", included);

                capmc::http_client::node_power_on::post(
                    shasta_token.to_string(),
                    shasta_base_url,
                    cli_apply_node_on.get_one::<String>("reason"),
                    included.into_iter().collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
                    false,
                )
                .await?;
            } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
                let included: HashSet<String>;
                let excluded: HashSet<String>;
                // Check andible limit matches the nodes in hsm_group
                let hsm_groups;

                let hsm_groups_nodes;

                // * Validate input params
                // Neither hsm_group (both config file or cli arg) nor xnames provided --> ERROR since we don't know the target nodes to apply the session to
                // NOTE: hsm group can be assigned either by config file or cli arg
                if cli_apply_node_off.get_one::<String>("XNAMES").is_none()
                    && hsm_group.is_none()
                    && cli_apply_node_off.get_one::<String>("hsm-group").is_none()
                {
                    // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
                    eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
                    std::process::exit(-1);
                }
                // * End validation input params

                // * Parse input params
                // Parse xnames
                // Get xnames nodes from cli arg
                // User provided list of xnames to power on
                let xnames: HashSet<String> = cli_apply_node_off
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .replace(' ', "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect();

                // Parse hsm group
                let mut hsm_group_value = None;

                // Get hsm group from config file
                if hsm_group.is_some() {
                    hsm_group_value = hsm_group;
                }
                // * End Parse input params

                // * Process/validate hsm group value (and ansible limit)
                if hsm_group_value.is_some() {
                    // Get all hsm groups related to hsm_group input
                    hsm_groups = cluster_ops::get_details(
                        &shasta_token,
                        &shasta_base_url,
                        hsm_group_value.unwrap(),
                    )
                    .await;

                    // Take all nodes for all hsm_groups found and put them in a Vec
                    hsm_groups_nodes = hsm_groups
                        .iter()
                        .flat_map(|hsm_group| {
                            hsm_group
                                .members
                                .iter()
                                .map(|xname| xname.as_str().unwrap().to_string())
                        })
                        .collect();

                    if !xnames.is_empty() {
                        // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

                        (included, excluded) =
                            node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

                        if !excluded.is_empty() {
                            println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                                excluded,
                                hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                                hsm_groups_nodes);
                            std::process::exit(-1);
                        }
                    } else {
                        // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
                        included = hsm_groups_nodes
                    }
                } else {
                    // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
                    included = xnames
                }
                // * End Process/validate hsm group value (and ansible limit)

                log::info!("Servers to power off: {:?}", included);

                capmc::http_client::node_power_off::post(
                    shasta_token.to_string(),
                    shasta_base_url,
                    cli_apply_node_off.get_one::<String>("reason"),
                    included.into_iter().collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
                    *cli_apply_node_off.get_one::<bool>("force").unwrap(),
                )
                .await?;
            } else if let Some(cli_apply_node_reset) = cli_apply_node.subcommand_matches("reset") {
                let included: HashSet<String>;
                let excluded: HashSet<String>;
                // Check andible limit matches the nodes in hsm_group
                let hsm_groups;

                let hsm_groups_nodes;

                // * Validate input params
                // Neither hsm_group (both config file or cli arg) nor xnames provided --> ERROR since we don't know the target nodes to apply the session to
                // NOTE: hsm group can be assigned either by config file or cli arg
                if cli_apply_node_reset.get_one::<String>("XNAMES").is_none()
                    && hsm_group.is_none()
                    && cli_apply_node_reset
                        .get_one::<String>("hsm-group")
                        .is_none()
                {
                    // TODO: move this logic to clap in order to manage error messages consistently??? can/should I??? Maybe I should look for input params in the config file if not provided by user???
                    eprintln!("Need to specify either ansible-limit or hsm-group or both. (hsm-group value can be provided by cli param or in config file)");
                    std::process::exit(-1);
                }
                // * End validation input params

                // * Parse input params
                // Parse xnames
                // Get xnames nodes from cli arg
                // User provided list of xnames to power on
                let xnames: HashSet<String> = cli_apply_node_reset
                    .get_one::<String>("XNAMES")
                    .unwrap()
                    .replace(' ', "") // trim xnames by removing white spaces
                    .split(',')
                    .map(|xname| xname.to_string())
                    .collect();

                // Parse hsm group
                let mut hsm_group_value = None;

                // Get hsm group from config file
                if hsm_group.is_some() {
                    hsm_group_value = hsm_group;
                }
                // * End Parse input params

                // * Process/validate hsm group value (and ansible limit)
                if hsm_group_value.is_some() {
                    // Get all hsm groups related to hsm_group input
                    hsm_groups = cluster_ops::get_details(
                        &shasta_token,
                        &shasta_base_url,
                        hsm_group_value.unwrap(),
                    )
                    .await;

                    // Take all nodes for all hsm_groups found and put them in a Vec
                    hsm_groups_nodes = hsm_groups
                        .iter()
                        .flat_map(|hsm_group| {
                            hsm_group
                                .members
                                .iter()
                                .map(|xname| xname.as_str().unwrap().to_string())
                        })
                        .collect();

                    if !xnames.is_empty() {
                        // both hsm_group provided and ansible_limit provided --> check ansible_limit belongs to hsm_group

                        (included, excluded) =
                            node_ops::check_hsm_group_and_ansible_limit(&hsm_groups_nodes, xnames);

                        if !excluded.is_empty() {
                            println!("Nodes in ansible-limit outside hsm groups members.\nNodes {:?} may be mistaken as they don't belong to hsm groups {:?} - {:?}", 
                                excluded,
                                hsm_groups.iter().map(|hsm_group| hsm_group.hsm_group_label.clone()).collect::<Vec<String>>(),
                                hsm_groups_nodes);
                            std::process::exit(-1);
                        }
                    } else {
                        // hsm_group provided but no ansible_limit provided --> target nodes are the ones from hsm_group
                        included = hsm_groups_nodes
                    }
                } else {
                    // no hsm_group provided but ansible_limit provided --> target nodes are the ones from ansible_limit
                    included = xnames
                }
                // * End Process/validate hsm group value (and ansible limit)

                log::info!("Servers to power reset: {:?}", included);

                capmc::http_client::node_power_restart::post(
                    shasta_token.to_string(),
                    shasta_base_url,
                    cli_apply_node_reset.get_one::<String>("reason"),
                    included.into_iter().collect(), // TODO: fix this HashSet --> Vec conversion. May need to specify lifespan for capmc struct
                    *cli_apply_node_reset.get_one::<bool>("force").unwrap(),
                )
                .await?; // TODO: idk why power on does not seems to work when forced
            }
        }
    } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
        logging_session_name = cli_log.get_one::<String>("SESSION");

        layer_id = cli_log.get_one::<u8>("layer-id");

        shasta_cfs_session_logs::client::session_logs_proxy(
            &shasta_token,
            &shasta_base_url,
            vault_base_url,
            None,
            logging_session_name,
            layer_id,
        )
        .await?;
    } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
        let included: HashSet<String>;
        let excluded: HashSet<String>;

        // User provided list of xnames to power reset
        let xnames: HashSet<String> = cli_console
            .get_one::<String>("XNAME")
            .unwrap()
            .replace(' ', "") // trim xnames by removing white spaces
            .split(',')
            .map(|xname| xname.to_string())
            .collect();

        let hsm_groups: Vec<ClusterDetails>;

        if hsm_group.is_some() {
            // hsm_group value provided
            hsm_groups =
                cluster_ops::get_details(&shasta_token, &shasta_base_url, hsm_group.unwrap()).await;

            // Take all nodes for all hsm_groups found and put them in a Set
            let hsm_groups_nodes = hsm_groups
                .iter()
                .flat_map(|hsm_group| {
                    hsm_group
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
        } else {
            // no hsm_group value provided
            included = xnames.clone();
        }

        connect_to_console(included.iter().next().unwrap(), vault_base_url).await?;
    }

    Ok(())
}
