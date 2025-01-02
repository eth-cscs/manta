use crate::{
    backend_dispatcher::StaticBackendDispatcher, cli::process::validate_target_hsm_members,
};
use backend_dispatcher::{contracts::BackendTrait, types::HsmGroup};

pub async fn exec(
    backend: StaticBackendDispatcher,
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
    let group = HsmGroup::new(label, xname_vec_opt);

    // Call backend to create group
    let result = backend.add_hsm_group(&auth_token, group).await;

    match result {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
