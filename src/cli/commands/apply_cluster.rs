use super::apply_sat_file;

#[deprecated(since = "1.28.2", note = "Please use `apply_sat_file` instead")]
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    sat_file_content: String,
    values_file_content_opt: Option<String>,
    values_cli_opt: Option<Vec<String>>,
    hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    gitea_base_url: &str,
    gitea_token: &str,
    // tag: &str,
    do_not_reboot: bool,
) {
    apply_sat_file::command::exec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        k8s_api_url,
        sat_file_content,
        values_file_content_opt,
        values_cli_opt,
        hsm_group_param_opt,
        hsm_group_available_vec,
        ansible_verbosity_opt,
        ansible_passthrough_opt,
        gitea_base_url,
        gitea_token,
        do_not_reboot,
        None,
        None,
        false,
        false,
        false,
    )
    .await;
}
