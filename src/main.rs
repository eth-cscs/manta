pub mod cfs_utils;
pub mod auth;
// pub mod k8s_programmatic_client;
mod shasta_cfs_configuration;
mod shasta_cfs_session;
mod shasta_cfs_session_logs;
mod shasta_vcs;
mod manta_cfs;

// use std::process;

use clap::{Args, ArgGroup, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]struct Cli {
    #[clap(subcommand)]
    command: Verb,
}

#[derive(Debug, Subcommand)]
enum Verb {
    // NOTE: clap uses '///' comments as a command description
    /// Get shasta objects data sorted by creation or update time in desc order
    Get(Get),
    // /// Create new shasta object (SESSION|CONFIGURATION)
    // Create(Create),
    /// Print session logs
    Log(Log),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Get {
    #[clap(subcommand)]
    shasta_object: ShastaObject,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Create {
    #[clap(subcommand)]
    shasta_object: ShastaObject,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Log {
    /// Session name
    #[clap(short, long, value_parser)]
    session_name: String,
    #[clap(short, long, value_parser)]
    /// Layer id to target. 0 => ansible-0; 1 => ansible-1 ...
    layer_id: u8,
}

#[derive(Debug, Subcommand)]
enum ShastaObject {
    /// Manage configuration details
    Configuration(Configuration),
    /// Manage session details
    Session(Session),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["name", "cluster-name"]),))]
#[clap(group(ArgGroup::new("config-limit").args(&["most-recent", "limit-number"]),))]
struct Configuration {
    /// Configuration name
    #[clap(short, long, value_parser)]
    name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    most_recent: bool,
    /// Number of configurations to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("session-type").args(&["name", "cluster-name"]),))]
#[clap(group(ArgGroup::new("session-limit").args(&["most-recent", "limit-number"]),))]struct Session {
    /// Session name
    #[clap(short, long, value_parser)]
    name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    cluster_name: Option<String>,
    /// Most recent (equivalent to --limit 1)
    #[clap(short, long, action)]
    most_recent: bool,
    /// Number of configurations to show on screen
    #[clap(short, long, action, value_parser = clap::value_parser!(u8).range(1..))]
    limit_number: Option<u8>
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Cluster {
    #[clap(short, long, value_parser)]
    /// Cluster name
    name: Option<String>,
}

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {

    // Init logger
    env_logger::init();

    let cluster_name;
    let most_recent;
    let configuration_name;
    let session_name;
    let limit_number;
    let logging_session_name;
    let layer_id;
    let shasta_token;
    let gitea_token;
    let shasta_base_url = "https://api-gw-service-nmn.local/apis";

    let shasta_admin_pwd = std::env::var("SHASTA_ADMIN_PWD").unwrap();

    let shasta_token_resp = auth::auth(&shasta_admin_pwd).await?;

    shasta_token = shasta_token_resp["access_token"].as_str().unwrap();
    gitea_token = std::env::var("GITEA_TOKEN").unwrap();

    // let resp = cfs::check_cfs_health(shasta_token, shasta_base_url).await?;

    // Parse input params
    let args = Cli::parse();

    // Process input params
    match args.command {
        Verb::Get(get_cmd) => {
            match get_cmd.shasta_object {
                ShastaObject::Configuration(configuration) => {

                    configuration_name = configuration.name;
                    cluster_name = configuration.cluster_name;
                    most_recent = configuration.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = configuration.limit_number;
                    }

                    // Get CFS configurations
                    let cfs_configurations = crate::shasta_cfs_configuration::http_client::get(shasta_token, shasta_base_url, &cluster_name, &configuration_name, &limit_number).await?;

                    if cfs_configurations.is_empty() {
                        log::info!("No CFS configuration found!");
                        return Ok(())
                    } else if cfs_configurations.len() == 1 {

                        let most_recent_cfs_configuration = &cfs_configurations[0];

                        log::info!("{}", manta_cfs::configuration::create(most_recent_cfs_configuration, gitea_token).await?);
                    } else {
                        cfs_utils::print_cfs_configurations(&cfs_configurations);
                    }
                },
                ShastaObject::Session(session) => {

                    session_name = session.name;
                    cluster_name = session.cluster_name;
                    most_recent = session.most_recent;
                    
                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = session.limit_number;
                    }

                    let cfs_sessions = crate::shasta_cfs_session::http_client::get(shasta_token, shasta_base_url, &cluster_name, &session_name, &limit_number).await?;

                    if cfs_sessions.is_empty() {
                        log::info!("No CFS session found!");
                        return Ok(())
                    } else {
                        cfs_utils::print_cfs_sessions(&cfs_sessions);

                    }
                }
            }
        }
        Verb::Log(log_cmd) => {
            logging_session_name = log_cmd.session_name;
            layer_id = log_cmd.layer_id;
            shasta_cfs_session_logs::client::session_logs(shasta_token, shasta_base_url, &None, &Some(logging_session_name), layer_id).await?;
        }
    }

    Ok(())
}

