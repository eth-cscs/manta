use mesa::hsm::group::r#struct::HsmGroup;

use crate::cli::process::validate_target_hsm_members;

pub async fn exec(
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    label: &str,
    xname_vec_opt: Option<Vec<&str>>,
) {
    // Validate user has access to the list of xnames requested
    if let Some(xname_vec) = &xname_vec_opt {
        validate_target_hsm_members(
            &auth_token,
            base_url,
            root_cert,
            xname_vec.iter().map(|xname| xname.to_string()).collect(),
        )
        .await;
    }

    // Create Group instance for http payload
    let group = HsmGroup::new(label, xname_vec_opt, None, None);

    // Call backend to create group
    let group_rslt =
        mesa::hsm::group::http_client::post(auth_token, base_url, root_cert, group.into()).await;

    match group_rslt {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
