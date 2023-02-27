use clap::{arg, command, value_parser, ArgAction, ArgGroup, ArgMatches, Command};

use std::path::PathBuf;

use crate::cli::commands::{
    apply_configuration, apply_node_off, apply_node_on, apply_node_reset, apply_session, console,
    get_configuration, get_hsm, get_nodes, get_session, get_template, log,
};

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
    let mut get_node = Command::new("nodes")
        .aliases(["n", "node"])
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

pub fn subcommand_apply_configuration(hsm_group: Option<&String>) -> Command {
    let mut apply_configuration = Command::new("configuration")
        .aliases(["c", "config", "configure"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration against a HSM group")
        .arg(arg!(-f --file <VALUE> "Sat file with configuration details").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-n --name <VALUE> "Configuration name"))
        .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").value_parser(value_parser!(PathBuf)))
        .group(ArgGroup::new("req_flags_name_repo-path").arg("name").arg("repo-path"))
        .group(ArgGroup::new("req_flag_file").arg("file"));

    match hsm_group {
        Some(_) => {}
        None => {
            apply_configuration =
                apply_configuration.arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
        }
    };

    apply_configuration
}

pub fn subcommand_apply_session(hsm_group: Option<&String>) -> Command {
    let mut apply_session = Command::new("session")
        .aliases(["s", "se", "ses", "sess"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration and a session against HSM group or xnames")
        .arg(arg!(-n --name <VALUE> "Session name").required(true))
        .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image")
            .required(true).value_parser(value_parser!(PathBuf)))
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
                .subcommand(subcommand_apply_configuration(hsm_group))
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
    if let Some(cli_get) = cli_root.subcommand_matches("get") {
        if let Some(cli_get_configuration) = cli_get.subcommand_matches("configuration") {
            get_configuration::exec(
                gitea_token,
                hsm_group,
                cli_get_configuration,
                &shasta_token,
                &shasta_base_url,
            )
            .await;
        } else if let Some(cli_get_session) = cli_get.subcommand_matches("session") {
            get_session::exec(hsm_group, cli_get_session, &shasta_token, &shasta_base_url).await;
        } else if let Some(cli_get_template) = cli_get.subcommand_matches("template") {
            get_template::exec(hsm_group, cli_get_template, &shasta_token, &shasta_base_url).await;
        } else if let Some(cli_get_node) = cli_get.subcommand_matches("nodes") {
            get_nodes::exec(hsm_group, cli_get_node, &shasta_token, &shasta_base_url).await;
        } else if let Some(cli_get_hsm_groups) = cli_get.subcommand_matches("hsm-groups") {
            get_hsm::exec(
                hsm_group,
                cli_get_hsm_groups,
                &shasta_token,
                &shasta_base_url,
            )
            .await;
        }
    } else if let Some(cli_apply) = cli_root.subcommand_matches("apply") {
        if let Some(cli_apply_configuration) = cli_apply.subcommand_matches("configuration") {
            apply_configuration::exec(
                hsm_group,
                cli_apply_configuration,
                &shasta_token,
                &shasta_base_url,
                &gitea_token.to_string(),
                &gitea_base_url.to_string()
            )
            .await;
        } else if let Some(cli_apply_session) = cli_apply.subcommand_matches("session") {
            apply_session::exec(
                gitea_token,
                gitea_base_url,
                vault_base_url,
                hsm_group,
                cli_apply_session,
                &shasta_token,
                &shasta_base_url,
            )
            .await;
        } else if let Some(cli_apply_node) = cli_apply.subcommand_matches("node") {
            if let Some(cli_apply_node_on) = cli_apply_node.subcommand_matches("on") {
                apply_node_on::exec(hsm_group, cli_apply_node_on, shasta_token, shasta_base_url)
                    .await;
            } else if let Some(cli_apply_node_off) = cli_apply_node.subcommand_matches("off") {
                apply_node_off::exec(hsm_group, cli_apply_node_off, shasta_token, shasta_base_url)
                    .await;
            } else if let Some(cli_apply_node_reset) = cli_apply_node.subcommand_matches("reset") {
                apply_node_reset::exec(
                    hsm_group,
                    cli_apply_node_reset,
                    shasta_token,
                    shasta_base_url,
                )
                .await;
            }
        }
    } else if let Some(cli_log) = cli_root.subcommand_matches("log") {
        log::exec(cli_log, &shasta_token, &shasta_base_url, vault_base_url).await;
    } else if let Some(cli_console) = cli_root.subcommand_matches("console") {
        console::exec(
            hsm_group,
            cli_console,
            &shasta_token,
            &shasta_base_url,
            vault_base_url,
        )
        .await;
    }

    Ok(())
}
