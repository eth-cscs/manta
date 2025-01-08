use mesa::cfs::{
    component::http_client::v2::types::{ComponentResponse, StateResponse},
    session::http_client::v3::types::{
        CfsSessionGetResponse, Configuration, Session, Status, Target,
    },
};

/// Test is_cfs_configuration_a_desired_configuration returns TRUE when a CFS configuration
/// name appears as desired configuration in a list of CFS components
#[test]
fn test_is_cfs_configuration_a_desired_configuration_of_other_true() {
    let cfs_component_state_1 = StateResponse {
        clone_url: None,
        playbook: None,
        commit: None,
        session_name: None,
        last_updated: None,
    };

    let mut state_vec = Vec::new();
    state_vec.push(cfs_component_state_1);

    let cfs_component_1 = ComponentResponse {
        id: Some("1".to_string()),
        state: Some(state_vec),
        state_append: None,
        desired_config: Some("cfs_config_1".to_string()),
        error_count: Some(0),
        retry_policy: Some(0),
        enabled: Some(true),
        configuration_status: Some("unconfigured".to_string()),
        tags: None,
    };

    let cfs_component_2 = ComponentResponse {
        id: Some("2".to_string()),
        state: None,
        state_append: None,
        desired_config: Some("cfs_config_1".to_string()),
        error_count: Some(0),
        retry_policy: Some(0),
        enabled: Some(true),
        configuration_status: Some("unconfigured".to_string()),
        tags: None,
    };

    let cfs_component_vec = vec![cfs_component_1, cfs_component_2];

    let cfs_configuration_name_in_cfs_session = "cfs_config_1";

    let xname_vec = vec!["2"];

    let sol = is_cfs_configuration_a_desired_configuration_of_other(
        &cfs_component_vec,
        cfs_configuration_name_in_cfs_session,
        xname_vec,
    );

    assert!(sol.eq(&vec!["1".to_string()]));
}

/// Test is_cfs_configuration_a_desired_configuration returns TRUE when a CFS configuration
/// name appears as desired configuration in a list of CFS components
#[test]
fn test_is_cfs_configuration_a_desired_configuration_of_other_false() {
    let cfs_component_state_1 = StateResponse {
        clone_url: None,
        playbook: None,
        commit: None,
        session_name: None,
        last_updated: None,
    };

    let mut state_vec = Vec::new();
    state_vec.push(cfs_component_state_1);

    let cfs_component_1 = ComponentResponse {
        id: Some("1".to_string()),
        state: Some(state_vec),
        state_append: None,
        desired_config: Some("cfs_config_1".to_string()),
        error_count: Some(0),
        retry_policy: Some(0),
        enabled: Some(true),
        configuration_status: Some("unconfigured".to_string()),
        tags: None,
    };

    let mut cfs_component_vec = Vec::new();

    cfs_component_vec.push(cfs_component_1);

    let cfs_configuration_name_in_cfs_session = "cfs_config_1";

    let xname_vec = vec!["1"];

    let sol = is_cfs_configuration_a_desired_configuration_of_other(
        &cfs_component_vec,
        cfs_configuration_name_in_cfs_session,
        xname_vec,
    );

    assert!(sol.is_empty());
}

/// Test is_cfs_configuration_a_desired_configuration returns FALSE when a CFS configuration
/// name appears as desired configuration in a list of CFS components
#[test]
fn test_is_cfs_configuration_a_desired_configuration_of_other_false_2() {
    let cfs_component_state_1 = StateResponse {
        clone_url: None,
        playbook: None,
        commit: None,
        session_name: None,
        last_updated: None,
    };

    let mut state_vec = Vec::new();
    state_vec.push(cfs_component_state_1);

    let cfs_component_1 = ComponentResponse {
        id: Some("1".to_string()),
        state: Some(state_vec),
        state_append: None,
        desired_config: Some("cfs_config_1".to_string()),
        error_count: Some(0),
        retry_policy: Some(0),
        enabled: Some(true),
        configuration_status: Some("unconfigured".to_string()),
        tags: None,
    };

    let mut cfs_component_vec = Vec::new();

    cfs_component_vec.push(cfs_component_1);

    let cfs_configuration_name_in_cfs_session = "cfs_config_2";

    let xname_vec = vec!["2"];

    let sol = is_cfs_configuration_a_desired_configuration_of_other(
        &cfs_component_vec,
        cfs_configuration_name_in_cfs_session,
        xname_vec,
    );

    assert!(sol.is_empty());
}

#[test]
fn test_is_cfs_configuration_used_to_build_image_true() {
    let cfs_config = Configuration {
        name: Some("cfs_config_1".to_string()),
        limit: None,
    };

    let session = Session {
        job: None,
        ims_job: None,
        completion_time: None,
        start_time: None,
        status: None,
        succeeded: Some("true".to_string()),
    };

    let status = Status {
        artifacts: None,
        session: Some(session),
    };

    let target = Target {
        definition: Some("image".to_string()),
        groups: None,
        image_map: Some(Vec::new()),
    };

    let cfs_session = CfsSessionGetResponse {
        name: Some("cfs_session_1".to_string()),
        configuration: Some(cfs_config),
        ansible: None,
        target: Some(target),
        status: Some(status),
        tags: None,
        debug_on_failure: false,
        logs: None,
    };

    let mut cfs_session_vec = Vec::new();
    cfs_session_vec.push(cfs_session);

    let cfs_configuration_name = "cfs_config_1";
    let cfs_session_name = "cfs_session_1";

    assert!(is_cfs_configuration_used_to_build_image(
        &cfs_session_vec,
        cfs_session_name,
        cfs_configuration_name
    )
    .is_empty());
}

#[test]
fn test_is_cfs_configuration_used_to_build_image_false() {
    let cfs_config = Configuration {
        name: Some("cfs_config_1".to_string()),
        limit: None,
    };

    let cfs_session = CfsSessionGetResponse {
        name: Some("cfs_session_1".to_string()),
        configuration: Some(cfs_config),
        ansible: None,
        target: None,
        status: None,
        tags: None,
        debug_on_failure: false,
        logs: None,
    };

    let mut cfs_session_vec = Vec::new();
    cfs_session_vec.push(cfs_session);

    let cfs_configuration_name = "cfs_config_2";
    let cfs_session_name = "cfs_session_1";

    assert!(
        !is_cfs_configuration_used_to_build_image(
            &cfs_session_vec,
            cfs_session_name,
            cfs_configuration_name
        )
        .len()
            > 0
    );
}

#[test]
fn test_is_cfs_configuration_used_to_build_image_false_2() {
    let cfs_config = Configuration {
        name: Some("cfs_config_1".to_string()),
        limit: None,
    };

    let target = Target {
        definition: Some("dynamic".to_string()),
        groups: None,
        image_map: Some(Vec::new()),
    };

    let cfs_session = CfsSessionGetResponse {
        name: Some("cfs_session_1".to_string()),
        configuration: Some(cfs_config),
        ansible: None,
        target: Some(target),
        status: None,
        tags: None,
        debug_on_failure: false,
        logs: None,
    };

    let mut cfs_session_vec = Vec::new();
    cfs_session_vec.push(cfs_session);

    let cfs_configuration_name = "cfs_config_2";
    let cfs_session_name = "cfs_session_1";

    assert!(
        !is_cfs_configuration_used_to_build_image(
            &cfs_session_vec,
            cfs_session_name,
            cfs_configuration_name
        )
        .len()
            > 0
    );
}

/// Validate CFS session type dynamic:
/// - check CFS configuration related to CFS session is not a desired configuration
pub fn is_cfs_configuration_a_desired_configuration(
    cfs_component_vec: &Vec<ComponentResponse>,
    cfs_configuration_name: &str,
) -> bool {
    // - check CFS configuration related to CFS session is not a desired configuration
    cfs_component_vec.iter().any(|cfs_component| {
        cfs_component
            .desired_config
            .as_ref()
            .unwrap()
            .eq(&cfs_configuration_name)
    })
}

/// Validate CFS session type dynamic:
/// - check CFS configuration related to CFS session is a desired configuration used by a node or
/// hsm group different than the provided one.
/// We need this validation because, when deleting a CFS session, we need to make sure it is not
/// used by a node that belongs to the HSM
pub fn is_cfs_configuration_a_desired_configuration_of_other(
    cfs_component_vec: &Vec<ComponentResponse>,
    cfs_configuration_name: &str,
    xname_vec: Vec<&str>,
) -> Vec<String> {
    // - check CFS configuration related to CFS session is not a desired configuration
    cfs_component_vec
        .iter()
        .filter(|cfs_component| {
            cfs_component
                .desired_config
                .as_ref()
                .unwrap()
                .eq(&cfs_configuration_name)
                && !xname_vec.contains(&cfs_component.id.as_ref().unwrap().as_str())
        })
        .map(|cfs_component| cfs_component.id.clone().unwrap())
        .collect()
}

/// Validate CFS session type image:
/// - check CFS configuration related to CFS session is not used to build any other image
pub fn is_cfs_configuration_used_to_build_image(
    cfs_session_vec: &Vec<CfsSessionGetResponse>,
    cfs_session_name: &str,
    cfs_configuration_name: &str,
) -> Vec<String> {
    /* cfs_session_vec
    .iter()
    .filter(|cfs_session| {
        cfs_session
                    .get_configuration_name()
                    .unwrap()
                    .eq(&cfs_configuration_name)
                // NOTE: No need the below condition because current CFS session to delete is suppossedly still running
                // therefore not yet finished and as a consequence it won't have a result_id
                // value
                    && cfs_session.name.as_ref().unwrap().eq(&cfs_session_name)
    })
    .any(|cfs_session| !cfs_session.get_result_id_vec().is_empty()) */
    cfs_session_vec
        .iter()
        .filter(|cfs_session| {
            cfs_session
                .get_configuration_name()
                .unwrap()
                .eq(&cfs_configuration_name)
                && cfs_session.name.as_ref().unwrap().eq(&cfs_session_name)
                && cfs_session.is_target_def_image()
                && cfs_session.is_success()
        })
        .flat_map(|cfs_session| cfs_session.get_result_id_vec())
        .collect()
}
