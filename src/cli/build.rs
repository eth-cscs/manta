use clap::{arg, value_parser, ArgAction, ArgGroup, Command};
use mesa::hsm::hw_inventory::hw_component::r#struct::ArtifactType;
use strum::IntoEnumIterator;

use std::path::PathBuf;

pub fn build_cli(hsm_group: Option<&String>) -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .term_width(100)
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommand(subcommand_power())
        .subcommand(subcommand_get(hsm_group))
        .subcommand(Command::new("add")
            .arg_required_else_help(true)
            .about("WIP - Add hw components to cluster")
            .subcommand(Command::new("hw-component")
                .alias("hw")
                .arg_required_else_help(true)
                .about("WIP - Add hw components from a cluster")
                .arg(arg!(-P --pattern <PATTERN> "Pattern"))
                .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the pattern is applying to."))
                .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster."))
                .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
                .arg(arg!(-c --"create-hsm-group" "If the target cluster name does not exist as HSM group, create it."))

            )
            .subcommand(Command::new("nodes")
                .aliases(["n", "node"])
                .arg_required_else_help(true)
                .about("WIP - Add nodes to a cluster")
                .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the nodes are moving to."))
                .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one offering and receiving nodes from the target cluster."))
                .arg(arg!(<XNAMES> "Comma separated list of xnames to add to a cluster.\neg: x1003c1s7b0n0,x1003c1s7b0n1,x1003c1s7b1n0"))
                .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
                .arg(arg!(-c --"create-hsm-group" "If the target cluster name does not exist as HSM group, create it."))
            )
        )
        .subcommand(Command::new("remove")
            .alias("r")
            .arg_required_else_help(true)
            .about("WIP - Remove hw components from cluster")
            .subcommand(Command::new("hw-component")
                .alias("hw")
                .arg_required_else_help(true)
                .about("WIP - Remove hw components from a cluster")
                .arg(arg!(-P --pattern <PATTERN> "Pattern"))
                .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the pattern is applying to (resources move from here)."))
                .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one receiving resources from the target cluster (resources move here)."))
                .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
                .arg(arg!(-d --"delete-hsm-group" "Delete the HSM group if empty after this action."))
            )
            .subcommand(Command::new("nodes")
                .aliases(["n", "node"])
                .about("WIP - Remove nodes to a cluster")
                .arg_required_else_help(true)
                .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the nodes are moving from (resources move from here)."))
                .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one receiving nodes from the target cluster (resources move here)."))
                .arg(arg!(<XNAMES> "Comma separated list of xnames to add to a cluster.\neg: x1003c1s7b0n0,x1003c1s7b0n1,x1003c1s7b1n0"))
                .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
                .arg(arg!(-d --"delete-hsm-group" "Delete the HSM group if empty after this action."))

            )
        )
        .subcommand(
            Command::new("set")
                .alias("s")
                .arg_required_else_help(true)
                .about("Set specific features on cluster nodes or a list of nodes")
                .subcommand(subcommand_set_runtime_configuration(hsm_group))
                .subcommand(subcommand_set_boot_configuration(hsm_group))
                .subcommand(subcommand_set_boot_image(hsm_group))
                .subcommand(subcommand_set_kernel_params(hsm_group)))
        .subcommand(
            Command::new("apply")
                .alias("a")
                .arg_required_else_help(true)
                .about("Make changes to Shasta system")
                .subcommand(subcommand_apply_hw_configuration())
                .subcommand(subcommand_apply_configuration(hsm_group))
                .subcommand(subcommand_apply_image(/* hsm_group */))
                .subcommand(subcommand_apply_cluster(/* hsm_group */))
                .subcommand(subcommand_apply_sat_file(/* hsm_group */))
                .subcommand(Command::new("boot")
                    .alias("b")
                    .arg_required_else_help(true)
                    .about("Change boot operations")
                    .subcommand(subcommand_apply_boot_nodes(hsm_group))
                    .subcommand(subcommand_apply_boot_cluster(hsm_group))
                )
                .subcommand(
                    Command::new("node")
                        .aliases(["n", "nod"])
                        .arg_required_else_help(true)
                        .about("DEPRECATED - Please use 'manta power' command instead.\nPower management for a list of nodes")
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
            Command::new("migrate")
                .alias("m")
                .arg_required_else_help(true)
                .about("WIP - Migrate vCluster")
                .subcommand(subcommand_migrate_backup())
                .subcommand(subcommand_migrate_restore()),
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
            subcommand_log(hsm_group)
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
        .subcommand(subcommand_validate_local_repo())
        .subcommand(subcommand_config())
}

pub fn subcommand_config() -> Command {
    // Enforce user to chose a HSM group is hsm_available config param is not empty. This is to
    // make sure tenants like PSI won't unset parameter hsm_group and take over all HSM groups.
    // NOTE: by default 'manta config set hsm' will unset the hsm_group config value and the user
    // will be able to access any HSM. The security meassures for this is to setup sticky bit to
    // manta binary so it runs as manta user, then 'chown manta:manta /home/manta/.config/manta/config.toml' so only manta and root users can edit the config file. Tenants can neither su to manta nor root under the access VM (eg castaneda)
    /* let subcommand_config_set_hsm = if !hsm_available_opt.is_empty() {
        Command::new("hsm")
            .about("Change config values")
            .about("Set target HSM group")
            .arg(arg!(<HSM_GROUP_NAME> "hsm group name"))
    } else {
        Command::new("hsm")
            .about("Change config value")
            .about("Set target HSM group")
            .arg(arg!([HSM_GROUP_NAME] "hsm group name"))
    }; */

    let subcommand_config_set_hsm = Command::new("hsm")
        .about("Set target HSM group")
        .arg(arg!(<HSM_GROUP_NAME> "hsm group name"));

    let subcommand_config_set_parent_hsm = Command::new("parent-hsm")
        .about("Set parent HSM group")
        .arg(arg!(<HSM_GROUP_NAME> "hsm group name"));

    let subcommand_config_set_site = Command::new("site")
        .about("Set site to work on")
        .arg(arg!(<SITE_NAME> "site name"));

    let subcommand_config_set_log = Command::new("log").about("Set site to work on").arg(
        arg!(<LOG_LEVEL> "log verbority level")
            .value_parser(["error", "warn", "info", "debug", "trace"]),
    );

    let subcommand_config_unset_hsm = Command::new("hsm").about("Unset target HSM group");

    let subcommand_config_unset_parent_hsm =
        Command::new("parent-hsm").about("Unset parent HSM group");

    let subcommand_config_unset_auth = Command::new("auth").about("Unset authentication token");

    Command::new("config")
        .alias("C")
        .arg_required_else_help(true)
        .about("Manta's configuration")
        .subcommand(Command::new("show").about("Show config values"))
        .subcommand(
            Command::new("set")
                .arg_required_else_help(true)
                .about("Change config values")
                .subcommand(subcommand_config_set_hsm)
                .subcommand(subcommand_config_set_parent_hsm)
                .subcommand(subcommand_config_set_site)
                .subcommand(subcommand_config_set_log),
        )
        .subcommand(
            Command::new("unset")
                .arg_required_else_help(true)
                .about("Reset config values")
                .subcommand(subcommand_config_unset_hsm)
                .subcommand(subcommand_config_unset_parent_hsm)
                .subcommand(subcommand_config_unset_auth),
        )
}

pub fn subcommand_delete(hsm_group: Option<&String>) -> Command {
    let mut delete = Command::new("delete")
                .arg_required_else_help(true)
                .about("Deletes CFS configurations, CFS sessions, BOS sessiontemplates, BOS sessions and images related to CFS configuration/s.")
                .arg(arg!(-n --"configuration-name" <CONFIGURATION> "CFS configuration, CFS sessions, BOS sessiontemplate, BOS sessions and IMS images related to the CFS configuration will be deleted.\neg:\nmanta delete --configuration-name my-config-v1.0\nDeletes all data related to CFS configuration with name 'my-config-v0.1'"))
                .arg(arg!(-p --pattern <CONFIGURATION_NAME_PATTERN> "Glob pattern for configuration name"))
                .arg(arg!(-s --since <DATE> "Deletes CFS configurations, CFS sessions, BOS sessiontemplate, BOS sessions and images related to CFS configurations with 'last updated' after since date. Note: date format is %Y-%m-%d\neg:\nmanta delete --since 2023-01-01 --until 2023-10-01\nDeletes all data related to CFS configurations created or updated between 01/01/2023T00:00:00Z and 01/10/2023T00:00:00Z"))
                .arg(arg!(-u --until <DATE> "Deletes CFS configuration, CFS sessions, BOS sessiontemplate, BOS sessions and images related to the CFS configuration with 'last updated' before until date. Note: date format is %Y-%m-%d\neg:\nmanta delete --until 2023-10-01\nDeletes all data related to CFS configurations created or updated before 01/10/2023T00:00:00Z"))
                .arg(arg!(-y --"yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively. Image artifacts and configurations used by nodes will not be deleted"))
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

pub fn subcommand_get_hw_components() -> Command {
    let command_get_hs_configuration_cluster = Command::new("cluster")
                .aliases(["c", "clstr"])
                .arg_required_else_help(true)
                .about("Get hw components for a cluster")
                .arg(arg!(<CLUSTER_NAME> "Name of the cluster").required(true))
                .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json", "summary", "details"]).default_value("summary"));

    let command_get_hs_configuration_node = Command::new("node")
                .alias("n")
                .arg_required_else_help(true)
                .about("Get hw components for some nodes")
                .arg(arg!(<XNAMES> "List of xnames separated by commas").required(true))
                .arg(arg!(-t --type <TYPE> "Filters output to specific type").value_parser(ArtifactType::iter().map(|e| e.into()).collect::<Vec<&str>>()))
                .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]));

    Command::new("hw-component")
        .alias("hw")
        .arg_required_else_help(true)
        .about("Get hardware components1 for a cluster or a node")
        .subcommand(command_get_hs_configuration_cluster)
        .subcommand(command_get_hs_configuration_node)
}

pub fn subcommand_get_cfs_configuration(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_configuration = Command::new("configurations")
        .aliases(["c", "cfg", "conf", "config", "cnfgrtn", "configuration"])
        .about("Get information from Shasta CFS configuration")
        .arg(arg!(-n --name <CONFIGURATION_NAME> "configuration name"))
        .arg(arg!(-p --pattern <CONFIGURATION_NAME_PATTERN> "Glob pattern for configuration name"))
        .arg(arg!(-m --"most-recent" "Only shows the most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of CFS configurations created")
                .value_parser(value_parser!(u8).range(1..)),
        )
        .arg(arg!(-o --output <FORMAT> "Output format. If missing, it will print output data in human redeable (tabular) format").value_parser(["json"]));

    match hsm_group {
        None => {
            get_cfs_configuration = get_cfs_configuration
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_configuration").args(["hsm-group", "name"]))
        }
        Some(_) => {}
    }

    get_cfs_configuration = get_cfs_configuration
        .group(ArgGroup::new("configuration_limit").args(["most-recent", "limit"]));

    get_cfs_configuration
}

pub fn subcommand_get_cfs_session(hsm_group: Option<&String>) -> Command {
    let mut get_cfs_session = Command::new("sessions")
        .aliases(["s", "se", "ses", "sess", "sssn", "session"])
        .about("Get information from Shasta CFS session")
        .arg(arg!(-n --name <SESSION_NAME> "Return only sessions with the given session name"))
        .arg(arg!(-a --"min-age" <MIN_AGE> "Return only sessions older than the given age. Age is given in the format '1d' or '6h'"))
        .arg(arg!(-A --"max-age" <MAX_AGE> "Return only sessions younger than the given age. Age is given in the format '1d' or '6h'"))
        .arg(arg!(-s --status <SESSION_STATUS> "Return only sessions with the given status")
            .value_parser(["pending", "running", "complete"]))
        .arg(arg!(-m --"most-recent" "Return only the most recent session created (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Return only last <VALUE> sessions created")
                .value_parser(value_parser!(u8).range(1..)),
        )
        .arg(arg!(-o --output <FORMAT> "Output format. If missing, it will print output data in human redeable (tabular) format").value_parser(["json"]))
        .arg(arg!(-x --xnames <XNAMES> "List of xnames"));

    match hsm_group {
        None => {
            get_cfs_session = get_cfs_session
                .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_xnames_or_name").args([
                    "hsm-group",
                    "xnames",
                    "name",
                ]))
        }
        Some(_) => {}
    }

    get_cfs_session =
        get_cfs_session.group(ArgGroup::new("session_limit").args(["most-recent", "limit"]));

    get_cfs_session
}

pub fn subcommand_get_bos_template(hsm_group: Option<&String>) -> Command {
    let mut get_bos_template = Command::new("templates")
        .aliases(["t", "tplt", "templ", "tmplt", "template"])
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

pub fn subcommand_get_cluster_details(hsm_group: Option<&String>) -> Command {
    let mut get_node = Command::new("cluster")
        .aliases(["C", "clstr"])
        .about("Get cluster details")
        .arg(arg!(-n --"nids-only-one-line" "Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,..."))
        .arg(arg!(-x --"xnames-only-one-line" "Prints xnames in one line eg x1001c1s5b0n0,x1001c1s5b0n1,..."))
        .arg(arg!(-s --"status" "Get cluster status:\n - OK: All nodes are operational (booted and configured)\n - OFF: At least one node is OFF\n - ON: No nodes OFF and at least one is ON\n - STANDBY: At least one node's heartbeat is lost\n - UNCONFIGURED: All nodes are READY but at least one of them is being configured\n - FAILED: At least one node configuration failed"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json", "summary"]));

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

pub fn subcommand_get_node(hsm_group: Option<&String>) -> Command {
    let mut get_node = Command::new("nodes")
        .aliases(["n", "node", "nd"])
        .about("DEPRECATED - Please use 'manta get cluster' command instead.\nGet members of a HSM group")
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
        .about("DEPRECATED - Please do not use this command.\nGet HSM groups details");

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
        .arg(arg!(-i --id <VALUE> "Image ID"))
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
        .about("Get information from CSM system")
        .subcommand(subcommand_get_hw_components())
        .subcommand(subcommand_get_cfs_session(hsm_group))
        .subcommand(subcommand_get_cfs_configuration(hsm_group))
        .subcommand(subcommand_get_bos_template(hsm_group))
        .subcommand(subcommand_get_node(hsm_group))
        .subcommand(subcommand_get_cluster_details(hsm_group))
        .subcommand(subcommand_get_hsm_groups_details(hsm_group))
        .subcommand(subcommand_get_images(hsm_group))
}

pub fn subcommand_apply_hw_configuration() -> Command {
    Command::new("hw-configuration")
        .alias("hw")
        .about("WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration")
        .arg_required_else_help(true)
        .subcommand(Command::new("cluster")
            .aliases(["c", "clstr"])
            .arg_required_else_help(true)
            .about("WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration")
            .arg(arg!(-P -- pattern <VALUE> "Hw pattern with keywords to fuzzy find hardware componented to assign to the cluster like <hw component name>:<hw component quantity>[:<hw component name>:<hw component quantity>]. Eg 'a100:12:epic:5' will update the nodes assigned to cluster 'zinal' with 4 nodes:\n - 3 nodes with 4 Nvidia gpus A100 and 1 epyc AMD cpu each\n - 1 node with 2 epyc AMD cpus").required(true))
            .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the pattern is applying to.").required(true))
            .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.").required(true))
            .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
            .arg(arg!(-c --"create-target-hsm-group" "If the target cluster name does not exist as HSM group, create it."))
            .arg(arg!(-d --"delete-empty-parent-hsm-group" "If the target HSM group is empty after this action, remove it."))
            .arg(arg!(-u --"unpin-nodes" "It will try to get any nodes available."))

                    // .arg(arg!(-f --file <SAT_FILE> "file with hw configuration details").value_parser(value_parser!(PathBuf)).required(true))
        )
}

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
        .arg(arg!(-P --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."));

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

pub fn subcommand_apply_configuration(hsm_group: Option<&String>) -> Command {
    let mut apply_configuration = Command::new("configuration")
        .aliases(["conf", "config"])
        .arg_required_else_help(true)
        .about("DEPRECATED - Please use 'manta apply sat-file' command instead.\nCreate a CFS configuration")
        // .about("Create a CFS configuration")
        .arg(arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-f --"values-file" <VALUES_FILE_PATH> "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using this values file.").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-V --"values" <VALUES_PATH> ... "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided."))
        // .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]))
        // .arg(arg!(-n --name <VALUE> "Configuration name"))
        // .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").value_parser(value_parser!(PathBuf)))
        // .group(ArgGroup::new("req_flags_name_repo-path").args(["name", "repo-path"]))
        // .group(ArgGroup::new("req_flag_file").arg("file"))
        ;

    match hsm_group {
        Some(_) => {}
        None => apply_configuration = apply_configuration.arg(
            arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name linked to this configuration"),
        ),
    };

    apply_configuration
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
        .about("DEPRECATED - Please use 'manta apply sat-file' command instead.\nCreate a CFS configuration and a CFS image")
        // .about("Create a CFS configuration and a CFS image")
        .arg(arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-f --"values-file" <VALUES_FILE_PATH> "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using this values file.").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-V --"values" <VALUES_PATH> ... "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided."))
        // .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        /* .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image")
           .value_parser(value_parser!(PathBuf))) */
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-P --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]))
}

pub fn subcommand_apply_cluster(/* hsm_group: Option<&String> */) -> Command {
    Command::new("cluster")
        .aliases(["clus","clstr"])
        .arg_required_else_help(true)
        .about("DEPRECATED - Please use 'manta apply sat-file' command instead.\nCreate a CFS configuration, a CFS image, a BOS sessiontemplate and a BOS session")
        // .about("Create a CFS configuration, a CFS image, a BOS sessiontemplate and a BOS session")
        .arg(arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-f --"values-file" <VALUES_FILE_PATH> "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using this values file.").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-V --"values" <VALUES_PATH> ... "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided."))
        .arg(arg!(--"do-not-reboot" "By default, nodes will restart if SAT file builds an image which is assigned to the nodes through a BOS sessiontemplate, if you do not want to reboot the nodes, then use this flag. The SAT file will be processeed as usual and different elements created but the nodes won't reboot."))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-P --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]))
}

pub fn subcommand_apply_sat_file(/* hsm_group: Option<&String> */) -> Command {
    Command::new("sat-file")
        .alias("sat")
        .arg_required_else_help(true)
        .about("Process a SAT file and creates the configurations, images, boot parameters and reboots the nodes to configure.")
        // .about("Create a CFS configuration, a CFS image, a BOS sessiontemplate and a BOS session")
        .arg(arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-f --"values-file" <VALUES_FILE_PATH> "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using this values file.").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-V --"values" <VALUES_PATH> ... "WIP - If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided."))
        .arg(arg!(--"do-not-reboot" "By default, nodes will restart if SAT file builds an image which is assigned to the nodes through a BOS sessiontemplate, if you do not want to reboot the nodes, then use this flag. The SAT file will be processeed as usual and different elements created but the nodes won't reboot."))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-P --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs."))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (tabular) format").value_parser(["json"]))
        .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before processing SAT file. If need to pass a command with params. Use \" or \'.\neg: --pre-hook \"echo hello\""))
        .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run immediately after processing SAT file successfully. Use \" or \'.\neg: --post-hook \"echo hello\"."))
}

pub fn subcommand_apply_node_on(hsm_group: Option<&String>) -> Command {
    let mut apply_node_on = Command::new("on")
        .about("DEPRECATED - Please use 'manta power on' command instead.\nStart nodes")
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
        .about("DEPRECATED - Please use 'manta power off' command instead.\nShutdown nodes")
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
        .about("DEPRECATED - Please use 'manta power reset' command instead.\nRestart nodes")
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
        .about("DEPRECATED - Please use 'manta apply boot nodes' command instead.\nUpdates boot and configuration of a group of nodes. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"boot-image" <CFS_CONFIG> "CFS configuration name related to the image to boot the nodes"))
        .arg(arg!(-d --"desired-configuration" <CFS_CONFIG> "CFS configuration name to configure the nodes after booting"));

    update_nodes = update_nodes
        .arg(arg!(<XNAMES> "Comma separated list of xnames which boot image will be updated"));

    update_nodes = match hsm_group {
        Some(_) => update_nodes,
        None => update_nodes.arg(arg!([HSM_GROUP_NAME] "hsm group name, this field should be used to validate the XNAMES belongs to HSM_GROUP_NAME")),
    };

    update_nodes
}

pub fn subcommand_update_hsm_group(hsm_group: Option<&String>) -> Command {
    let mut update_hsm_group = Command::new("hsm-group")
        .aliases(["h", "hsm"])
        .arg_required_else_help(true)
        .about("DEPRECATED - Please use 'manta apply boot cluster' command instead.\nUpdates boot and configuration of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"boot-image" <CFS_CONFIG> "CFS configuration name related to the image to boot the nodes"))
        .arg(arg!(-d --"desired-configuration" <CFS_CONFIG> "CFS configuration name to configure the nodes after booting"));

    update_hsm_group = match hsm_group {
        Some(_) => update_hsm_group,
        None => update_hsm_group.arg(arg!(<HSM_GROUP_NAME> "HSM group name").required(true)),
    };

    update_hsm_group
}

pub fn subcommand_apply_boot_nodes(hsm_group: Option<&String>) -> Command {
    let mut apply_boot_nodes = Command::new("nodes")
        .aliases(["n", "node"])
        .arg_required_else_help(true)
        .about("Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id.\neg:\nmanta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>")
        .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
        .arg(arg!(-b --"boot-image-configuration" <CFS_CONFIG_NAME> "CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes"))
        .arg(arg!(-r --"runtime-configuration" <CFS_CONFIG_NAME> "CFS configuration name to configure the nodes after booting"))
        // .arg(arg!(-k --"kernel-params" <VALUE> "Kernel boot parameters to assigned to the nodes while booting"))
        .group(ArgGroup::new("boot-image_or_boot-config").args(["boot-image", "boot-image-configuration"]));

    apply_boot_nodes = apply_boot_nodes
        .arg(arg!(<XNAMES> "Comma separated list of xnames which boot image will be updated"));

    apply_boot_nodes = match hsm_group {
        Some(_) => apply_boot_nodes,
        None => apply_boot_nodes.arg(arg!([CLUSTER_NAME] "Cluster name, this field should be used to validate the XNAMES belongs to CLUSTER_NAME")),
    };

    apply_boot_nodes
}

pub fn subcommand_apply_boot_cluster(hsm_group: Option<&String>) -> Command {
    let mut apply_boot_cluster = Command::new("cluster")
        .alias("c")
        .arg_required_else_help(true)
        .about("Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id.\neg:\nmanta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>")
        .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
        .arg(arg!(-b --"boot-image-configuration" <CFS_CONFIG_NAME> "CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes"))
        .arg(arg!(-r --"runtime-configuration" <CFS_CONFIG_NAME> "CFS configuration name to configure the nodes after booting"))
        // .arg(arg!(-k --"kernel-params" <VALUE> "Kernel boot parameters to assigned to the nodes while booting"))
        .group(ArgGroup::new("boot-image_or_boot-config").args(["boot-image", "boot-image-configuration"]));

    apply_boot_cluster = match hsm_group {
        Some(_) => apply_boot_cluster,
        None => apply_boot_cluster.arg(arg!(<CLUSTER_NAME> "Cluster name").required(true)),
    };

    apply_boot_cluster
}

pub fn subcommand_migrate_backup() -> Command {
    Command::new("backup")
        .aliases(["mb"])
        .arg_required_else_help(true)
        .about("Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.")
        .arg(arg!(-b --"bos" <SESSIONTEMPLATE> "BOS Sessiontemplate to use to derive CFS, boot parameters and HSM group"))
        .arg(arg!(-d --"destination" <FOLDER> "Destination folder to store the backup on"))
        .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before doing the backup. If need to pass a command with params. Use \" or \'.\neg: --pre-hook \"echo hello\""))
        .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run immediately after the backup is completed successfully. Use \" or \'.\neg: --post-hook \"echo hello\"."))
}

pub fn subcommand_migrate_restore() -> Command {
    Command::new("restore")
        .aliases(["mr"])
        .arg_required_else_help(true)
        .about("MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"bos-file" <BOS_session_template_file> "BOS session template of the cluster backed previously with migrate backup"))
        .arg(arg!(-c --"cfs-file" <CFS_configuration_file> "CFS session template of the cluster backed previously with migrate backup"))
        .arg(arg!(-j --"hsm-file" <HSM_group_description_file> "HSM group description file of the cluster backed previously with migrate backup"))
        .arg(arg!(-m --"ims-file" <IMS_file> "IMS file backed previously with migrate backup"))
        .arg(arg!(-i --"image-dir" <IMAGE_path> "Path where the image files are stored."))
        .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before doing the backup. If need to pass a command with params. Use \" or \'.\neg: --pre-hook \"echo hello\""))
        .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run immediately after the backup is completed successfully. Use \" or \'.\neg: --pre-hook \"echo hello\"."))
}

pub fn subcommand_power() -> Command {
    Command::new("power")
        .aliases(["p", "pwr"])
        .arg_required_else_help(true)
        .about("Command to submit commands related to cluster/node power management")
        .subcommand(
            Command::new("on")
                .arg_required_else_help(true)
                .about("Command to power on cluster/node")
                .subcommand(
                    Command::new("cluster")
                        .aliases(["c", "clstr"])
                        .arg_required_else_help(true)
                        .about("Command to power on all nodes in a cluster")
                        .arg(arg!(-r --reason <TEXT> "reason to power on"))
                        .arg(arg!(<CLUSTER_NAME> "Cluster name")),
                )
                .subcommand(
                    Command::new("node")
                        .alias("n")
                        .arg_required_else_help(true)
                        .about("Command to power on a group of nodes")
                        .arg(arg!(-r --reason <TEXT> "reason to power on"))
                        .arg(arg!(<NODE_NAME> "Node name")),
                ),
        )
        .subcommand(
            Command::new("off")
                .arg_required_else_help(true)
                .about("Command to power off cluster/node")
                .subcommand(
                    Command::new("cluster")
                        .aliases(["c", "clstr"])
                        .arg_required_else_help(true)
                        .about("Command to power off all nodes in a cluster")
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-r --reason <TEXT> "reason to power off"))
                        .arg(arg!(<CLUSTER_NAME> "Cluster name")),
                )
                .subcommand(
                    Command::new("node")
                        .alias("n")
                        .arg_required_else_help(true)
                        .about("Command to power off a group of nodes")
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-r --reason <TEXT> "reason to power off"))
                        .arg(arg!(<NODE_NAME> "Node name")),
                ),
        )
        .subcommand(
            Command::new("reset")
                .arg_required_else_help(true)
                .about("Command to power reset cluster/node")
                .subcommand(
                    Command::new("cluster")
                        .aliases(["c", "clstr"])
                        .arg_required_else_help(true)
                        .about("Command to power reset all nodes in a cluster")
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-r --reason <TEXT> "reason to power reset"))
                        .arg(arg!(<CLUSTER_NAME> "Cluster name")),
                )
                .subcommand(
                    Command::new("node")
                        .alias("n")
                        .arg_required_else_help(true)
                        .about("Command to power reset a group of nodes")
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-r --reason <TEXT> "reason to power reset"))
                        .arg(arg!(<NODE_NAME> "Node name")),
                ),
        )
}

pub fn subcommand_log(hsm_group_opt: Option<&String>) -> Command {
    let mut log = Command::new("log")
        .alias("l")
        .about("get cfs session logs")
        .arg(arg!([SESSION_NAME] "show logs related to session name"));

    match hsm_group_opt {
        None => {
            log =
                log.arg(arg!(-c --cluster <cluster_name> "Show logs most recent CFS session created for cluster."))
                    .group(ArgGroup::new("cluster_or_session_name").args(["cluster", "SESSION_NAME"]));
        }
        Some(_) => {}
    }

    log
}

pub fn subcommand_validate_local_repo() -> Command {
    Command::new("validate-local-repo")
        .alias("vlr")
        .about("Check all tags and HEAD information related to a local repo exists in Gitea")
        .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path to a local a git repo related to a CFS configuration layer to test against Gitea").required(true))
}

pub fn subcommand_set_runtime_configuration(hsm_group_opt: Option<&String>) -> Command {
    let mut set_runtime_configuration = Command::new("runtime-configuration")
        .alias("rc")
        .about("Set runtime-configuration to a set of nodes or all nodes in a cluster")
        .arg(arg!(-c --"configuration" <VALUE> "Configuration name to set").required(true));

    match hsm_group_opt {
        None => {
            set_runtime_configuration = set_runtime_configuration
                .arg(arg!(-x --xnames <XNAMES> "List of nodes to set runtime configuration"))
                .arg(arg!(-H --"hsm-group" <HSM_GROUP> "Cluster to set runtime configuration"))
                .group(
                    ArgGroup::new("cluster_or_session_name")
                        .args(["hsm-group", "xnames"])
                        .required(true),
                );
        }
        Some(_) => {}
    }

    set_runtime_configuration
}

pub fn subcommand_set_boot_image(hsm_group_opt: Option<&String>) -> Command {
    let mut set_boot_image = Command::new("boot-image")
        .alias("bi")
        .about("Set boot image to boot a set of nodes or all nodes in a cluster")
        .arg(arg!(-i --"image-id" <VALUE> "Image id to set").required(true));

    match hsm_group_opt {
        None => {
            set_boot_image = set_boot_image
                .arg(arg!(-x --xnames <XNAMES> "List of nodes to set runtime configuration"))
                .arg(arg!(-H --"hsm-group" <HSM_GROUP> "Cluster to set runtime configuration"))
                .group(
                    ArgGroup::new("cluster_or_session_name")
                        .args(["hsm-group", "xnames"])
                        .required(true),
                );
        }
        Some(_) => {}
    }

    set_boot_image
}

pub fn subcommand_set_boot_configuration(hsm_group_opt: Option<&String>) -> Command {
    let mut set_boot_image = Command::new("boot-configuration")
        .alias("bc")
        .about("Set boot configuration to boot a set of nodes or all nodes in a cluster. The algorithm will look for the most recent image id created with the provided configuration name and assign it to the nodes.")
        .arg(arg!(-c --"configuration" <VALUE> "Configuration name to set").required(true));

    match hsm_group_opt {
        None => {
            set_boot_image = set_boot_image
                .arg(arg!(-x --xnames <XNAMES> "List of nodes to set runtime configuration"))
                .arg(arg!(-H --"hsm-group" <HSM_GROUP> "Cluster to set runtime configuration"))
                .group(
                    ArgGroup::new("cluster_or_xnames")
                        .args(["hsm-group", "xnames"])
                        .required(true),
                );
        }
        Some(_) => {}
    }

    set_boot_image
}

pub fn subcommand_set_kernel_params(hsm_group_opt: Option<&String>) -> Command {
    let mut set_boot_image = Command::new("kernel-parameters")
        .alias("kp")
        .about("Set kernel boot parameters to boot a set of nodes or all nodes in a cluster.")
        .arg(arg!(-k --"kernel-parameters" <VALUE> "List of kernel parameters to a set of nodes or all nodes in a cluster. Do not add any parameter related to the boot image rootfs, this information needs to be configured using `manta set boot-configuration` or `manta set boot-image`").required(true));

    match hsm_group_opt {
        None => {
            set_boot_image = set_boot_image
                .arg(arg!(-x --xnames <XNAMES> "List of nodes to set runtime configuration"))
                .arg(arg!(-H --"hsm-group" <HSM_GROUP> "Cluster to set runtime configuration"))
                .group(
                    ArgGroup::new("cluster_or_xnames")
                        .args(["hsm-group", "xnames"])
                        .required(true),
                );
        }
        Some(_) => {}
    }

    set_boot_image
}
