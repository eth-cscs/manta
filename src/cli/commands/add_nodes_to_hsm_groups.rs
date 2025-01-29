use backend_dispatcher::interfaces::group::GroupTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::{backend_dispatcher::StaticBackendDispatcher, common};

/// Add/assign a list of xnames to a list of HSM groups
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    target_hsm_name: &String,
    is_regex: bool,
    hosts_string: &str,
    dryrun: bool,
) {
    // Convert user input to xname
    let mut xname_to_move_vec = common::node_ops::resolve_node_list_user_input_to_xname(
        backend,
        shasta_token,
        hosts_string,
        is_regex,
    )
    .await
    .unwrap_or_else(|e| {
        eprintln!(
            "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
            e
        );
        std::process::exit(1);
    });

    xname_to_move_vec.sort();
    xname_to_move_vec.dedup();

    // Check if there are any xname to migrate/move and exit otherwise
    if xname_to_move_vec.is_empty() {
        println!("No hosts to move. Exit");
        std::process::exit(0);
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{:?}\nThe nodes above will be added to HSM group '{}'. Do you want to proceed?",
            xname_to_move_vec, target_hsm_name
        ))
        .interact()
        .unwrap()
    {
        log::info!("Continue",);
    } else {
        println!("Cancelled by user. Aborting.");
        std::process::exit(0);
    }

    let target_hsm_group = backend.get_group(shasta_token, &target_hsm_name).await;

    if target_hsm_group.is_err() {
        eprintln!(
            "Target HSM group {} does not exist, Nothing to do. Exit",
            target_hsm_name
        );
    }

    let xnames_to_move: Vec<&str> = xname_to_move_vec
        .iter()
        .map(|xname| xname.as_str())
        .collect();

    if dryrun {
        println!(
            "dryrun - Add nodes {:?} to {}",
            xnames_to_move, target_hsm_name
        );
        std::process::exit(0);
    }

    let node_migration_rslt = backend
        .add_members_to_group(shasta_token, &target_hsm_name, xnames_to_move)
        .await;

    match node_migration_rslt {
        Ok(mut target_hsm_group_member_vec) => {
            target_hsm_group_member_vec.sort();
            println!(
                "HSM '{}' members: {:?}",
                target_hsm_name, target_hsm_group_member_vec
            );
        }
        Err(e) => eprintln!("{}", e),
    }
}
