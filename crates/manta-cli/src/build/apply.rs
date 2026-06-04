//! Clap definitions for `manta apply *` subcommands.

use clap::{ArgAction, ArgGroup, Command, ValueHint, arg, value_parser};
use std::path::PathBuf;

use super::run::add_run_session_args;
use super::{HOSTLIST_HELP, output_flag, output_flag_long_only};

/// Attach the hardware-rescale argument set to a clap `Command`.
/// Shared between the canonical `apply hardware group` and the
/// deprecated `apply hardware cluster` paths so they stay in lockstep.
///
/// `--target-cluster`/`--parent-cluster` are renamed to
/// `--target-group`/`--parent-group`; old flag names retained as
/// visible aliases. Same for the create/delete lifecycle flags.
fn add_apply_hw_group_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(
      arg!(-P -- pattern <PATTERN> "Hardware pattern: <component>:<qty>[:<component>:<qty>...].\neg: 'a100:12:epyc:5'")
        .required(true),
    )
    .arg(
      arg!(-t --"target-group" <TARGET_GROUP_NAME> "Group to rescale")
        .required(true)
        .visible_alias("target-cluster"),
    )
    .arg(
      arg!(-p --"parent-group" <PARENT_GROUP_NAME> "Group that donates or receives the redistributed nodes")
        .required(true)
        .visible_alias("parent-cluster"),
    )
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(
      arg!(-c --"create-target-group" "Create the target group if it does not exist")
        .visible_alias("create-target-hsm-group"),
    )
    .arg(
      arg!(-D --"delete-empty-parent-group" "Delete the parent group if empty after this operation")
        .visible_alias("delete-empty-parent-hsm-group"),
    )
    .arg(arg!(-u --"unpin-nodes" "Allow any available nodes to be selected"))
    .arg(output_flag())
}

pub fn subcommand_apply_hw_configuration() -> Command {
  Command::new("hardware")
    .about("[experimental] Rescale a group's hardware allocation")
    .arg_required_else_help(true)
    .subcommand(
      add_apply_hw_group_args(Command::new("group"))
        .about("[experimental] Rescale a group's hardware allocation")
        .long_about(
          "[experimental] Upscale or downscale a group by specifying a hardware component pattern.\n\n\
          If the group does not exist it will be created; otherwise its node assignment is updated.\n\n\
          Pattern format: <component>:<quantity>[:<component>:<quantity>...]\n\
          eg: 'a100:12:epyc:5'  — assign nodes with 12 A100 GPUs and 5 EPYC CPUs total",
        ),
    )
    .subcommand(
      add_apply_hw_group_args(Command::new("cluster"))
        .about("[DEPRECATED] Use 'manta apply hardware group' instead"),
    )
}

pub fn subcommand_apply_session() -> Command {
  add_run_session_args(Command::new("session"))
    .about("[DEPRECATED] Use 'manta run session' instead")
    .long_about(
      "Create and run a configuration session from a local git repo.\n\n\
      The repo must already exist in the system's VCS. The session runs \
      the specified Ansible playbook against the target nodes or group.\n\n\
      DEPRECATED: this command has moved to `manta run session`. The old \
      path keeps working for one release.",
    )
}

pub fn subcommand_apply_configuration() -> Command {
  Command::new("configuration")
    .arg_required_else_help(true)
    .about("Create a configuration (deprecated — use 'apply sat-file')")
    .arg(
      arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file path")
        .value_parser(value_parser!(PathBuf))
        .required(true),
    )
    .arg(
      arg!(-f --"values-file" <VALUES_FILE_PATH> "Values file for SAT jinja2 templates")
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(arg!(-V --"values" <VALUES> ... "Inline values for SAT jinja2 templates (overrides --values-file)"))
    .arg(arg!(-o --output <FORMAT> "Output format").value_parser(["json"]))
    .arg(arg!(-H --group <GROUP_NAME> "Node group name").visible_alias("hsm-group"))
}

pub fn subcommand_apply_template() -> Command {
  Command::new("template")
    .arg_required_else_help(true)
    .about("Boot nodes using an existing session template")
    .arg(arg!(-n --name <VALUE> "Name of the boot session to create"))
    .arg(
      arg!(-o --operation <VALUE> "Boot operation to perform")
        .value_parser(["reboot", "boot", "shutdown"])
        .default_value("reboot"),
    )
    .arg(arg!(-t --template <VALUE> "Session template name").required(true))
    .arg(
      arg!(-l --limit <VALUE>
        "Limit to specific nodes, groups, or roles (OR by default; prefix with '&' for AND or '!' for NOT)")
        .required(true),
    )
    .arg(
      arg!(-i --"include-disabled" "Include nodes marked as disabled in the hardware state manager")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
}

pub fn subcommand_apply_ephemeral_environment() -> Command {
  Command::new("ephemeral-environment")
    .arg_required_else_help(true)
    .about("Launch an ephemeral SSH environment from an image")
    .long_about(
      "Launch an ephemeral SSH environment from an image.\n\n\
      Returns an SSH hostname once the environment is ready (usually within a few seconds).",
    )
    .arg(arg!(-i --"image-id" <IMAGE_ID> "Image ID to use").required(true))
}

pub fn subcommand_apply_sat_file() -> Command {
  Command::new("sat-file")
    .arg_required_else_help(true)
    .about("Process a SAT file to create configurations, images, and session templates")
    .long_about(
      "Process a SAT file containing up to three sections:\n\
      \n\
      - `configurations`:   configurations to create\n\
      - `images`:           images to build from those configurations\n\
      - `session_templates`: session templates to create\n\
      \n\
      Use --image-only to process only configurations and images.\n\
      Use --sessiontemplate-only to process only configurations and session templates.",
    )
    .arg(
      arg!(-t --"sat-template-file" <FILE> "SAT file path (may be a jinja2 template)")
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(-f --"values-file" <FILE> "Values file to expand jinja2 variables in the SAT file")
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::FilePath),
    )
    .arg(arg!(-V --"values" <VALUE> ... "Inline values to expand jinja2 variables (overrides --values-file)"))
    .arg(arg!(--"reboot" "Reboot nodes after applying session templates").action(ArgAction::SetTrue))
    .arg(
      arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)")
        .value_parser(["1", "2", "3", "4"])
        .num_args(1)
        .default_value("2")
        .default_missing_value("2"),
    )
    .arg(
      arg!(-P --"ansible-passthrough" <VALUE>
        "Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)")
        .allow_hyphen_values(true),
    )
    .arg(
      arg!(-o --"overwrite-configuration" "Overwrite an existing configuration with the same name")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-w --"watch-logs" "Stream session logs to stdout").action(ArgAction::SetTrue))
    .arg(arg!(-T --timestamps "Show log timestamps").action(ArgAction::SetTrue))
    .arg(
      arg!(-i --"image-only" "Process only the `configurations` and `images` sections")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-s --"sessiontemplate-only" "Process only the `configurations` and `session_templates` sections")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before processing.\neg: --pre-hook \"echo hello\""))
    .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run after successful processing.\neg: --post-hook \"echo hello\""))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(output_flag_long_only())
}

pub fn subcommand_apply_boot_nodes() -> Command {
  Command::new("nodes")
    .arg_required_else_help(true)
    .about("Update boot parameters for a set of nodes")
    .long_about(
      "Update the boot parameters (image, runtime configuration, and kernel parameters) for a set of nodes.\n\n\
      The boot image can be specified by image ID or by the configuration name used to build it \
      (the most recent matching image is used).\n\n\
      eg:\n  \
      manta apply boot nodes \\\n    \
        --boot-image-configuration <config-name> \\\n    \
        --runtime-configuration <config-name> <nodes>",
    )
    .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
    .arg(
      arg!(-b --"boot-image-configuration" <NAME>
        "Configuration name used to build the boot image (uses the most recent matching image)"),
    )
    .arg(arg!(-r --"runtime-configuration" <NAME> "Configuration to apply to nodes after booting"))
    .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel parameters to assign to the nodes"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(
      arg!(--"do-not-reboot" "Suppress the automatic reboot after updating boot parameters")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .group(
      ArgGroup::new("boot-image_or_boot-config")
        .args(["boot-image", "boot-image-configuration"]),
    )
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP))
    .arg(output_flag())
}

/// Attach the per-group boot-parameter argument set. Shared between
/// the canonical `apply boot group` and the deprecated
/// `apply boot cluster` paths so they stay in lockstep.
fn add_apply_boot_group_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
    .arg(
      arg!(-b --"boot-image-configuration" <NAME>
        "Configuration name used to build the boot image (uses the most recent matching image)"),
    )
    .arg(arg!(-r --"runtime-configuration" <NAME> "Configuration to apply to nodes after booting"))
    .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel parameters to assign to all group members"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(
      arg!(--"do-not-reboot" "Suppress the automatic reboot after updating boot parameters")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .group(
      ArgGroup::new("boot-image_or_boot-config")
        .args(["boot-image", "boot-image-configuration"]),
    )
    // ID preserved as "CLUSTER_NAME" for handler compatibility
    .arg(arg!(<CLUSTER_NAME> "Group name").required(true))
    .arg(output_flag())
}

pub fn subcommand_apply_boot_cluster_deprecated() -> Command {
  add_apply_boot_group_args(Command::new("cluster"))
    .about("[DEPRECATED] Use 'manta apply boot group' instead")
    .long_about(
      "Update the boot parameters (image, runtime configuration, and kernel parameters) for all nodes in a group.\n\n\
      DEPRECATED: this command has been renamed to `manta apply boot group`. The old name keeps working for one release.",
    )
}

/// Canonical replacement for `apply boot cluster`.
pub fn subcommand_apply_boot_group() -> Command {
  add_apply_boot_group_args(Command::new("group"))
    .about("Update boot parameters for all nodes in a group")
    .long_about(
      "Update the boot parameters (image, runtime configuration, and kernel parameters) for all nodes in a group.\n\n\
      The boot image can be specified by image ID or by the configuration name used to build it \
      (the most recent matching image is used).\n\n\
      eg:\n  \
      manta apply boot group \\\n    \
        --boot-image-configuration <config-name> \\\n    \
        --runtime-configuration <config-name> <group-name>",
    )
}

pub fn subcommand_apply_kernel_parameters() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(
      arg!(-H --group <GROUP_NAME> "Replace kernel parameters on every node in this group")
        .visible_alias("hsm-group"),
    )
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(--"do-not-reboot" "Do not reboot nodes after applying changes").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Space-separated kernel parameters to apply.\neg: bos_update_frequency=4h console=ttyS0,115200 crashkernel=512M")
        .value_name("PARAMS"),
    )
    .group(
      ArgGroup::new("cluster_or_nodes")
        .args(["group", "nodes"])
        .required(true),
    )
    .arg(output_flag())
}

pub fn subcommand_apply() -> Command {
  Command::new("apply")
    .arg_required_else_help(true)
    .about("Roll out configurations, images, session templates, boot/kernel parameters, and hardware rescaling")
    .subcommand(subcommand_apply_hw_configuration())
    .subcommand(subcommand_apply_configuration())
    .subcommand(subcommand_apply_sat_file())
    .subcommand(
      Command::new("boot")
        .arg_required_else_help(true)
        .about("Update boot parameters and runtime configuration")
        .subcommand(subcommand_apply_boot_nodes())
        .subcommand(subcommand_apply_boot_group())
        .subcommand(subcommand_apply_boot_cluster_deprecated()),
    )
    .subcommand(subcommand_apply_kernel_parameters())
    .subcommand(subcommand_apply_session())
    .subcommand(subcommand_apply_ephemeral_environment())
    .subcommand(subcommand_apply_template())
}
