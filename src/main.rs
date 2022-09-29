pub mod cfs_utils;
pub mod auth;
// pub mod k8s_programmatic_client;
mod shasta_cfs_configuration;
mod shasta_cfs_session;
mod shasta_cfs_session_logs;
mod shasta_vcs;
mod manta_cfs;

// use std::process;

use std::path::Path;

use clap::{Args, ArgGroup, Parser, Subcommand};
use git2::{ObjectType, Repository, PushOptions};
use k8s_openapi::chrono::NaiveDateTime;

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
    /// Create new shasta object 
    Apply(Apply),
    /// Print session logs
    Log(Log),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Get {
    #[clap(subcommand)]
    shasta_object: ShastaObjectGet,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Apply {
    #[clap(subcommand)]
    shasta_object: ShastaObjectApply,
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
enum ShastaObjectGet {
    /// Get configuration details
    Configuration(GetConfigurationOptions),
    /// Get session details
    Session(GetSessionOptions),
}

#[derive(Debug, Subcommand)]
enum ShastaObjectApply {
    /// Apply/Create new session
    Session(ApplySessionOptions),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("config-type").args(&["name", "cluster-name"]),))]
#[clap(group(ArgGroup::new("config-limit").args(&["most-recent", "limit-number"]),))]
struct GetConfigurationOptions {
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
#[clap(group(ArgGroup::new("session-limit").args(&["most-recent", "limit-number"]),))]
struct GetSessionOptions {
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
#[clap(group(ArgGroup::new("session-type").args(&["session-name", "cluster-name"]),))]
struct ApplySessionOptions {
    /// Session name
    #[clap(short, long, value_parser)]
    session_name: Option<String>,
    /// Cluster name
    #[clap(short, long, value_parser)]
    cluster_name: Option<String>
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
                ShastaObjectGet::Configuration(configuration) => {

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
                ShastaObjectGet::Session(session) => {

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
        Verb::Apply(apply_cmd) => {

            // Code below inspired on https://github.com/rust-lang/git2-rs/issues/561

            // Get repo on current dir (pwd)
            let repo_root = std::env::current_dir().unwrap();
            log::debug!("Checking repo on {}", repo_root.display());

            let repo = Repository::open(repo_root.as_os_str()).expect("Couldn't open repository");
            log::debug!("{} state={:?}", repo.path().display(), repo.state());

            // Adding all files (git add)
            log::debug!("Running 'git add'");

            let mut index = repo.index().unwrap();
            index.add_all(&["."], git2::IndexAddOption::DEFAULT, Some(&mut |path: &Path, _matched_spec: &[u8]| -> i32 {
                let status = repo.status_file(path).unwrap();
        
                let ret = if status.contains(git2::Status::WT_MODIFIED)
                    || status.contains(git2::Status::WT_NEW)
                {
                    log::debug!(" - Adding file: '{}'", path.display());

                    0
                } else {
                    log::debug!(" - NOT adding file: '{}'", path.display());

                    1
                };

                ret
            }))
                .unwrap();
            index.write().unwrap();
            log::debug!("git add command ran successfully");

            // Get last commit
            let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
            let commit = obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit")).unwrap();

            let timestamp = commit.time().seconds();
            let tm = NaiveDateTime::from_timestamp(timestamp, 0);
            log::debug!("commit {}\nAuthor: {}\nDate:   {}\n\n    {}",
                    commit.id(),
                    commit.author(),
                    tm,
                    commit.message().unwrap_or("no commit message"));

            // Get remote from repo
            let mut remote = repo.find_remote("origin").unwrap();

            log::debug!("remote name: {}", remote.name().unwrap());

            let refspecs = remote.refspecs();
            for refspec in refspecs {
                log::debug!("remote refspecs: {:#?}", refspec.str().unwrap());

            }
            log::debug!("url: {}", remote.url().unwrap());

            // Create commit
            log::debug!("Committing changes");

            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let signature = repo.signature().unwrap();
            let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
            let tree = repo.find_tree(oid).unwrap();
            repo.commit(
                Some("HEAD"), 
                &signature, 
                &signature, 
                "testing git2-rs... commit created programatically...", 
                &tree, 
                &[&parent_commit]).unwrap();

            log::debug!("Commit seems successful");

            // Configure callbacks for push operation
            let mut callbacks = git2::RemoteCallbacks::new();

            callbacks.credentials(|_url, _username_from_url, _allowed_types| {
                log::debug!("url is: {}", _url);
                log::debug!("username from url is: {}", _username_from_url.unwrap_or("Not defined")); // IMPORTANT: username from url is None because .git/config has https address 'url = https://git.cscs.ch/msopena/manta.git' 
                log::debug!("allowed types are: {:#?}", _allowed_types);
                
                git2::Cred::userpass_plaintext("msopena", "MasberLugano0720") // IMPORTANT: this with combination of .git/config having an https address 'url = https://git.cscs.ch/msopena/manta.git' makes library to switch to CredentialType::USER_PASS_PLAINTEXT
            });

            callbacks.push_update_reference(|_reference_name, callback_status| {
                log::debug!("reference name: {}", _reference_name);
                log::debug!("callback status: {}", callback_status.unwrap_or("Not defined"));

                Ok(())
            });

            // Configure push options
            let po = &mut PushOptions::default();
            po.remote_callbacks(callbacks);

            // Push
            remote.push(&["+refs/heads/main","+refs/heads/apply-dynamic-target-session"], Some(po))?;
            
        }
        Verb::Log(log_cmd) => {
            logging_session_name = log_cmd.session_name;
            layer_id = log_cmd.layer_id;
            shasta_cfs_session_logs::client::session_logs(shasta_token, shasta_base_url, &None, &Some(logging_session_name), layer_id).await?;
        }
    }

    Ok(())
}

