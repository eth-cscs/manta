use crate::shasta::nodes;
use crate::{gitea, manta, shasta, create_cfs_session_from_repo, shasta_cfs_session_logs, cluster_ops};

use clap::{Parser, Subcommand, Args, ArgGroup};

use crate::manta::cfs::configuration as manta_cfs_configuration;
use crate::node_console::connect_to_console;

use crate::shasta::{
    authentication, bos_template, capmc,
    cfs::{
        component as shasta_cfs_component, configuration as shasta_cfs_configuration,
        session as shasta_cfs_session,
    },
};

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
    /// Access node console
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
    /// Get Cluster details
    Cluster(GetClusterArg),
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
pub struct GetClusterArg {
    /// Cluster name
    pub cluster_name: String,
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

pub async fn process_command(shasta_token: String, shasta_base_url: String, gitea_token: String) -> core::result::Result<(), Box<dyn std::error::Error>> {
    
    let args = Cli::parse();
    
    let cluster_name;
    let most_recent;
    let configuration_name;
    let session_name;
    let template_name;
    let limit_number;
    let logging_session_name;
    let xname;
    let layer_id;

    match args.command {
        MainSubcommand::Get(main_subcommand) => {
            match main_subcommand.main_get_subcommand {
                GetSubcommand::Configuration(configuration) => {
                    configuration_name = configuration.name;
                    cluster_name = configuration.cluster_name;
                    most_recent = configuration.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = configuration.limit_number;
                    }

                    // Get CFS configurations
                    let cfs_configurations = shasta_cfs_configuration::http_client::get(
                        &shasta_token,
                        &shasta_base_url,
                        &cluster_name,
                        &configuration_name,
                        &limit_number,
                    )
                    .await?;

                    if cfs_configurations.is_empty() {
                        println!("No CFS configuration found!");
                        return Ok(());
                    } else if cfs_configurations.len() == 1 {
                        let most_recent_cfs_configuration = &cfs_configurations[0];

                        let mut layers: Vec<manta_cfs_configuration::Layer> = vec![];
                        for layer in most_recent_cfs_configuration["layers"].as_array().unwrap() {
                            let gitea_commit_details = gitea::http_client::get_commit_details(
                                layer["cloneUrl"].as_str().unwrap(),
                                layer["commit"].as_str().unwrap(),
                                &gitea_token,
                            )
                            .await?;

                            layers.push(manta_cfs_configuration::Layer::new(
                                layer["name"].as_str().unwrap(),
                                layer["cloneUrl"]
                                    .as_str()
                                    .unwrap()
                                    .trim_start_matches("https://api-gw-service-nmn.local/vcs/")
                                    .trim_end_matches(".git"),
                                layer["commit"].as_str().unwrap(),
                                gitea_commit_details["commit"]["committer"]["name"]
                                    .as_str()
                                    .unwrap(),
                                gitea_commit_details["commit"]["committer"]["date"]
                                    .as_str()
                                    .unwrap(),
                            ));
                        }

                        manta_cfs_configuration::print_table(
                            manta::cfs::configuration::Configuration::new(
                                most_recent_cfs_configuration["name"].as_str().unwrap(),
                                most_recent_cfs_configuration["lastUpdated"]
                                    .as_str()
                                    .unwrap(),
                                layers,
                            ),
                        );
                    } else {
                        shasta_cfs_configuration::utils::print_table(cfs_configurations);
                    }
                }
                GetSubcommand::Session(session) => {
                    session_name = session.name;
                    cluster_name = session.cluster_name;
                    most_recent = session.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = session.limit_number;
                    }

                    let cfs_sessions = shasta_cfs_session::http_client::get(
                        &shasta_token,
                        &shasta_base_url,
                        &cluster_name,
                        &session_name,
                        &limit_number,
                    )
                    .await?;

                    if cfs_sessions.is_empty() {
                        log::info!("No CFS session found!");
                        return Ok(());
                    } else {
                        shasta_cfs_session::utils::print_table(cfs_sessions);
                    }
                }
                // MainGetSubcommand::BosSession(session) => {

                //     session_name = session.name;
                //     cluster_name = session.cluster_name;
                //     most_recent = session.most_recent;

                //     if most_recent {
                //         limit_number = Some(1);
                //     } else {
                //         limit_number = session.limit_number;
                //     }

                //     let bos_sessions = bos_session::http_client::get(&shasta_token, &shasta_base_url, &cluster_name, &session_name, &limit_number).await?;

                //     if bos_sessions.is_empty() {
                //         log::info!("No BOS session found!");
                //         return Ok(())
                //     } else {

                //         bos_session::utils::print_table(bos_sessions);
                //     }
                // },
                GetSubcommand::Template(template) => {
                    template_name = template.name;
                    cluster_name = template.cluster_name;
                    most_recent = template.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = template.limit_number;
                    }

                    let bos_templates = bos_template::http_client::get(
                        &shasta_token,
                        &shasta_base_url,
                        &cluster_name,
                        &template_name,
                        &limit_number,
                    )
                    .await?;

                    if bos_templates.is_empty() {
                        log::info!("No BOS template found!");
                        return Ok(());
                    } else {
                        bos_template::utils::print_table(bos_templates);
                    }
                }
                GetSubcommand::Node(node) => {
                    let cluster_name = node.cluster_name;

                    let nodes = shasta::hsm::http_client::get_hsm_groups(
                        &shasta_token,
                        &shasta_base_url,
                        cluster_name,
                    )
                    .await?;

                    if nodes.is_empty() {
                        log::info!("No nodes found!");
                        return Ok(());
                    } else {
                        shasta::hsm::utils::print_table(nodes);
                    }
                }
                GetSubcommand::Cluster(get_cluster_args) => {
                    let cluster_name = get_cluster_args.cluster_name;

                    let clusters =
                        cluster_ops::get_details(&shasta_token, &shasta_base_url, &cluster_name)
                            .await;

                    // println!("{:#?}", clusters);

                    for cluster in clusters {
                        println!("************************* CLUSTER *************************");

                        println!(" * HSM group label: {}", cluster.hsm_group_label);
                        println!(" * CFS configuration details:");
                        println!(
                            "   - name: {}",
                            cluster.most_recent_cfs_configuration_name_created["name"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - last updated: {}",
                            cluster.most_recent_cfs_configuration_name_created["lastUpdated"]
                                .as_str()
                                .unwrap_or_default()
                        );

                        let mut i = 0;
                        for layer in cluster.most_recent_cfs_configuration_name_created["layers"]
                            .as_array()
                            .unwrap()
                        {
                            println!("   + Layer {}", i);
                            println!(
                                "     - name: {}",
                                layer["name"].as_str().unwrap_or_default()
                            );
                            println!(
                                "     - url: {}",
                                layer["cloneUrl"].as_str().unwrap_or_default()
                            );
                            println!(
                                "     - commit: {}",
                                layer["commit"].as_str().unwrap_or_default()
                            );
                            println!(
                                "     - playbook: {}",
                                layer["playbook"].as_str().unwrap_or_default()
                            );
                            i += 1;
                        }

                        println!(" * CFS session details:");
                        println!(
                            "   - Name: {}",
                            cluster.most_recent_cfs_session_name_created["name"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - Configuration name: {}",
                            cluster.most_recent_cfs_session_name_created["configuration"]["name"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - Target: {}",
                            cluster.most_recent_cfs_session_name_created["target"]["definition"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!("   + Ansible details:");
                        println!(
                            "     - name: {}",
                            cluster.most_recent_cfs_session_name_created["ansible"]["config"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - limit: {}",
                            cluster.most_recent_cfs_session_name_created["ansible"]["limit"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!("   + Status:");
                        println!(
                            "     - status: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["status"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - succeeded: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["succeeded"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - job: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["job"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "     - start: {}",
                            cluster.most_recent_cfs_session_name_created["status"]["session"]
                                ["startTime"]
                                .as_str()
                                .unwrap_or_default()
                        );
                        println!(
                            "   - tags: {}",
                            cluster.most_recent_cfs_session_name_created["tags"]
                        );

                        println!(" * members: {}", nodes::nodes_to_string(&cluster.members));
                    }
                }
            }
        }
        MainSubcommand::Apply(main_subcommand) => {
            match main_subcommand.main_apply_subcommand {
                ApplySubcommand::Session(apply_session_params) => {
                    // Code below inspired on https://github.com/rust-lang/git2-rs/issues/561

                    let cfs_session_name = create_cfs_session_from_repo::run(
                        &apply_session_params.session_name,
                        apply_session_params.repo_path,
                        gitea_token,
                        shasta_token,
                        shasta_base_url,
                        apply_session_params.ansible_limit,
                        apply_session_params.ansible_verbosity,
                    )
                    .await;

                    if apply_session_params.watch_logs {
                        log::info!("Fetching logs ...");
                        shasta_cfs_session_logs::client::session_logs(
                            cfs_session_name.unwrap().as_str(),
                            None,
                        )
                        .await?;
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
                ApplySubcommand::Node(main_apply_subcommand) => {
                    match main_apply_subcommand.main_apply_node_subcommand {
                        ApplyNodeSubcommand::Off(main_apply_node_off_subcommand) => {
                            let xnames;
                            if main_apply_node_off_subcommand.xnames.is_some() {
                                // user provides a list of xnames
                                xnames = main_apply_node_off_subcommand
                                    .xnames
                                    .unwrap()
                                    .split(',')
                                    .map(|xname| String::from(xname.trim()))
                                    .collect();
                            } else {
                                // user provides a cluster name
                                let hsm_groups = shasta::hsm::http_client::get_hsm_groups(
                                    &shasta_token,
                                    &shasta_base_url,
                                    main_apply_node_off_subcommand.cluster_name,
                                )
                                .await?;
                                xnames = hsm_groups[0]["members"]["ids"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|xname_value| String::from(xname_value.as_str().unwrap()))
                                    .collect();
                            }
                            log::info!("Servers to turn off: {:?}", xnames);
                            capmc::http_client::node_power_off::post(
                                shasta_token.to_string(),
                                main_apply_node_off_subcommand.reason,
                                xnames,
                                main_apply_node_off_subcommand.force,
                            )
                            .await?;
                        }
                        ApplyNodeSubcommand::On(main_apply_node_on_subcommand) => {
                            let xnames;
                            if main_apply_node_on_subcommand.xnames.is_some() {
                                // user provides a list of xnames
                                xnames = main_apply_node_on_subcommand
                                    .xnames
                                    .unwrap()
                                    .split(',')
                                    .map(|xname| String::from(xname.trim()))
                                    .collect();
                            } else {
                                // user provides a cluster name
                                let hsm_groups = shasta::hsm::http_client::get_hsm_groups(
                                    &shasta_token,
                                    &shasta_base_url,
                                    main_apply_node_on_subcommand.cluster_name,
                                )
                                .await?;
                                xnames = hsm_groups[0]["members"]["ids"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|xname_value| String::from(xname_value.as_str().unwrap()))
                                    .collect();
                            }
                            log::info!("Servers to turn on: {:?}", xnames);
                            capmc::http_client::node_power_on::post(
                                shasta_token.to_string(),
                                main_apply_node_on_subcommand.reason,
                                xnames,
                                false,
                            )
                            .await?; // TODO: idk why power on does not seems to work when forced
                        }
                        ApplyNodeSubcommand::Reset(main_apply_node_reset_subcommand) => {
                            let xnames;
                            if main_apply_node_reset_subcommand.xnames.is_some() {
                                // user provides a list of xnames
                                xnames = main_apply_node_reset_subcommand
                                    .xnames
                                    .unwrap()
                                    .split(',')
                                    .map(|xname| String::from(xname.trim()))
                                    .collect();
                            } else {
                                // user provides a cluster name
                                let hsm_groups = shasta::hsm::http_client::get_hsm_groups(
                                    &shasta_token,
                                    &shasta_base_url,
                                    main_apply_node_reset_subcommand.cluster_name,
                                )
                                .await?;
                                xnames = hsm_groups[0]["members"]["ids"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|xname_value| String::from(xname_value.as_str().unwrap()))
                                    .collect();
                            }
                            log::info!("Servers to reboot: {:?}", xnames);
                            capmc::http_client::node_restart::post(
                                shasta_token.to_string(),
                                main_apply_node_reset_subcommand.reason,
                                xnames,
                                main_apply_node_reset_subcommand.force,
                            )
                            .await?;
                        }
                    }
                }
            }
        }
        MainSubcommand::Log(log_cmd) => {
            logging_session_name = log_cmd.session_name;
            layer_id = log_cmd.layer_id;
            shasta_cfs_session_logs::client::session_logs_proxy(
                &shasta_token,
                &shasta_base_url,
                &None,
                &Some(logging_session_name),
                layer_id,
            )
            .await?;
        }
        MainSubcommand::Console(console_cmd) => {
            xname = console_cmd.xname;

            connect_to_console(&xname).await?;
        }
    }

    Ok(())
}