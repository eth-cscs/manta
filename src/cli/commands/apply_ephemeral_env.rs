use csm_rs::ims;

pub async fn exec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  /* block: Option<bool>, */ image_id: &str,
) {
  // Take user name and check if there is an SSH public key with that name already in Alps
  let user_public_key_name =
    csm_rs::common::jwt_ops::get_preferred_username(shasta_token)
      .expect("ERROR - claim 'preferred_user' not found in JWT token");

  log::info!("Looking for user '{}' public SSH key", user_public_key_name);

  let user_public_ssh_id_value = if let Some(user_public_ssh_value) =
    ims::public_keys::http_client::v3::get_single(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &user_public_key_name,
    )
    .await
  {
    user_public_ssh_value["id"].clone()
  } else {
    eprintln!("User '{}' does not have an SSH public key in Alps, Please contact platform sys admins. Exit", user_public_key_name);
    std::process::exit(1);
  };

  log::info!("SSH key found with ID {}", user_public_ssh_id_value);

  // If public ssh key not found, then pompt user to provide public key
  // NOT YET. At this stage just throw an erro because the key was not found

  // Create IMS Job
  log::info!(
    "Creating ephemeral environment baed on image ID {}",
    image_id
  );
  let resp_json_rslt = ims::job::http_client::post_customize(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    "__ephemeral_image",
    image_id,
    user_public_ssh_id_value.as_str().unwrap(),
  )
  .await;

  let hostname_value = match resp_json_rslt {
    Ok(resp_json) => resp_json
      .pointer("/ssh_containers/0/connection_info/customer_access/host")
      .cloned()
      .unwrap(),
    Err(error) => {
      eprintln!(
        "ERROR: could not create ephemeral environment. Reason:\n{:#?}\nExit",
        error
      );
      std::process::exit(1);
    }
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
