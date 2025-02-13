use clap::{arg, value_parser, ArgAction, ArgGroup, Command, ValueHint};
use mesa::hsm::hw_inventory::hw_component::r#struct::ArtifactType;
use strum::IntoEnumIterator;

use std::path::PathBuf;

pub fn build_cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .term_width(100)
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommand(subcommand_config())
        .subcommand(subcommand_get())
        .subcommand(subcommand_add())
        .subcommand(subcommand_apply())
        .subcommand(subcommand_delete())
        .subcommand(subcommand_migrate())
        .subcommand(subcommand_power())
        .subcommand(subcommand_log())
        .subcommand(subcommand_console())
        .subcommand(subcommand_validate_local_repo())
        .subcommand(subcommand_add_nodes_to_groups())
        .subcommand(subcommand_remove_nodes_from_groups())
}

pub fn subcommand_config() -> Command {
    // Enforce user to choose a HSM group is hsm_available config param is not empty. This is to
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
        // .visible_alias("C")
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
        .subcommand(
            Command::new("gen-autocomplete")
                // .alias("gen-autocomplete")
                .about("Generate shell auto completion script")
                .arg(
                    arg!(-s --shell <SHELL> "Shell type. Will try to guess from $SHELL if missing")
                        .value_parser(["bash", "zsh", "fish"]),
                )
                .arg(
                    arg!(-p --path <PATH> "Path to put the autocomplete script or prints to stdout if missing.\nNOTE: Do not specify filename, only path to directory")
                    .value_parser(value_parser!(PathBuf)).value_hint(ValueHint::DirPath),
                ),
        )
}

pub fn subcommand_delete() -> Command {
    Command::new("delete")
        // .visible_alias("d")
        .arg_required_else_help(true)
        .about("Deletes data")
        .subcommand(subcommand_delete_group())
        .subcommand(subcommand_delete_kernel_parameter())
        .subcommand(subcommand_delete_session())
        .subcommand(subcommand_delete_image())
        .subcommand(subcommand_delete_hw_component())
}

pub fn subcommand_delete_group() -> Command {
    Command::new("group")
                // .visible_alias("g")
                .arg_required_else_help(true)
                .about("Delete group. This command will fail if any node becomes orphan (no groups assigned to it), please move group members to another group using command 'migrate nodes' before deletion")
                .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                .arg(arg!(-D --"dryrun" "No changes applied to the system.").action(ArgAction::SetTrue))
                .arg(arg!(<VALUE> "Group name to delete"))
}

pub fn subcommand_delete_hw_component() -> Command {
    Command::new("hw-component")
                // .visible_alias("hw")
                .arg_required_else_help(true)
                .about("WIP - Remove hw components from a cluster")
                .arg(arg!(-P --pattern <PATTERN> "Pattern"))
                .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the pattern is applying to (resources move from here)."))
                .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one receiving resources from the target cluster (resources move here)."))
                .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
                .arg(arg!(-d --"delete-hsm-group" "Delete the HSM group if empty after this action."))
}

pub fn subcommand_delete_image() -> Command {
    Command::new("images")
            // .visible_alias("di")
            .arg_required_else_help(true)
            .about("WIP - Deletes a list of images.")
            .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
            .arg(arg!(<IMAGE_LIST> "Comma separated list of image ids to delete/\neg: e2ce82f0-e7ba-4f36-9f5c-750346599600,59e0180a-3fdd-4936-bba7-14ba914ffd34").required(true))
}

pub fn subcommand_delete_session() -> Command {
    Command::new("session")
            // .visible_alias("ds")
            .arg_required_else_help(true)
            .about("Deletes a session. For 'image' sessions, it also removes the associated image. For 'dynamic' sessions, it sets the 'error count' to its maximum value.")
            .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
            .arg(arg!(<SESSION_NAME> "Session name to delete").required(true))
}

pub fn subcommand_delete_kernel_parameter() -> Command {
    Command::new("kernel-parameters")
        // .visible_alias("kp")
        .arg_required_else_help(true)
        .about("Delete kernel parameters")
        .arg(arg!(-x --xnames <XNAMES> "Comma separated list of nodes to set runtime configuration.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'"))
        .arg(arg!(-H --"hsm-group" <HSM_GROUP> "Cluster to set runtime configuration"))
        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
        .arg(arg!(<VALUE> "Space separated list of kernel parameters. Eg: console,bad_page,crashkernel,hugepagelist,quiet"))
        .group(
            ArgGroup::new("cluster_or_xnames")
                .args(["hsm-group", "xnames"])
                .required(true),
        )
}

pub fn subcommand_get_hw_components() -> Command {
    let command_get_hs_configuration_cluster = Command::new("cluster")
                // .visible_alias("c")
                .arg_required_else_help(true)
                .about("Get hw components for a cluster")
                .arg(arg!(<CLUSTER_NAME> "Name of the cluster").required(true))
                .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (table) format").value_parser(["json", "summary", "details"]).default_value("summary"));

    let command_get_hs_configuration_node = Command::new("node")
                // .visible_alias("n")
                .arg_required_else_help(true)
                .about("Get hw components for some nodes")
                .arg(arg!(<XNAMES> "Comma separated list of xnames.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'").required(true))
                .arg(arg!(-t --type <TYPE> "Filters output to specific type").value_parser(ArtifactType::iter().map(|e| e.into()).collect::<Vec<&str>>()))
                .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (table) format").value_parser(["json"]));

    Command::new("hw-component")
        // .visible_alias("hw")
        .arg_required_else_help(true)
        .about("Get hardware components1 for a cluster or a node")
        .subcommand(command_get_hs_configuration_cluster)
        .subcommand(command_get_hs_configuration_node)
}

pub fn subcommand_get_cfs_configuration() -> Command {
    Command::new("configurations")
        // .visible_alias("c"])
        .about("Get information from Shasta CFS configuration")
        .arg(arg!(-n --name <CONFIGURATION_NAME> "configuration name"))
        .arg(arg!(-p --pattern <CONFIGURATION_NAME_PATTERN> "Glob pattern for configuration name"))
        .arg(arg!(-m --"most-recent" "Only shows the most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of CFS configurations created")
                .value_parser(value_parser!(u8).range(1..)),
        )
        .arg(arg!(-o --output <FORMAT> "Output format. If missing, it will print output data in human redeable (table) format").value_parser(["json"]))
        .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
                .group(ArgGroup::new("hsm-group_or_configuration").args(["hsm-group", "name"]))
        .group(ArgGroup::new("configuration_limit").args(["most-recent", "limit"]))
}

pub fn subcommand_get_cfs_session() -> Command {
    Command::new("sessions")
        // .visible_alias("s")
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
        .arg(arg!(-o --output <FORMAT> "Output format. If missing, it will print output data in human redeable (table) format").value_parser(["json"]))
        .arg(arg!(-x --xnames <XNAMES> "Comma separated list of xnames.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'"))
        .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
        .group(ArgGroup::new("hsm-group_or_xnames_or_name").args([
                    "hsm-group",
                    "xnames",
                    "name",
                ]))
        .group(ArgGroup::new("session_limit").args(["most-recent", "limit"]))
}

pub fn subcommand_get_bos_template() -> Command {
    Command::new("templates")
        // .visible_alias("t")
        .about("Get information from Shasta BOS template")
        .arg(arg!(-n --name <TEMPLATE_NAME> "template name"))
        .arg(arg!(-m --"most-recent" "Only shows the most recent (equivalent to --limit 1)"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of BOS templates created")
                .value_parser(value_parser!(u8).range(1..)),
        )
        .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (table) format").value_parser(["json", "table"]).default_value("table"))
        .group(ArgGroup::new("hsm-group_or_template").args(["hsm-group", "name"]))
}

pub fn subcommand_get_cluster_details() -> Command {
    Command::new("cluster")
        // .visible_alias("C")
        .about("Get cluster details")
        .arg(arg!(-n --"nids-only-one-line" "Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,..."))
        .arg(arg!(-x --"xnames-only-one-line" "Prints xnames in one line eg x1001c1s5b0n0,x1001c1s5b0n1,..."))
        .arg(arg!(-s --status "Get cluster status:\n - OK: All nodes are operational (booted and configured)\n - OFF: At least one node is OFF\n - ON: No nodes OFF and at least one is ON\n - STANDBY: At least one node's heartbeat is lost\n - UNCONFIGURED: All nodes are READY but at least one of them is being configured\n - FAILED: At least one node configuration failed"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human readable (table) format").value_parser(["table", "table-wide", "json", "summary"]).default_value("table"))
        .arg_required_else_help(true)
        .arg(arg!(<HSM_GROUP_NAME> "hsm group name"))
}

pub fn subcommand_get_node_details() -> Command {
    Command::new("nodes")
        // .visible_alias("n")
        .about("Get node details")
        .arg(arg!(-n --"nids-only-one-line" "Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,..."))
        .arg(arg!(-s --status "Get cluster status:\n - OK: All nodes are operational (booted and configured)\n - OFF: At least one node is OFF\n - ON: No nodes OFF and at least one is ON\n - STANDBY: At least one node's heartbeat is lost\n - UNCONFIGURED: All nodes are READY but at least one of them is being configured\n - FAILED: At least one node configuration failed"))
        .arg(arg!(-S --"include-siblings" "Output includes extra nodes related to the ones requested by used. 2 nodes are siblings if they share the same power supply.").action(ArgAction::SetTrue))
        .arg(arg!(-r --regex "Input nodes in regex format.").action(ArgAction::SetTrue))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human readable (table) format").value_parser(["table", "table-wide", "json", "summary"]).default_value("table"))
        .arg_required_else_help(true)
        .arg(arg!(<XNAMES> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'"))
}

pub fn subcommand_get_hsm_groups_details() -> Command {
    Command::new("hsm-groups")
        // .visible_aliases(["h", "hg", "hsm", "hsmgrps"])
        .about("DEPRECATED - Please do not use this command.\nGet HSM groups details")
        .arg_required_else_help(true)
        .arg(arg!(<HSM_GROUP_NAME> "hsm group name"))
}

pub fn subcommand_get_images() -> Command {
    Command::new("images")
        // .visible_aliases(["i", "img", "imag", "image"])
        .about("Get image information")
        .arg(arg!(-i --id <VALUE> "Image ID"))
        .arg(
            arg!(-l --limit <VALUE> "Filter records to the <VALUE> most common number of images created")
                .value_parser(value_parser!(u8).range(1..)),
        )
        .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
}

pub fn subcommand_get_kernel_parameters() -> Command {
    Command::new("kernel-parameters")
        // .visible_aliases(["k", "kp", "kernel-params"])
        .about("Get kernel-parameters information")
        .arg(arg!(-x --xnames <VALUE> "Comma separated list of xnames to retreive the kernel parameters from.\neg: 'x1001c1s0b0n1,x1001c1s0b1n0'"))
        .arg(arg!(-H --"hsm-group" <VALUE> "List kernel parameters for all nodes in a HSM group name"))
        .arg(arg!(-f --filter <VALUE> "Comma separated list of kernel parameters to filter.\neg: 'console,bad_page,crashkernel,hugepagelist,root'"))
        .arg(arg!(-o --output <VALUE> "Output format.").value_parser(["table", "json"]).default_value("table"))
        .group(ArgGroup::new("hsm-group_or_xnames").args(["hsm-group", "xnames"]).required(true))
}

pub fn subcommand_get() -> Command {
    Command::new("get")
        // .visible_alias("g")
        .arg_required_else_help(true)
        .about("Get information from CSM system")
        .subcommand(subcommand_get_hw_components())
        .subcommand(subcommand_get_cfs_session())
        .subcommand(subcommand_get_cfs_configuration())
        .subcommand(subcommand_get_bos_template())
        // .subcommand(subcommand_get_node())
        .subcommand(subcommand_get_cluster_details())
        .subcommand(subcommand_get_node_details())
        .subcommand(subcommand_get_hsm_groups_details())
        .subcommand(subcommand_get_images())
        .subcommand(subcommand_get_kernel_parameters())
}

pub fn subcommand_apply_hw_configuration() -> Command {
    Command::new("hw-configuration")
        // .visible_alias("hw")
        .about("WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration")
        .arg_required_else_help(true)
        .subcommand(Command::new("cluster")
            // .visible_aliases(["c", "clstr"])
            .arg_required_else_help(true)
            .about("WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration")
            .arg(arg!(-P -- pattern <VALUE> "Hw pattern with keywords to fuzzy find hardware componented to assign to the cluster like <hw component name>:<hw component quantity>[:<hw component name>:<hw component quantity>]. Eg 'a100:12:epic:5' will update the nodes assigned to cluster 'zinal' with 4 nodes:\n - 3 nodes with 4 Nvidia gpus A100 and 1 epyc AMD cpu each\n - 1 node with 2 epyc AMD cpus").required(true))
            .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the pattern is applying to.").required(true))
            .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.").required(true))
            .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
            .arg(arg!(-c --"create-target-hsm-group" "If the target cluster name does not exist as HSM group, create it."))
            .arg(arg!(-d --"delete-empty-parent-hsm-group" "If the target HSM group is empty after this action, remove it."))
            .arg(arg!(-u --"unpin-nodes" "It will try to get any nodes available."))
        )
}

pub fn subcommand_apply_session() -> Command {
    Command::new("session")
        // .visible_aliases(["s", "se", "ses", "sess", "sssn"])
        .arg_required_else_help(true)
        .about("Runs the ansible script in local directory against HSM group or xnames.\nNote: the local repo must alrady exists in Shasta VCS")
        .arg(arg!(-n --name <VALUE> "Session name").required(true))
        .arg(arg!(-p --"playbook-name" <VALUE> "Playbook YAML file name. eg (site.yml)").default_value("site.yml"))
        .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").required(true)
            .value_parser(value_parser!(PathBuf)).value_hint(ValueHint::DirPath))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts"))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["0", "1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-P --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.").allow_hyphen_values(true))
        .arg(arg!(-l --"ansible-limit" <VALUE> "Ansible limit. Target xnames to the CFS session. Note: ansible-limit must be a subset of hsm-group if both parameters are provided").required(true))
        .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name"))
        .group(ArgGroup::new("hsm-group_or_ansible-limit").args(["hsm-group", "ansible-limit"]).required(true))
}

pub fn subcommand_apply_configuration() -> Command {
    Command::new("configuration")
        // .visible_aliases(["conf", "config"])
        .arg_required_else_help(true)
        .about("DEPRECATED - Please use 'manta apply sat-file' command instead.\nCreate a CFS configuration")
        // .about("Create a CFS configuration")
        .arg(arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.").value_parser(value_parser!(PathBuf)).required(true))
        .arg(arg!(-f --"values-file" <VALUES_FILE_PATH> "If the SAT file is a jinja2 template, then variables values can be expanded using this values file.").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-V --"values" <VALUES_PATH> ... "If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided."))
        // .arg(arg!(-t --tag <VALUE> "Tag added as a suffix in the CFS configuration name and CFS session name. If missing, then a default value will be used with timestamp"))
        .arg(arg!(-o --output <FORMAT> "Output format. If missing it will print output data in human redeable (table) format").value_parser(["json"]))
        // .arg(arg!(-n --name <VALUE> "Configuration name"))
        // .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path with a git repo and an ansible-playbook to configure the CFS image").value_parser(value_parser!(PathBuf)))
        // .group(ArgGroup::new("req_flags_name_repo-path").args(["name", "repo-path"]))
        // .group(ArgGroup::new("req_flag_file").arg("file"))
        .arg( arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group name linked to this configuration"))
}

pub fn subcommand_apply_template() -> Command {
    Command::new("template")
                    // .visible_aliases(["t", "temp", "tmpl"])
                    .arg_required_else_help(true)
                    .about("Create a new BOS session from an existing BOS sessiontemplate")
                    .arg(arg!(-n --name <VALUE> "Name of the Session"))
                    .arg(arg!(-o --operation <VALUE> "An operation to perform on Components in this Session. Boot Applies the Template to the Components and boots/reboots if necessary. Reboot Applies the Template to the Components; guarantees a reboot. Shutdown Power down Components that are on").value_parser(["reboot", "boot", "shutdown"]).default_value("reboot"))
                    .arg(arg!(-t --template <VALUE> "Name of the Session Template").required(true))
                    .arg(arg!(-l --limit <VALUE> "A comma-separated list of nodes, groups, or roles to which the Session will be limited. Components are treated as OR operations unless preceded by '&' for AND or '!' for NOT"))
                    .arg(arg!(-i --"include-disabled" <VALUE> "Set to include nodes that have been disabled as indicated in the Hardware State Manager (HSM)").action(ArgAction::SetTrue))
                    .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
}

pub fn subcommand_apply_ephemeral_environment() -> Command {
    Command::new("ephemeral-environment")
                    // .visible_aliases(["ee", "eph", "ephemeral"])
                    .arg_required_else_help(true)
                    .about("Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.")
                    // .arg(arg!(-b --block "Blocks this operation and won't return prompt until the ephemeral environment has been created."))
                    // .arg(arg!(-p --"public-ssh-key-id" <PUBLIC_SSH_ID> "Public ssh key id stored in Alps"))
                    .arg(arg!(-i --"image-id" <IMAGE_ID> "Image ID to use as a container image").required(true))
}

pub fn subcommand_apply_sat_file(/* hsm_group: Option<&String> */) -> Command {
    Command::new("sat-file")
        // .visible_alias("sat")
        .arg_required_else_help(true)
        .about("Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure.\nThe ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the 'debug_wait_time options'")
        // .about("Create a CFS configuration, a CFS image, a BOS sessiontemplate and a BOS session")
        .arg(arg!(-t --"sat-template-file" <VALUE> "SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.").value_parser(value_parser!(PathBuf)).required(true).value_hint(ValueHint::FilePath))
        .arg(arg!(-f --"values-file" <VALUE> "If the SAT file is a jinja2 template, then variables values can be expanded using this values file.").value_parser(value_parser!(PathBuf)).value_hint(ValueHint::FilePath))
        .arg(arg!(-V --"values" <VALUE> ... "If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided."))
        .arg(arg!(--"do-not-reboot" "By default, nodes will restart if SAT file builds an image which is assigned to the nodes through a BOS sessiontemplate, if you do not want to reboot the nodes, then use this flag. The SAT file will be processeed as usual and different elements created but the nodes won't reboot. This means, you will have to run 'manta apply template' command with the sessoin_template created'").action(ArgAction::SetTrue))
        .arg(arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command.\n1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.")
            .value_parser(["1", "2", "3", "4"])
            .num_args(1)
            // .require_equals(true)
            .default_value("2")
            .default_missing_value("2"))
        .arg(arg!(-P --"ansible-passthrough" <VALUE> "Additional parameters that are added to all Ansible calls for the session to create an image. This field is currently limited to the following Ansible parameters: \"--extra-vars\", \"--forks\", \"--skip-tags\", \"--start-at-task\", and \"--tags\". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.").allow_hyphen_values(true))
        .arg(arg!(-w --"watch-logs" "Watch logs. Hooks stdout to see container running ansible scripts").action(ArgAction::SetTrue))
        .arg(arg!(-i --"image-only" "Only process `configurations` and `images` sections in SAT file. The `session_templates` section will be ignored.").action(ArgAction::SetTrue))
        .arg(arg!(-s --"sessiontemplate-only" "Only process `configurations` and `session_templates` sections in SAT file. The `images` section will be ignored.").action(ArgAction::SetTrue))
        .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before processing SAT file. If need to pass a command with params. Use \" or \'.\neg: --pre-hook \"echo hello\""))
        .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run immediately after processing SAT file successfully. Use \" or \'.\neg: --post-hook \"echo hello\"."))
        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue).action(ArgAction::SetTrue))
    // .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
}

pub fn subcommand_update_nodes() -> Command {
    Command::new("nodes")
        // .visible_aliases(["n", "node", "nd"])
        .arg_required_else_help(true)
        .about("DEPRECATED - Please use 'manta apply boot nodes' command instead.\nUpdates boot and configuration of a group of nodes. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"boot-image" <CFS_CONFIG> "CFS configuration name related to the image to boot the nodes"))
        .arg(arg!(-d --"desired-configuration" <CFS_CONFIG> "CFS configuration name to configure the nodes after booting"))
        .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel boot parameters to assign to all cluster nodes while booting"))
        .arg(arg!(<XNAMES> "Comma separated list of xnames which boot image will be updated.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'"))
        .arg(arg!([HSM_GROUP_NAME] "hsm group name, this field should be used to validate the XNAMES belongs to HSM_GROUP_NAME"))
}

pub fn subcommand_apply_boot_nodes() -> Command {
    Command::new("nodes")
        // .visible_aliases(["n", "node"])
        .arg_required_else_help(true)
        .about("Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id.\neg:\nmanta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>")
        .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
        .arg(arg!(-b --"boot-image-configuration" <VALUE> "CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes"))
        .arg(arg!(-r --"runtime-configuration" <VALUE> "CFS configuration name to configure the nodes after booting"))
        .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel boot parameters to assign to the nodes while booting"))
        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue).action(ArgAction::SetTrue))
        .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
        .group(ArgGroup::new("boot-image_or_boot-config").args(["boot-image", "boot-image-configuration"]))
        .arg(arg!(<XNAMES> "Comma separated list of xnames which boot image will be updated.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'"))
}

pub fn subcommand_apply_boot_cluster() -> Command {
    Command::new("cluster")
        // .visible_alias("c")
        .arg_required_else_help(true)
        .about("Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id.\neg:\nmanta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>")
        .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
        .arg(arg!(-b --"boot-image-configuration" <VALUE> "CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes"))
        .arg(arg!(-r --"runtime-configuration" <VALUE> "CFS configuration name to configure the nodes after booting"))
        .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel boot parameters to assign to all cluster nodes while booting"))
        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue).action(ArgAction::SetTrue))
        .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
        .group(ArgGroup::new("boot-image_or_boot-config").args(["boot-image", "boot-image-configuration"]))
        .arg(arg!(<CLUSTER_NAME> "Cluster name").required(true))
}

pub fn subcommand_migrate_backup() -> Command {
    Command::new("backup")
        // .visible_aliases(["mb"])
        .arg_required_else_help(true)
        .about("Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.")
        .arg(arg!(-b --"bos" <SESSIONTEMPLATE> "BOS Sessiontemplate to use to derive CFS, boot parameters and HSM group"))
        .arg(arg!(-d --"destination" <FOLDER> "Destination folder to store the backup on"))
        .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before doing the backup. If need to pass a command with params. Use \" or \'.\neg: --pre-hook \"echo hello\""))
        .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run immediately after the backup is completed successfully. Use \" or \'.\neg: --post-hook \"echo hello\"."))
}

pub fn subcommand_migrate_restore() -> Command {
    Command::new("restore")
        // .visible_aliases(["mr"])
        .arg_required_else_help(true)
        .about("MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted.\neg:\nmanta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>")
        .arg(arg!(-b --"bos-file" <BOS_session_template_file> "BOS session template of the cluster backed previously with migrate backup").value_hint(ValueHint::FilePath))
        .arg(arg!(-c --"cfs-file" <CFS_configuration_file> "CFS session template of the cluster backed previously with migrate backup").value_hint(ValueHint::FilePath))
        .arg(arg!(-j --"hsm-file" <HSM_group_description_file> "HSM group description file of the cluster backed previously with migrate backup").value_hint(ValueHint::FilePath))
        .arg(arg!(-m --"ims-file" <IMS_file> "IMS file backed previously with migrate backup").value_hint(ValueHint::FilePath))
        .arg(arg!(-i --"image-dir" <IMAGE_path> "Path where the image files are stored.").value_hint(ValueHint::DirPath))
        .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before doing the backup. If need to pass a command with params. Use \" or \'.\neg: --pre-hook \"echo hello\""))
        .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run immediately after the backup is completed successfully. Use \" or \'.\neg: --pre-hook \"echo hello\"."))
}

pub fn subcommand_power() -> Command {
    Command::new("power")
        // .visible_aliases(["p", "pwr"])
        .arg_required_else_help(true)
        .about("Command to submit commands related to cluster/node power management")
        .subcommand(
            Command::new("on")
                .arg_required_else_help(true)
                .about("Command to power on cluster/node")
                .subcommand(
                    Command::new("cluster")
                        // .visible_aliases(["c", "clstr"])
                        .arg_required_else_help(true)
                        .about("Command to power on all nodes in a cluster")
                        .arg(arg!(-R --reason <TEXT> "reason to power on"))
                        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                        .arg(arg!(-o --output <FORMAT> "Output format.").value_parser(["table", "json"]).default_value("table"))
                        .arg(arg!(<CLUSTER_NAME> "Cluster name")),
                )
                .subcommand(
                    Command::new("nodes")
                        // .visible_aliases(["n", "node"])
                        .arg_required_else_help(true)
                        .about("Command to power on a group of nodes.\neg: 'x1001c1s0b0n1,x1001c1s0b1n0'")
                        .arg(arg!(-r --regex "Input nodes in regex format.").action(ArgAction::SetTrue))
                        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                        .arg(arg!(-o --output <FORMAT> "Output format.").value_parser(["table", "json"]).default_value("table"))
                        .arg(arg!(<VALUE> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'")),
                ),
        )
        .subcommand(
            Command::new("off")
                .arg_required_else_help(true)
                .about("Command to power off cluster/node")
                .subcommand(
                    Command::new("cluster")
                        // .visible_aliases(["c", "clstr"])
                        .arg_required_else_help(true)
                        .about("Command to power off all nodes in a cluster")
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-R --reason <TEXT> "reason to power off"))
                        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                        .arg(arg!(-o --output <FORMAT> "Output format.").value_parser(["table", "json"]).default_value("table"))
                        .arg(arg!(<CLUSTER_NAME> "Cluster name")),
                )
                .subcommand(
                    Command::new("nodes")
                        // .visible_aliases(["n", "node"])
                        .arg_required_else_help(true)
                        .about("Command to power off a group of nodes.\neg: 'x1001c1s0b0n1,x1001c1s0b1n0'")
                        .arg(arg!(-n --nodes <VALUE> "Comma separated list of nodes"))
                        .arg(arg!(-r --regex "Input nodes in regex format.\neg 'x1003c1s.*'").action(ArgAction::SetTrue))
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                        .arg(arg!(-o --output <FORMAT> "Output format.").value_parser(["table", "json"]).default_value("table"))
                        .arg(arg!(<VALUE> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'")),
                ),
        )
        .subcommand(
            Command::new("reset")
                .arg_required_else_help(true)
                .about("Command to power reset cluster/node")
                .subcommand(
                    Command::new("cluster")
                        // .visible_aliases(["c", "clstr"])
                        .arg_required_else_help(true)
                        .about("Command to power reset all nodes in a cluster")
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                        .arg(arg!(-o --output <FORMAT> "Output format.").value_parser(["table", "json"]).default_value("table"))
                        .arg(arg!(-r --reason <TEXT> "reason to power reset"))
                        .arg(arg!(<CLUSTER_NAME> "Cluster name")),
                )
                .subcommand(
                    Command::new("nodes")
                        // .visible_aliases(["n", "node"])
                        .arg_required_else_help(true)
                        .about("Command to power reset a group of nodes.\neg: 'x1001c1s0b0n1,x1001c1s0b1n0'")
                        .arg(arg!(-r --regex "Input nodes in regex format.").action(ArgAction::SetTrue))
                        .arg(arg!(-f --force "force").action(ArgAction::SetTrue))
                        .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                        .arg(arg!(-o --output <FORMAT> "Output format.").value_parser(["table", "json"]).default_value("table"))
                        .arg(arg!(<VALUE> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'")),
                ),
        )
}

pub fn subcommand_log() -> Command {
    Command::new("log")
        // .visible_alias("l")
        .about("get cfs session logs")
        .arg(arg!([SESSION_NAME] "show logs related to session name"))
        .arg(arg!(-c --cluster <CLUSTERNAME> "Show logs most recent CFS session logs created for cluster."))
        .arg(arg!(-n --node <NODENAME> "Show logs most recent CFS session logs created for a node."))
        .group(ArgGroup::new("cluster_or_session_name").args(["cluster", "SESSION_NAME"]))
}

pub fn subcommand_validate_local_repo() -> Command {
    Command::new("validate-local-repo")
        // .visible_alias("vlr")
        .about("Check all tags and HEAD information related to a local repo exists in Gitea")
        .arg(arg!(-r --"repo-path" <REPO_PATH> ... "Repo path. The path to a local a git repo related to a CFS configuration layer to test against Gitea").required(true))
}

pub fn subcommand_add() -> Command {
    Command::new("add")
            .arg_required_else_help(true)
            .about("Add hw components to cluster")
            .subcommand(
                Command::new("group")
                // .visible_alias("g")
                .about("add/create new group")
                .arg_required_else_help(true)
                .arg(arg!(-l --label <VALUE> "group name").required(true))
                .arg(arg!(-n --nodes <VALUE> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'"))
                .arg(arg!(-r --regex "Input nodes in regex format.").action(ArgAction::SetTrue))
                .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                .arg(arg!(-D --"dryrun" "No changes applied to the system.").action(ArgAction::SetTrue))
            )
            .subcommand(Command::new("hw-component")
                // .visible_alias("hw")
                .arg_required_else_help(true)
                .about("WIP - Add hw components from a cluster")
                .arg(arg!(-P --pattern <PATTERN> "Pattern"))
                .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Target cluster name. This is the name of the cluster the pattern is applying to."))
                .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster."))
                .arg(arg!(-x --"no-dryrun" "No dry-run, actually change the status of the system. The default for this command is a dry-run."))
                .arg(arg!(-c --"create-hsm-group" "If the target cluster name does not exist as HSM group, create it."))

            )
            .subcommand(
                Command::new("kernel-parameters")
                // .visible_alias("kp")
                .arg_required_else_help(true)
                .about("Delete kernel parameters")
                .arg(arg!(-x --xnames <XNAMES> "Comma separated list of nodes to set runtime configuration.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'"))
                .arg(arg!(-H --"hsm-group" <HSM_GROUP> "Cluster to set runtime configuration"))
                .arg(arg!(-y --"assume-yes" "Automatic yes to prompts; assume 'yes' as answer to all prompts and run non-interactively.").action(ArgAction::SetTrue))
                .arg(arg!(<VALUE> "Space separated list of kernel parameters. Eg: console,bad_page,crashkernel,hugepagelist,quiet"))
                .group(
                    ArgGroup::new("cluster_or_xnames")
                        .args(["hsm-group", "xnames"])
                        .required(true),
                )
            )
}

pub fn subcommand_apply() -> Command {
    Command::new("apply")
        // .visible_alias("a")
        .arg_required_else_help(true)
        .about("Make changes to Shasta system")
        .subcommand(subcommand_apply_hw_configuration())
        .subcommand(subcommand_apply_configuration())
        .subcommand(subcommand_apply_sat_file(/* hsm_group */))
        .subcommand(
            Command::new("boot")
                // .visible_alias("b")
                .arg_required_else_help(true)
                .about("Change boot operations")
                .subcommand(subcommand_apply_boot_nodes())
                .subcommand(subcommand_apply_boot_cluster()),
        )
        .subcommand(subcommand_apply_session())
        .subcommand(subcommand_apply_ephemeral_environment())
        .subcommand(subcommand_apply_template())
}

pub fn subcommand_migrate() -> Command {
    Command::new("migrate")
                // .visible_alias("m")
                .arg_required_else_help(true)
                .subcommand(Command::new("vCluster")
                    // .visible_aliases(["c", "clstr"])
                    .about("WIP - Migrate vCluster")
                    .subcommand(subcommand_migrate_backup())
                    .subcommand(subcommand_migrate_restore())
                )
                .subcommand(Command::new("nodes")
                    // .visible_aliases(["n", "node"])
                    .arg_required_else_help(true)
                    .about("Migrate nodes across vClusters")
                    .arg(arg!(-f --from <VALUE> "The name of the source vCluster from which the compute nodes will be moved."))
                    .arg(arg!(-t --to <VALUE> "The name of the target vCluster to which the compute nodes will be moved.").required(true))
                    .arg(arg!(<XNAMES> "Comma separated list of xnames to add to a cluster.\neg: 'x1003c1s7b0n0,x1003c1s7b0n1,x1003c1s7b1n0'").required(true))
                    .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
                )
}

pub fn subcommand_console() -> Command {
    Command::new("console")
        // .visible_aliases(["c", "con", "cons", "conso"])
        .arg_required_else_help(true)
        .about("Opens an interective session to a node or CFS session ansible target container")
        .subcommand(
            Command::new("node")
                // .visible_alias("n")
                .about("Connects to a node's console")
                .arg(arg!(<XNAME> "node xname").required(true)),
        )
        .subcommand(
            Command::new("target-ansible")
                // .visible_aliases(["t", "ta", "target", "ansible"])
                .arg_required_else_help(true)
                .about(
                    "Opens an interactive session to the ansible target container of a CFS session",
                )
                .arg(arg!(<SESSION_NAME> "CFS session name").required(true)),
        )
}

pub fn subcommand_add_nodes_to_groups() -> Command {
    Command::new("add-nodes-to-groups")
            // .visible_aliases(["ag"])
            .about("Add nodes to a list of groups")
            .arg(arg!(-g --group <VALUE> "HSM group to assign the nodes to"))
            .arg(arg!(-n --nodes <VALUE> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'"))
            .arg(arg!(-r --regex "Input nodes in regex format.").action(ArgAction::SetTrue))
            .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
}

pub fn subcommand_remove_nodes_from_groups() -> Command {
    Command::new("remove-nodes-from-groups")
           // .visible_aliases(["rg"])
           .about("Remove nodes from groups")
           .arg(arg!(-g --group <VALUE> "HSM group to remove the nodes from"))
            .arg(arg!(-n --nodes <VALUE> "List of xnames or nids.\neg 'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0' or 'nid001313,nid001314'\n Host list also accepted eg 'x1003c1s7b0n[0-1],x1003c1s7b1n0' or 'nid00131[0-9]'"))
           .arg(arg!(-r --regex "Input nodes in regex format.").action(ArgAction::SetTrue))
           .arg(arg!(-d --"dry-run" "Simulates the execution of the command without making any actual changes.").action(ArgAction::SetTrue))
}
