use clap::{arg, value_parser, ArgAction, ArgGroup, Command};

use std::path::PathBuf;

pub fn build_cli(hsm_group: Option<&String>, hsm_available_vec: Vec<String>) -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .term_width(100)
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommand(subcommand_get(hsm_group))
        .subcommand(
            Command::new("apply")
                .alias("a")
                .arg_required_else_help(true)
                .about("Make changes to Shasta system")
                // .subcommand(subcommand_apply_configuration(hsm_group))
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
                )
                .subcommand(subcommand_apply_session(hsm_group))
                .subcommand(Command::new("ephemeral-environment")
                .aliases(["ee", "eph", "ephemeral"])
                .arg_required_else_help(true)
                .about("Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.")
                // .arg(arg!(-b --block "Blocks this operation and won't return prompt until the ephemeral environment has been created."))
                // .arg(arg!(-p --"public-ssh-key-id" <PUBLIC_SSH_ID> "Public ssh key id stored in Alps"))
                .arg(arg!(-i --"image-id" <IMAGE_ID> "Image ID to use as a container image").required(true))
                            ),
        )
        .subcommand(
            Command::new("update")
                .alias("u")
                .arg_required_else_help(true)
                .about("Update nodes power status or boot params")
                .subcommand(subcommand_update_nodes(hsm_group))
                .subcommand(subcommand_update_hsm_group(hsm_group)),
        )
        .subcommand(
            Command::new("log")
                .alias("l")
                .arg_required_else_help(true)
                .about("Get CFS session logs")
                .arg(arg!(<SESSION_NAME> "session name"))
                // .arg(
                //     arg!(-l --"layer-id" <VALUE> "layer id")
                //         .required(false)
                //         .value_parser(value_parser!(u8)),
                // ),
        )
        .subcommand(
            Command::new("console")
                .aliases(["c", "con", "cons", "conso"])
                .arg_required_else_help(true)
                .about("Opens an interective session to a node or CFS session ansible target container")
                .subcommand(
                    Command::new("node")
                        .alias("n")
                        .about("Connects to a node's console")
                        .arg(arg!(<XNAME> "node xname").required(true)),
                )
                .subcommand(
                    Command::new("target-ansible")
                        .aliases(["t", "ta", "target", "ansible"])
                        .arg_required_else_help(true)
                        .about("Opens an interactive session to the ansible target container of a CFS session")
                        .arg(arg!(<SESSION_NAME> "CFS session name").required(true)),
                ),
        )
        .subcommand(subcommand_delete(hsm_group))
        .subcommand(subcommand_config(hsm_available_vec))
}

pub fn subcommand_config(hsm_available_opt: Vec<String>) -> Command {
    // Enforce user to chose a HSM group is hsm_available config param is not empty. This is to
    // make sure tenants like PSI won't unset parameter hsm_group and take over all HSM groups.
    // NOTE: by default 'manta config set hsm' will unset the hsm_group config value and the user
    // will be able to access any HSM. The security meassures for this is to setup sticky bit to
    // manta binary so it runs as manta user, then 'chown manta:manta /home/manta/.config/manta/config.toml' so only manta and root users can edit the config file. Tenants can neither su to manta nor root under the access VM (eg castaneda)
    let subcommand_config_set_hsm = if !hsm_available_opt.is_empty() {
        Command::new("hsm")
            .about("Change config values")
            .about("Set target HSM group")
            .arg(arg!(<HSM_GROUP_NAME> "hsm group name"))
    } else {
        Command::new("hsm")
            .about("Change config values")
            .about("Set target HSM group")
            .arg(arg!([HSM_GROUP_NAME] "hsm group name"))
    };

    Command::new("config")
        .alias("C")
        .arg_required_else_help(true)
        .about("Manta's configuration")
        .subcommand(Command::new("show").about("Show config values"))
        .subcommand(
            Command::new("set")
                .arg_required_else_help(true)
                .about("Change config values")
                .subcommand(subcommand_config_set_hsm),
        )
}

pub fn subcommand_delete(hsm_group: Option<&String>) -> Command {
    let mut delete = Command::new("delete")
                .arg_required_else_help(true)
                .about("Deletes CFS configurations, CFS sessions, BOS sessiontemplates, BOS sessions and images related to CFS configuration/s.")
                .arg(arg!(-n --"configuration-name" <CONFIGURATION> "CFS configuration, CFS sessions, BOS sessiontemplate, BOS sessions and images related to the CFS configuration will be deleted.\neg:\nmanta delete --configuration-name my-config-v1.0\nDeletes all data related to CFS configuration with name 'my-config-v0.1'"))
                .arg(arg!(-s --since <DATE> "Deletes CFS configurations, CFS sessions, BOS sessiontemplate, BOS sessions and images related to CFS configurations with 'last updated' after since date. Note: date format is %Y-%m-%d\neg:\nmanta delete --since 2023-01-01 --until 2023-10-01\nDeletes all data related to CFS configurations created or updated between 01/01/2023T00:00:00Z and 01/10/2023T00:00:00Z"))
                .arg(arg!(-u --until <DATE> "Deletes CFS configuration, CFS sessions, BOS sessiontemplate, BOS sessions and images related to the CFS configuration with 'last updated' before until date. Note: date format is %Y-%m-%d\neg:\nmanta delete --until 2023-10-01\nDeletes all data related to CFS configurations created or updated before 01/10/2023T00:00:00Z"))
                .group(ArgGroup::new("since_and_until").args(["since", "until"]).multiple(true).requires("until").conflicts_with("configuration-name"));

    match hsm_group {
        None => {
            delete =
                delete.arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name").required(true))
        }
        Some(_) => {}
    }

    delete
}

pub fn subcommand_get_cfs_configuration() -> Command {
    let mut get_cfs_configuration = Command::new("configuration")
        .aliases(["c", "cfg", "conf", "config", "cnfgrtn"])
        .about("Get information from Shasta CFS configuration")
        .arg(arg!(-n --name <CONFIGURATION_NAME> "configuration name"))
        .arg(arg!(-m --"most-recent" "Only shows the most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of CFS configurations created")
                .value_parser(value_parser!(u8).range(1..)),
        );

    /* match hsm_group {
        None => {
            get_cfs_configuration = get_cfs_configuration
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-n --name <SESSION_NAME> "session name"))
        .arg(arg!(-m --"most-recent" "Only shows the most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of CFS sessions created")
                .value_parser(value_parser!(u8).range(1..)),
        )
        .arg(arg!(-o --output <FORMAT> "Output format. If missing, it will print output data in human redeable (tabular) format").value_parser(["json"]));

    match hsm_group {
        None => {
            get_cfs_session =
                get_cfs_session.arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-n --name <TEMPLATE_NAME> "template name"))
        .arg(arg!(-m --"most-recent" "Only shows the most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of BOS templates created")
                .value_parser(value_parser!(u8).range(1..)),
        );

    match hsm_group {
        None => {
            get_bos_template = get_bos_template
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-n --"nids-only-one-line" "Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,..."))
        .arg(arg!(-x --"xnames-only-one-line" "Prints xnames in one line eg x1001c1s5b0n0,x1001c1s5b0n1,..."))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]));

    match hsm_group {
        None => {
            get_node = get_node
                .arg_required_else_help(true)
                .arg(arg!(<HSM_GROUP_NAME> "hsm group name"))
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
                .arg(arg!(<HSM_GROUP_NAME> "hsm group name"))
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
        .about("Get image information")
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of images created")
                .value_parser(value_parser!(u8).range(1..)),
        );

    match hsm_group {
        None => {
            get_cfs_session =
                get_cfs_session.arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-f --file <SAT_FILE> "SAT file with configuration details").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-n --name <VALUE> "Configuration name"))
        .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").value_parser(value_parser!(PathBuf)))
        .group(ArgGroup::new("req_flags_name_repo-path").args(["name", "repo-path"]))
        // .group(ArgGroup::new("req_flag_file").arg("file"))
        ;

    match hsm_group {
        Some(_) => {}
        None => {
            apply_configuration =
                apply_configuration.arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
        }
    };

    apply_configuration
} */

pub fn subcommand_apply_session(hsm_group: Option<&String>) -> Command {
    let mut apply_session = Command::new("session")
        .aliases(["s", "se", "ses", "sess", "sssn"])
        .arg_required_else_help(true)
        .about("Runs the ansible script in local directory against HSM group or xnames.\nNote: the local repo must alrady exists in Shasta VCS")
        .arg(arg!(-n --name <VALUE> "Session name").required(true))
        // .arg(arg!(-i --image "If set, creates a CFS sesison of target image, otherwise it will create a CFS session target dynamic").action(ArgAction::SetTrue))
        .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").required(true)
            .value_parser(value_parser!(PathBuf)))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["0", "1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-p --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."));

    apply_session = match hsm_group {
        Some(_) => {
            apply_session
                .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS session. Note: ansible-limit must be a subset of hsm-group if both parameters are provided").required(true))
        }
        None => {
            apply_session
                .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS session. Note: ansible-limit must be a subset of hsm-group if both parameters are provided"))
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-f --file <SAT_FILE> "SAT file with the CFS configuration and CFS image details").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        /* .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image")
           .value_parser(value_parser!(PathBuf))) */
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-p --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]))
}

pub fn subcommand_apply_cluster(/* hsm_group: Option<&String> */) -> Command {
    Command::new("cluster")
        .aliases(["clus","clstr"])
        .arg_required_else_help(true)
        .about("Create a CFS configuration, a CFS image, a BOS sessiontemplate and a BOS session")
        .arg(arg!(-f --file <SAT_FILE> "SAT file with CFS configuration, CFS image and BOS session template details").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-p --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]))
}

pub fn subcommand_apply_node_on(hsm_group: Option<&String>) -> Command {
    let mut apply_node_on = Command::new("on")
        .about("Start nodes")
        .arg_required_else_help(true)
        .arg(arg!(<XNAMES> "nodes' xnames"))
        .arg(arg!(-r --reason <TEXT> "reason to power on"));

    match hsm_group {
        None => {
            apply_node_on = apply_node_on
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-r --reason <TEXT> "reason to power off"));

    match hsm_group {
        None => {
            apply_node_off = apply_node_off
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
        .arg(arg!(-r --reason <TEXT> "reason to reset"));

    match hsm_group {
        None => {
            apply_node_reset = apply_node_reset
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
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
    let mut update_nodes = Command::new("nodes")
        .aliases(["n", "node", "nd"])
        .arg_required_else_help(true)
        .about("Updates boot and configuration of a group of nodes. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"boot-image" <CFS_CONFIG> "CFS configuration name related to the image to boot the nodes"))
        .arg(arg!(-d --"desired-configuration" <CFS_CONFIG> "CFS configuration name to configure the nodes after booting"));

    update_nodes = update_nodes
        .arg(arg!(<XNAMES> "Comma separated list of xnames which boot image will be updated"));

    /* update_nodes = update_nodes
    .arg(arg!(<CFS_CONFIG> "CFS configuration name used to boot and configure the nodes")); */

    update_nodes = match hsm_group {
        Some(_) => update_nodes,
        None => update_nodes.arg(arg!([HSM_GROUP_NAME] "hsm group name, this field should be used to validate the XNAMES belongs to HSM_GROUP_NAME")),
    };

    /* update_nodes = update_nodes.group(
        ArgGroup::new("boot-image_or_desired-configuration")
            .args(["boot-image", "desired-configuration"]),
    ); */

    /* update_nodes = update_nodes.groups([
        ArgGroup::new("update-node-boot_or_update-node-desired-configuration")
            .args(["boot", "desired-configuration"]),
        ArgGroup::new("update-node-args").args(["XNAMES", "CFS_CONFIG"]),
    ]); */

    /* update_node = update_node
        .group(
            ArgGroup::new("boot_and_config")
                .args(["boot-image", "configuration"])
                .required(true),
        )
        .group(ArgGroup::new("config").args(["CFS_CONFIG"]));

    update_node = update_node.group(ArgGroup::new("boot-config_or_config").args(["boot_and_config", "config"])); */

    update_nodes
}

pub fn subcommand_update_hsm_group(hsm_group: Option<&String>) -> Command {
    let mut update_hsm_group = Command::new("hsm-group")
        .aliases(["h", "hsm"])
        .arg_required_else_help(true)
        .about("Updates boot and configuration of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"boot-image" <CFS_CONFIG> "CFS configuration name related to the image to boot the nodes"))
        .arg(arg!(-d --"desired-configuration" <CFS_CONFIG> "CFS configuration name to configure the nodes after booting"));

    update_hsm_group = match hsm_group {
        Some(_) => update_hsm_group,
        None => update_hsm_group.arg(arg!(<HSM_GROUP_NAME> "HSM group name").required(true)),
    };

    update_hsm_group
}
