// pub mod cfs_utils;
mod shasta_authentication;
mod shasta_cfs_configuration;
mod shasta_cfs_session;
mod shasta_cfs_session_logs;
mod shasta_vcs_utils;
mod shasta_cfs_component;
mod shasta_bos_template;
mod shasta_capmc;
mod shasta_hsm;
mod manta_cfs;
mod node_console;
mod git2_rs_utils;
mod create_cfs_session_from_repo;
mod vault;
mod cli_struct;

use clap::{Parser};
use cli_struct::{Cli, MainSubcommand, MainGetSubcommand, MainApplySubcommand, MainApplyNodeSubcommand};
use config::Config;
use manta_cfs::{configuration::{print_table}, layer::ConfigLayer};
use node_console::connect_to_console;

use clap_complete::{generate, Generator, Shell};

#[tokio::main]
async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {

    // Init logger
    env_logger::init();

    let cluster_name;
    let most_recent;
    let configuration_name;
    let session_name;
    let template_name;
    let limit_number;
    let logging_session_name;
    let xname;
    let layer_id;
    let shasta_token;
    let gitea_token;
    let shasta_base_url;

    let settings = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    shasta_base_url = settings.get::<String>("shasta_base_url").unwrap();
    // std::env::set_var("KUBECONFIG", settings.get::<String>("kubeconfig").unwrap());
    // std::env::set_var("SOCKS5", settings.get::<String>("socks5_proxy").unwrap());
    match settings.get::<String>("socks5_proxy") {
        Ok(socks_proxy) => std::env::set_var("SOCKS5", socks_proxy),
        Err(_) => log::info!("socks proxy not provided")
    }
    
    shasta_token = shasta_authentication::get_api_token().await?;
    gitea_token = vault::http_client::fetch_shasta_vcs_token().await.unwrap();

    // Parse input params
    let args = Cli::parse();

    // Process input params
    match args.command {
        MainSubcommand::Get(main_subcommand) => {
            match main_subcommand.main_get_subcommand {
                MainGetSubcommand::Configuration(configuration) => {

                    configuration_name = configuration.name;
                    cluster_name = configuration.cluster_name;
                    most_recent = configuration.most_recent;

                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = configuration.limit_number;
                    }

                    // Get CFS configurations
                    let cfs_configurations = crate::shasta_cfs_configuration::http_client::get(&shasta_token, &shasta_base_url, &cluster_name, &configuration_name, &limit_number).await?;

                    if cfs_configurations.is_empty() {
                        println!("No CFS configuration found!");
                        return Ok(())
                    } else if cfs_configurations.len() == 1 {

                        let most_recent_cfs_configuration = &cfs_configurations[0];

                        let mut layers: Vec<ConfigLayer> = vec![];
                        for layer in most_recent_cfs_configuration["layers"].as_array().unwrap() {

                            let gitea_commit_details = shasta_vcs_utils::http_client::get_commit_details(
                                layer["cloneUrl"].as_str().unwrap(), 
                                layer["commit"].as_str().unwrap(), 
                                &gitea_token).await?;

                            layers.push(
                                manta_cfs::layer::ConfigLayer::new(
                                    layer["name"].as_str().unwrap(), 
                                    layer["cloneUrl"].as_str().unwrap().trim_start_matches("https://api-gw-service-nmn.local/vcs/").trim_end_matches(".git"), 
                                    layer["commit"].as_str().unwrap(),
                                    gitea_commit_details["commit"]["committer"]["name"].as_str().unwrap(), 
                                    gitea_commit_details["commit"]["committer"]["date"].as_str().unwrap()
                                )
                            );
                        }

                        print_table(
                            manta_cfs::configuration::Config::new(
                                most_recent_cfs_configuration["name"].as_str().unwrap(), 
                                most_recent_cfs_configuration["lastUpdated"].as_str().unwrap(), 
                                layers
                            )
                        );
                    } else {

                        shasta_cfs_configuration::utils::print_table(cfs_configurations);
                    }
                },
                MainGetSubcommand::Session(session) => {

                    session_name = session.name;
                    cluster_name = session.cluster_name;
                    most_recent = session.most_recent;
                    
                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = session.limit_number;
                    }

                    let cfs_sessions = crate::shasta_cfs_session::http_client::get(&shasta_token, &shasta_base_url, &cluster_name, &session_name, &limit_number).await?;

                    if cfs_sessions.is_empty() {
                        log::info!("No CFS session found!");
                        return Ok(())
                    } else {

                        shasta_cfs_session::utils::print_table(cfs_sessions);
                    }
                },
                MainGetSubcommand::Template(template) => {

                    template_name = template.name;
                    cluster_name = template.cluster_name;
                    most_recent = template.most_recent;
                    
                    if most_recent {
                        limit_number = Some(1);
                    } else {
                        limit_number = template.limit_number;
                    }

                    let bos_templates = crate::shasta_bos_template::http_client::get(&shasta_token, &shasta_base_url, &cluster_name, &template_name, &limit_number).await?;

                    if bos_templates.is_empty() {
                        log::info!("No BOS template found!");
                        return Ok(())
                    } else {

                        shasta_bos_template::utils::print_table(bos_templates);
                    }
                }
            }
        }
        MainSubcommand::Apply(main_subcommand ) => {
            match main_subcommand.main_apply_subcommand {
                MainApplySubcommand::Session(apply_session_params) => {
                    
                    // Code below inspired on https://github.com/rust-lang/git2-rs/issues/561
        
                    // Get repo on current dir (pwd)
                    let repo = git2_rs_utils::local::get_repo(apply_session_params.repo_path.clone());

                    log::debug!("Local repo: {} state: {:?}", repo.path().display(), repo.state());
        
                    let cfs_session_name = create_cfs_session_from_repo::run(
                        repo
                        , gitea_token
                        , String::from(shasta_token)
                        , String::from(shasta_base_url)
                        , apply_session_params.ansible_limit
                        , apply_session_params.ansible_verbosity
                    ).await;

                    if apply_session_params.watch_logs {
                        log::info!("Fetching logs ...");
                        shasta_cfs_session_logs::client::session_logs(cfs_session_name.unwrap().as_str(), 0).await?;
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
                },
                MainApplySubcommand::Node(main_apply_subcommand) => {
                    match main_apply_subcommand.main_apply_node_subcommand {
                        MainApplyNodeSubcommand::Off(main_apply_node_off_subcommand) => {
                            let xnames = main_apply_node_off_subcommand.xnames.split(",").map(|xname| String::from(xname.trim())).collect();
                            log::info!("Servers to turn off: {:?}", xnames);
                            shasta_capmc::http_client::node_power_off::post(shasta_token.to_string(), main_apply_node_off_subcommand.reason, xnames, main_apply_node_off_subcommand.force).await?;
                        },
                        MainApplyNodeSubcommand::On(main_apply_node_on_subcommand) => {
                            let xnames = main_apply_node_on_subcommand.xnames.split(",").map(|xname| String::from(xname.trim())).collect();
                            log::info!("Servers to turn on: {:?}", xnames);
                            shasta_capmc::http_client::node_power_on::post(shasta_token.to_string(), main_apply_node_on_subcommand.reason, xnames, false).await?; // TODO: idk why power on does not seems to work when forced
                        },
                        MainApplyNodeSubcommand::Reset(main_apply_node_reset_subcommand) => {
                            let xnames = main_apply_node_reset_subcommand.xnames.split(",").map(|xname| String::from(xname.trim())).collect();
                            log::info!("Servers to reboot: {:?}", xnames);
                            shasta_capmc::http_client::node_restart::post(shasta_token.to_string(), main_apply_node_reset_subcommand.reason, xnames, main_apply_node_reset_subcommand.force).await?;
                        }
                    }
                }
            }
        }
        MainSubcommand::Log(log_cmd) => {
            logging_session_name = log_cmd.session_name;
            layer_id = log_cmd.layer_id;
            shasta_cfs_session_logs::client::session_logs_proxy(&shasta_token, &shasta_base_url, &None, &Some(logging_session_name), layer_id).await?;
        }
        MainSubcommand::Console(console_cmd) => {
            
            xname = console_cmd.xname;

            connect_to_console(&xname).await?;
        }
    }

    Ok(())
}

