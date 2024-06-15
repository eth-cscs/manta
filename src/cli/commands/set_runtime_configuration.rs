use mesa::error::Error;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: &str,
    hsm_group_name_opt: Option<&Vec<String>>,
    xname_vec_opt: Option<&Vec<String>>,
) -> Result<(), Error> {
    println!("Set runtime-configuration");

    let xnames = if let Some(hsm_group_name_vec) = hsm_group_name_opt {
        mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_name_vec,
        )
        .await
    } else if let Some(xname_vec) = xname_vec_opt {
        xname_vec.clone()
    } else {
        return Err(Error::Message(
            "Setting runtime configuration without a list of nodes".to_string(),
        ));
    };

    // TODO: try to not modify the CFS component directly but create a new BOS sessiontemplate,
    // this requires using BOS sessions v2
    mesa::cfs::component::shasta::utils::update_component_list_desired_configuration(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xnames,
        configuration_name,
        true,
    )
    .await;

    Ok(())
}
