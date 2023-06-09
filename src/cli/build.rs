use clap::{arg, value_parser, ArgAction, ArgGroup, Command};

use std::path::PathBuf;

pub fn build_cli(hsm_group: Option<&String>) -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommand(subcommand_get(hsm_group))
        .subcommand(
            Command::new("apply")
                .alias("a")
                .arg_required_else_help(true)
                .about("Make changes to Shasta HSM group or nodes")
                // .subcommand(subcommand_apply_configuration(hsm_group))
                .subcommand(subcommand_apply_session(hsm_group))
                .subcommand(subcommand_apply_image(/* hsm_group */))
                .subcommand(subcommand_apply_cluster(/* hsm_group */))
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
            Command::new("update")
                .alias("u")
                .arg_required_else_help(true)
                .about("Update nodes boot params")
                .subcommand(subcommand_update_nodes(hsm_group))
                .subcommand(subcommand_update_hsm_group(hsm_group)),
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
}

pub fn subcommand_get_cfs_configuration() -> Command {
    let mut get_cfs_configuration = Command::new("configuration")
        .aliases(["c", "cfg", "conf", "config", "cnfgrtn"])
        .about("Get information from Shasta CFS configuration")
        .arg(arg!(-n --name <VALUE> "configuration name"))
        .arg(arg!(-m --"most-recent" "most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "number of CFS configurations to show on screen")
                .value_parser(value_parser!(u8).range(1..)),
        );

    /* match hsm_group {
        None => {
            get_cfs_configuration = get_cfs_configuration
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_configuration").args(["hsm-group", "name"]))
        }
        Some(_) => {}
    } */

    get_cfs_configuration = get_cfs_configuration
        .group(ArgGroup::new("configuration_limit").args(["most-recent", "limit"]));

    get_cfs_configuration
}

pub fn subcommand_get_cfs_session(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_session = Command::new("session")
        .aliases(["s", "se", "ses", "sess", "sssn"])
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
        Some(_) => {}
    }

    get_cfs_session =
        get_cfs_session.group(ArgGroup::new("session_limit").args(["most-recent", "limit"]));

    get_cfs_session
}

pub fn subcommand_get_bos_template(hsm_group: Option<&String>) -> Command {
    let mut get_bos_template = Command::new("template")
        .aliases(["t", "tplt", "templ", "tmplt"])
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
        .aliases(["n", "node", "nd"])
        .about("Get members of a HSM group")
        .arg(arg!(-s --silent "Be more silent, produces less output by giving a list of nids like nidxxxxxx,nidyyyyyy,nidzzzzzz,..."))
        .arg(arg!(-x --"silent-xname" "Be more silent, produces less output by giving a list of xnames"));

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
        .aliases(["h", "hg", "hsm", "hsmgrps"])
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

pub fn subcommand_get_images(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_session = Command::new("images")
        .aliases(["i", "img", "imag", "image"])
        .about("Get image information");

    match hsm_group {
        None => {
            get_cfs_session = get_cfs_session.arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
        }
        Some(_) => {}
    }

    get_cfs_session
}

pub fn subcommand_get(hsm_group: Option<&String>) -> Command {
    Command::new("get")
        .alias("g")
        .arg_required_else_help(true)
        .about("Get information from Shasta system")
        .subcommand(subcommand_get_cfs_session(hsm_group))
        .subcommand(subcommand_get_cfs_configuration())
        .subcommand(subcommand_get_bos_template(hsm_group))
        .subcommand(subcommand_get_node(hsm_group))
        .subcommand(subcommand_get_hsm_groups_details(hsm_group))
        .subcommand(subcommand_get_images(hsm_group))
}

/* pub fn subcommand_apply_configuration(hsm_group: Option<&String>) -> Command {
    let mut apply_configuration = Command::new("configuration")
        .aliases(["c", "cfg", "conf", "config", "cnfgrtn"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration against a HSM group")
        .arg(arg!(-f --file <VALUE> "SAT file with configuration details").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-n --name <VALUE> "Configuration name"))
        .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").value_parser(value_parser!(PathBuf)))
        .group(ArgGroup::new("req_flags_name_repo-path").args(["name", "repo-path"]))
        // .group(ArgGroup::new("req_flag_file").arg("file"))
        ;

    match hsm_group {
        Some(_) => {}
        None => {
            apply_configuration =
                apply_configuration.arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
        }
    };

    apply_configuration
} */

pub fn subcommand_apply_session(hsm_group: Option<&String>) -> Command {
    let mut apply_session = Command::new("session")
        .aliases(["s", "se", "ses", "sess", "sssn"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration and a session against HSM group or xnames")
        .arg(arg!(-n --name <VALUE> "Session name").required(true))
        // .arg(arg!(-i --image "If set, creates a CFS sesison of target image, otherwise it will create a CFS session target dynamic").action(ArgAction::SetTrue))
        .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").required(true)
            .value_parser(value_parser!(PathBuf)))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["0", "1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"));

    apply_session = match hsm_group {
        Some(_) => {
            apply_session
                .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS session. NOTE: ansible-limit must be a subset of hsm-group if both parameters are provided").required(true))
        }
        None => {
            apply_session
                .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS session. NOTE: ansible-limit must be a subset of hsm-group if both parameters are provided"))
                .arg(arg!(-H --"hsm-group" <VALUE> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_ansible-limit").args(["hsm-group", "ansible-limit"]).required(true))
        }
    };

    apply_session
}

/// Creates an image based on a list of Ansible scripts (CFS layers) and assigns the image to a HSM
/// group.
/// Returns: the image id.
/// First creates a CFS configuration (configuration name is autogenerated). Then creates a CFS session
/// of 'target image' (session name is autogenerated).
pub fn subcommand_apply_image(/* hsm_group: Option<&String> */) -> Command {
    Command::new("image")
        .aliases(["i", "img", "imag"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration and a CFS image")
        .arg(arg!(-f --file <VALUE> "SAT file with the CFS configuration and CFS image details").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        /* .arg(arg!(-r --"repo-path" <VALUE> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image")
           .value_parser(value_parser!(PathBuf))) */
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
       .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
}

pub fn subcommand_apply_cluster(/* hsm_group: Option<&String> */) -> Command {
    Command::new("cluster")
        .aliases(["clus","clstr"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration, a CFS image, a BOS sessiontemplate and a BOS session")
        .arg(arg!(-f --file <VALUE> "SAT file with CFS configuration, CFS image and BOS session template details").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
}

pub fn subcommand_apply_node_on(hsm_group: Option<&String>) -> Command {
    let mut apply_node_on = Command::new("on")
        .about("Start nodes")
        .arg_required_else_help(true)
        .arg(arg!(<XNAMES> "nodes' xnames"))
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
        .about("Shutdown nodes")
        .arg(arg!(<XNAMES> "nodes' xnames"))
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
        .aliases(["r", "res", "rst", "restart", "rstrt"])
        .arg_required_else_help(true)
        .about("Restart nodes")
        .arg(arg!(<XNAMES> "nodes' xnames"))
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

pub fn subcommand_update_nodes(hsm_group: Option<&String>) -> Command {
    let mut update_node = Command::new("nodes")
        .aliases(["n", "node", "nd"])
        .arg_required_else_help(true)
        .about("Search for the most recent successful CFS session and assigns its image as a boot param and it's CFS configuration as BOS sessiontemplate for BOA to all the xnames provided");

    update_node = match hsm_group {
        Some(_) => update_node,
        None => update_node.arg(arg!(<HSM_GROUP> "hsm group name").required(true)),
    };

    update_node = update_node
        .arg(
            arg!(<XNAMES> "Comma separated list of xnames which boot image will be updated")
                .required(true),
        )
        .arg(
            arg!(<CFS_CONFIG> "CFS configuration name used to boot and configure the nodes")
                .required(true),
        );

    update_node
}

pub fn subcommand_update_hsm_group(hsm_group: Option<&String>) -> Command {
    let mut update_hsm_group = Command::new("hsm-group")
        .aliases(["h", "hsm"])
        .arg_required_else_help(true)
        .about("Search for the provided CFS session and assigns its image as a boot param and it's CFS configuration as BOS sessiontemplate for BOA to all the xnames linked to the HSM group provided");

    update_hsm_group = match hsm_group {
        Some(_) => update_hsm_group,
        None => update_hsm_group.arg(arg!(<HSM_GROUP> "HSM group name").required(true)),
    };

    update_hsm_group = update_hsm_group.arg(
        arg!([CFS_CONFIG] "CFS configuration name used to boot and configure the nodes")
            .required(true),
    );

    update_hsm_group
}
