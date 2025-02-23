use crate::backend_dispatcher::StaticBackendDispatcher;
use backend_dispatcher::{error::Error, interfaces::hsm::group::GroupTrait};

pub async fn exec(backend: &StaticBackendDispatcher, auth_token: &str, label: &str, force: bool) {
    if !force {
        // Validate if group can be deleted
        validation(backend, auth_token, label).await.unwrap();
    }

    // Delete group
    let result = backend.delete_group(auth_token, label).await;

    match result {
        Ok(_) => {
            println!("Group '{}' deleted", label);
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

// Checks if a group can be deleted.
// A group can be deleted if none of its members becomes orphan.
// A group member is orphan if it does not have a group assigned to it
async fn validation(
    backend: &StaticBackendDispatcher,
    auth_token: &str,
    label: &str,
) -> Result<(), Error> {
    // Find the list of xnames belonging only to the label to delete and if any, then stop
    // processing the request because those nodes can't get orphan
    let xname_vec = backend
        .get_member_vec_from_group_name_vec(auth_token, vec![label.to_string()])
        .await?;

    let xname_vec = xname_vec.iter().map(|e| e.as_str()).collect();

    let mut xname_map = backend
        .get_group_map_and_filter_by_group_vec(auth_token, xname_vec)
        .await?;

    xname_map.retain(|_xname, group_name_vec| {
        group_name_vec.len() == 1 && group_name_vec.first().unwrap() == label
    });

    let mut members_orphan_if_group_deleted: Vec<String> = xname_map
        .into_iter()
        .map(|(xname, _)| xname.clone())
        .collect();

    members_orphan_if_group_deleted.sort();

    if !members_orphan_if_group_deleted.is_empty() {
        return Err(Error::Message(format!(
            "ERROR - The hosts below will become orphan if group '{}' gets deleted.\n{:?}\n",
            label, members_orphan_if_group_deleted
        )));
    }

    Ok(())
}
