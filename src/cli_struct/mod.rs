use clap::{Parser, Subcommand, Args, ArgGroup};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: MainSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MainSubcommand {
    /// Get information from Shasta system
    Get(GetArgs),
    /// Make changes to Shata clusters/nodes 
    Apply(ApplyArgs),
    /// Print session logs
    Log(LogArgs),
    /// WIP Access node console
    Console(ConsoleArg)
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct GetArgs {
    #[clap(subcommand)]
    pub main_get_subcommand: GetSubcommand,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ApplyArgs {
    #[clap(subcommand)]
    pub main_apply_subcommand: ApplySubcommand,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct LogArgs {
    /// Session name
    #[clap(short, long, value_parser)]
    pub session_name: String,
    #[clap(short, long, value_parser)]
    /// Layer id to target. 0 => ansible-0; 1 => ansible-1 ...
    pub layer_id: Option<u8>,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ConsoleArg {
    /// xname of the node to connect to
    #[clap(short, long, value_parser)]
    pub xname: String,
}

#[derive(Debug, Subcommand)]
pub enum GetSubcommand {
    /// Get CFS configuration details
    Configuration(GetConfigurationOptions),
    /// Get CFS session details
    Session(GetSessionOptions),
    /// Get BOS template details
    Template(GetTemplateOptions),
    /// Get HSM nodes
    Node(GetNodeOptions),
}

#[derive(Debug, Subcommand)]
pub enum ApplySubcommand {
    /// Create new CFS session
    Session(ApplySessionOptions),
    /// Restart Power on/off a node
    Node(ApplyNodeArgs)
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ApplyNodeArgs {
    #[clap(subcommand)]
    pub main_apply_node_subcommand: ApplyNodeSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ApplyNodeSubcommand {
    /// Start a node
    On(ApplyNodeOnOptions),
    /// Shutdown a node
    Off(ApplyNodeOffOptions),
    /// Restart a node
    Reset(ApplyNodeResetOptions)
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["name", "cluster_name"]),))]
#[clap(group(ArgGroup::new("config-limit").args(&["most_recent", "limit_number"]),))]
pub struct GetConfigurationOptions {
    /// Cfs configuration name
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    pub most_recent: bool,
    /// Number of CFS configurations to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    pub limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("session-type").args(&["name", "cluster_name"]),))]
#[clap(group(ArgGroup::new("session-limit").args(&["most_recent", "limit_number"]),))]
pub struct GetSessionOptions {
    /// Cfs session name
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    pub most_recent: bool,
    /// Number of CFS configurations to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    pub limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["name", "cluster_name"]),))]
#[clap(group(ArgGroup::new("config-limit").args(&["most_recent", "limit_number"]),))]
pub struct GetTemplateOptions {
    /// Bos template name
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    pub most_recent: bool,
    /// Number of BOS templates to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    pub limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct GetNodeOptions {
    /// Cluster name
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ApplySessionOptions {
    /// Session name
    #[clap(short, long, value_parser)]
    pub session_name: String,
    /// Repo path. The path with a git repo and an ansible-playbook to configure the CFS image.
    #[clap(short, long, value_parser)]
    pub repo_path: Vec<String>,
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
#[clap(group(ArgGroup::new("config-type").args(&["xnames", "cluster_name"]).required(true),))]
pub struct ApplyNodeOffOptions {
    /// Reason to shutdown
    #[clap(short, long, value_parser)]
    pub reason: Option<String>,   
    /// List of xnames to power off
    #[clap(short, long, value_parser)]
    pub xnames: Option<String>,
    /// All nodes belonging to this cluster will power off
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
    /// Force node operation
    #[clap(short, long, value_parser)]
    pub force: bool
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["xnames", "cluster_name"]).required(true),))]
pub struct ApplyNodeOnOptions {
    /// Reason to power on
    #[clap(short, long, value_parser)]
    pub reason: Option<String>, 
    /// List of xnames to power on
    #[clap(short, long, value_parser)]
    pub xnames:Option<String>,
    /// All nodes belonging to this cluster will power on
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["xnames", "cluster_name"]).required(true),))]
pub struct ApplyNodeResetOptions {
    /// Reason to reboot
    #[clap(short, long, value_parser)]
    pub reason: Option<String>, 
    /// List of xnames to reboot
    #[clap(short, long, value_parser)]
    pub xnames: Option<String>,
    /// All nodes belonging to this cluster will reboot
    #[clap(short, long, value_parser)]
    pub cluster_name: Option<String>,
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