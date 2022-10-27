use clap::{Parser, Subcommand, Args, ArgGroup};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: MainSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MainSubcommand {
    /// Get information from Shasta system
    Get(MainGetArgs),
    /// Make changes to Shata clusters/nodes 
    Apply(MainApplyArgs),
    /// Print session logs
    Log(MainLogArgs),
    /// WIP Access node console
    Console(MainConsoleArg)
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainGetArgs {
    #[clap(subcommand)]
    pub main_get_subcommand: MainGetSubcommand,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainApplyArgs {
    #[clap(subcommand)]
    pub main_apply_subcommand: MainApplySubcommand,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainLogArgs {
    /// Session name
    #[clap(short, long, value_parser)]
    pub session_name: String,
    #[clap(short, long, value_parser)]
    /// Layer id to target. 0 => ansible-0; 1 => ansible-1 ...
    pub layer_id: u8,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainConsoleArg {
    /// xname of the node to connect to
    #[clap(short, long, value_parser)]
    pub xname: String,
}

#[derive(Debug, Subcommand)]
pub enum MainGetSubcommand {
    /// Get configuration details
    Configuration(MainGetConfigurationOptions),
    /// Get session details
    Session(MainGetSessionOptions),
}

#[derive(Debug, Subcommand)]
pub enum MainApplySubcommand {
    /// Create new CFS session
    Session(ApplySessionOptions),
    /// Restart Power on/off a node
    Node(MainApplyNodeArgs)
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainApplyNodeArgs {
    #[clap(subcommand)]
    pub main_apply_node_subcommand: MainApplyNodeSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MainApplyNodeSubcommand {
    /// Start a node
    On(MainApplyNodeOnOptions),
    /// Shutdown a node
    Off(MainApplyNodeOffOptions),
    /// Restart a node
    Reset(MainApplyNodeResetOptions)
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["name", "cluster_name"]),))]
#[clap(group(ArgGroup::new("config-limit").args(&["most_recent", "limit_number"]),))]
pub struct MainGetConfigurationOptions {
    /// Configuration name
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    pub most_recent: bool,
    /// Number of configurations to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    pub limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("session-type").args(&["name", "cluster_name"]),))]
#[clap(group(ArgGroup::new("session-limit").args(&["most_recent", "limit_number"]),))]
pub struct MainGetSessionOptions {
    /// Session name
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    pub most_recent: bool,
    /// Number of configurations to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    pub limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ApplySessionOptions {
    /// Session name
    #[clap(short, long, value_parser)]
    pub session_name: Option<String>,
    /// Repo path. The path with a git repo and an ansible-playbook to configure the CFS image.
    #[clap(short, long, value_parser)]
    pub repo_path: String,
    /// Watch logs. Hooks stdout to aee container running ansible scripts
    #[clap(short, long, value_parser)]
    pub watch_logs: bool,
    /// Ansible limit
    #[clap(short, long, value_parser)]
    pub ansible_limit: String,
    /// Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 
    /// 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.
    #[clap(short = 'v', long, value_parser, default_value_t = 2)]
    pub ansible_verbosity: u8
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainApplyNodeOffOptions {
    /// Reason to shutdown
    #[clap(short, long, value_parser)]
    pub reason: Option<String>,   
    /// List of xnames to power off
    #[clap(short, long, value_parser)]
    pub xnames: String,
    /// Force node operation
    #[clap(short, long, value_parser)]
    pub force: bool
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainApplyNodeOnOptions {
    /// Reason to power on
    #[clap(short, long, value_parser)]
    pub reason: Option<String>, 
    /// List of xnames to power on
    #[clap(short, long, value_parser)]
    pub xnames: String,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct MainApplyNodeResetOptions {
    /// Reason to reboot
    #[clap(short, long, value_parser)]
    pub reason: Option<String>, 
    /// List of xnames to reboot
    #[clap(short, long, value_parser)]
    pub xnames: String,
    /// Force node operation
    #[clap(short, long, value_parser)]
    pub force: bool
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Cluster {
    #[clap(short, long, value_parser)]
    /// Cluster name
    pub name: Option<String>,
}