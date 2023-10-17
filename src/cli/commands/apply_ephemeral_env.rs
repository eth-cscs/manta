use mesa::shasta::ims;

use crate::common::jwt_ops::get_claims_from_jwt_token;

pub async fn exec(shasta_token: &str, shasta_base_url: &str, /* block: Option<bool>, */ image_id: &str) {
    // Take user name and check if there is an SSH public key with that name already in Alps
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    let user_public_key_name = jwt_claims["preferred_username"].as_str();

    log::info!(
        "Looking for user {} public SSH key",
        user_public_key_name.unwrap()
    );

    let user_public_ssh_id_value = if let Some(user_public_ssh_value) =
        ims::public_keys::http_client::get_single(
            shasta_token,
            shasta_base_url,
            user_public_key_name,
        )
        .await
    {
        user_public_ssh_value["id"].clone()
    } else {
        eprintln!("User '{}' does not have an SSH public key in Alps, Please contact platform sys admins. Exit", user_public_key_name.unwrap());
        std::process::exit(1);
    };

    log::info!("SSH key found with ID {}", user_public_ssh_id_value);

    // If public ssh key not found, then pompt user to provide public key
    // NOT YET. At this stage just throw an erro because the key was not found

    // Create IMS Job
    log::info!("Creating ephemeral environment baed on image ID {}", image_id);
    let resp_json_rslt = ims::job::http_client::post(
        shasta_token,
        shasta_base_url,
        "__test_image_to_delete",
        image_id,
        user_public_ssh_id_value.as_str().unwrap(),
    )
    .await;

    let hostname_value = match resp_json_rslt {
        Ok(resp_json) => resp_json
            .pointer("/ssh_containers/0/connection_info/customer_access/host")
            .cloned()
            .unwrap(),
        Err(_) => std::process::exit(1),
    };

    // if block.unwrap() {
    //     println!("Now block the call");
    //
    // } else {
    //     println!("Do not block");
    // }

    log::info!(
        "Ephemeral environment successfully created! hostname with ssh enabled: {}",
        hostname_value.as_str().unwrap()
    );
    println!("{}", hostname_value.as_str().unwrap());
}
