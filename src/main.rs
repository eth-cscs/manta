pub mod cfs_utils;
pub mod auth;
// pub mod k8s_programmatic_client;
mod shasta_cfs_configuration;
mod shasta_cfs_session;
mod shasta_cfs_session_logs;
mod shasta_vcs_utils;
mod manta_cfs;
mod git2_rs_utils;
mod create_session_from_repo;

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
    cluster_name: Option<String>,
    /// Repo path
    #[clap(short, long, value_parser)]
    repo_path: String,
    /// Watch logs
    #[clap(short, long, value_parser)]
    watch_logs: bool,
    /// Ansible limit
    #[clap(short, long, value_parser)]
    ansible_limit: String
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
    // let repo_path;

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

                        log::info!("{}", manta_cfs::configuration::create(most_recent_cfs_configuration, &gitea_token).await?);
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
            match apply_cmd.shasta_object {
                ShastaObjectApply::Session(apply_session_params) => {
                    
                    // Code related to this method inspired on https://github.com/rust-lang/git2-rs/issues/561
        
                    // Get repo on current dir (pwd)
                    let repo = git2_rs_utils::local::get_repo(apply_session_params.repo_path.clone());

                    log::debug!("{} state={:?}", repo.path().display(), repo.state());
        
                    let cfs_session_name = create_session_from_repo::run(repo, gitea_token, String::from(shasta_token), String::from(shasta_base_url), apply_session_params.ansible_limit).await;

                    log::info!("cfs session: {}", cfs_session_name.as_ref().unwrap());

                    if apply_session_params.watch_logs {
                        log::info!("Fetching logs ...");
                        shasta_cfs_session_logs::client::session_logs(cfs_session_name.unwrap().as_str(), 0).await;
                    }

                    // match session_logs_result {
                    //     Ok(()) => log::debug!("Logs finished"),
                    //     Err(_) => {
                    //         // Session do exists hence a pod managing the CFS session should start soon ...
                    //         session_logs_result = shasta_cfs_session_logs::client::session_logs(&cfs_session_name.as_ref().unwrap(), 0).await;
                    //     }
                    // }

                    // // // Get indexes
                    // // let index = repo.index().unwrap();
        
                    // // // Check if conflicts
                    // // // TODO: This may be the wrong place to check if there are conflicts (maybe too early) and we need to fetch data from remote
                    // // if index.has_conflicts() {
                    // //     log::error!("THERE ARE CONFLICTS!!!!!");
        
                    // //     std::process::exit(1);
                    // // }
        
                    // // // Adding all files (git add)
                    // // log::debug!("Running 'git add'");
        
                    // // git2_rs_utils::local::add_all(&repo);
                    // // log::debug!("git add command ran successfully");
        
                    // // // Get last commit
                    // // let commit = git2_rs_utils::local::get_last_commit(&repo);
        
                    // // let timestamp = commit.time().seconds();
                    // // let tm = NaiveDateTime::from_timestamp(timestamp, 0);
                    // // log::debug!("\nCommit {}\nAuthor: {}\nDate:   {}\n\n    {}", commit.id(), commit.author(), tm, commit.message().unwrap_or("no commit message"));
        
                    // // // Create commit
                    // // log::debug!("Committing changes");
        
                    // // git2_rs_utils::local::commit(&repo);
        
                    // // log::debug!("Commit seems successful");
        
                    // // // Get remote from repo
                    // // let remote = repo.find_remote("origin")?;
        
                    // // log::debug!("remote name: {}", remote.name().unwrap());
                    // // log::debug!("url: {}", remote.url().unwrap());
                    
                    // // // Get refspecs
                    // // let refspecs = remote.refspecs();
                    
                    // // for refspec in refspecs {
                    // //     log::debug!("remote refspecs: {:#?}", refspec.str().unwrap());
                    
                    // // }
        
                    // // // Check conflicts
                    // // git2_rs_utils::local::fetch_and_check_conflicts(repo)?;
                    // // log::debug!("No conflicts");
        
                    // // // Push commit
                    // // git2_rs_utils::local::push(remote)?;
        
        
        
                    // // Get last (most recent) commit
                    // let local_last_commit_local = git2_rs_utils::local::get_last_commit(&repo).unwrap();
        
                    // if !git2_rs_utils::local::untracked_changed_local_files(&repo).unwrap() {
        
                    //     if Confirm::with_theme(&ColorfulTheme::default())
                    //         .with_prompt("Your local repo has not commited changes. Do you want to continue?")
                    //         .interact()
                    //         .unwrap()
                    //     {
                    //         println!("Continue. Checking commit id {} against remote", local_last_commit_local.id());
                    //     } else {
                    //         println!("Cancelled by user.");
                    //         std::process::exit(0);
                    //     }
                    //     // Question::new("Your repo has some untracked files. Do you want to continue ()?")
                    //     //     .yes_no()
                    //     //     .until_acceptable()
                    //     //     .ask();       
                    // }

                    // // Check local repo/branch exists in remote ???

                    // // Check last commit in local exists in remote ???

                    // // Check last commit in local and remote matches ???

                    // // Check site.yml file exists inside repo folder
                    // if !Path::new(repo.path()).exists() {
                    //     log::error!("site.yaml file does not exists in {}", repo.path().display());
                    //     std::process::exit(1);
                    // }

                    // // Get repo name
                    // let repo_ref_origin = repo.find_remote("origin").unwrap();
                    // let repo_ref_origin_url = repo_ref_origin.url().unwrap();
                    // let repo_name = repo_ref_origin_url.substring(repo_ref_origin_url.rfind(|c| c == '/').unwrap()+1, repo_ref_origin_url.rfind(|c| c == '.').unwrap());
                    
                    // log::info!("Repo name:\n{}", repo_name);

                    // // Check if latest local commit id exists in Shasta cvs
                    // let shasta_commitid_details_resp = shasta_vcs_utils::http_client::get_commit_details(&format!("cray/{}", repo_name), &local_last_commit_local.id().to_string(), &gitea_token).await;
                    
                    // match shasta_commitid_details_resp {
                    //     Ok(_) => log::info!("Local latest commit id {} for repo {} exists in shasta", local_last_commit_local.id(), repo_name),
                    //     Err(e) => {
                    //         log::error!("{}", e);
                    //         std::process::exit(1);
                    //     }
                    // }

                    // let shasta_commitid_details = shasta_commitid_details_resp.unwrap();

                    // // Check conflicts
                    // // git2_rs_utils::local::fetch_and_check_conflicts(&repo)?;
                    // // log::debug!("No conflicts");

                    // // Create CFS configuration
                    // let cfs_layer = shasta_cfs_configuration::Layer::new(
                    //     String::from(format!("https://api-gw-service-nmn.local/vcs/cray/{}", repo_name)), 
                    //     String::from(shasta_commitid_details["sha"].as_str().unwrap()), 
                    //     String::from(format!("{}-{}", repo_name.substring(1, repo_name.len()), chrono::offset::Local::now().to_rfc3339_opts(SecondsFormat::Secs, true))), 
                    //     String::from("site.yml"));

                    // let mut cfs_configuration = shasta_cfs_configuration::Configuration::new();

                    // cfs_configuration = shasta_cfs_configuration::add_layer(cfs_layer, cfs_configuration);

                    // log::info!("CFS configuration:\n{:#?}", cfs_configuration);
                    
                    // // Update/PUT CFS configuration
                    // log::debug!("Replacing '_' with '-' in repo name and create configuration and session name.");
                    // let cfs_object_name = format!("m-{}", str::replace(&repo_name, "_", "-"));
                    // let cfs_configuration_resp = shasta_cfs_configuration::http_client::put(shasta_token, shasta_base_url, cfs_configuration, &cfs_object_name).await;

                    // match cfs_configuration_resp {
                    //     Ok(_) => log::info!("CFS configuration response: {:#?}", cfs_configuration_resp),
                    //     Err(e) => {
                    //         log::error!("{}", e);
                    //         std::process::exit(1);
                    //     }
                    // };

                    // // Create CFS session
                    // let cfs_session_name = format!("{}-{}", cfs_object_name, chrono::Utc::now().format("%Y%m%d%H%M%S"));
                    // let session = shasta_cfs_session::Session::new(
                    //     cfs_session_name,
                    //     cfs_object_name, 
                    //     Some(String::from("x1500c3s4b0n0"))
                    // );

                    // log::debug!("Session:\n{:#?}", session);
                    // let cfs_session_resp = shasta_cfs_session::http_client::post(shasta_token, shasta_base_url, session).await;

                    // match cfs_session_resp {
                    //     Ok(_) => log::info!("CFS session response: {:#?}", cfs_session_resp),
                    //     Err(e) => {
                    //         log::error!("{}", e);
                    //         std::process::exit(1);
                    //     }
                    // };

                    // // Get pod name running the CFS session

                    // // Get list of ansible containers in pod running CFS session

                    // // Connect logs ????





                    // log::info!("last commit for shasta repo commit id {} vs local repo commit id {}", shasta_commitid_details["sha"].as_str().unwrap(), local_last_commit_local.id());
        
                    // log::info!("last commit author: {}", local_last_commit_local.author());
                    // log::info!("last commit body: {:?}", local_last_commit_local.body());
                    // log::info!("last commit committer: {}", local_last_commit_local.committer());
                    // log::info!("last commit id: {}", local_last_commit_local.id());
                    // log::info!("last commit message: {}", local_last_commit_local.message().unwrap());
                    // log::info!("last commit summary: {}", local_last_commit_local.summary().unwrap());
                    // log::info!("last commit time: {:?}", local_last_commit_local.time());
                    // log::info!("last commit tree: {:#?}", local_last_commit_local.tree().unwrap());

                    // log::info!("head target: {}", repo.head().unwrap().target().unwrap());
                    // log::info!("head name: {}", repo.head().unwrap().name().unwrap());
                    // log::info!("is head remote? {}", repo.head().unwrap().is_remote());
                    // log::info!("name: {}", repo.head().unwrap().name().unwrap());
                    // log::info!("path: {}", repo.path().as_os_str().to_str().unwrap());
                    // // log::info!("namespace: {}", repo.namespace().unwrap());
                    // // log::info!("message: {}", repo.message().unwrap());
                    // log::info!("state: {:?}", repo.state());
                    // log::info!("workdir: {}", repo.workdir().unwrap().as_os_str().to_str().unwrap());
                    // log::info!("List remotes: ");
                    // for remote in repo.remotes().unwrap().into_iter() {
                    //     log::info!("remote name: {}", remote.unwrap());
                    //     log::info!("remote name: {}", repo.find_remote(remote.unwrap()).unwrap().name().unwrap());
                    //     log::info!("remote url: {}", repo.find_remote(remote.unwrap()).unwrap().url().unwrap());
                    //     for refspec in repo.find_remote(remote.unwrap()).unwrap().refspecs().into_iter() {
                    //         log::info!("remote refspec destination: {}", refspec.dst().unwrap());
                    //         log::info!("remote refspec direction: {:?}", refspec.direction());
                    //         log::info!("remote refspec src: {}", refspec.src().unwrap());
                    //         log::info!("remote refspec str: {}", refspec.str().unwrap());
                    //     }
                    // }
                    // for branch in repo.branches(None).unwrap() {
                    //     let branch = &branch.unwrap();
                    //     log::info!("branch type: {:?}", branch.1);
                    //     log::info!("branch name: {:?}", branch.0.name()?.unwrap());
                    //     log::info!("branch reference name: {:?}", branch.0.get().name().unwrap());
                    // }
                    // for reference in repo.references().unwrap() {
                    //     let refer = reference.unwrap();
                    //     log::info!("is reference a branch? {}", refer.is_branch());
                    //     log::info!("is reference remote? {}", refer.is_remote());
                    //     log::info!("reference type: {}", refer.kind().unwrap().str());
                    //     log::info!("reference name: {}", refer.name().unwrap());
                    //     log::info!("reference target: {:?}", refer.target());
                    // }
                }
            }
        }
        Verb::Log(log_cmd) => {
            logging_session_name = log_cmd.session_name;
            layer_id = log_cmd.layer_id;
            shasta_cfs_session_logs::client::session_logs_proxy(shasta_token, shasta_base_url, &None, &Some(logging_session_name), layer_id).await?;
        }
    }

    Ok(())
}

