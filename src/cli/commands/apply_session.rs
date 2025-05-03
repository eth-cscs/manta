use std::path::PathBuf;

use futures::{AsyncBufReadExt, TryStreamExt};
use manta_backend_dispatcher::{
    error::Error,
    interfaces::{apply_session::ApplySessionTrait, cfs::CfsTrait},
    types::K8sDetails,
};

use crate::{
    common::{audit::Audit, jwt_ops, kafka::Kafka},
    manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Creates a CFS session target dynamic
/// Returns a tuple like (<cfs configuration name>, <cfs session name>)
pub async fn exec(
    backend: StaticBackendDispatcher,
    site: &str,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_conf_sess_name: Option<&String>,
    playbook_yaml_file_name_opt: Option<&String>,
    hsm_group_opt: Option<&String>,
    repos_paths: Vec<PathBuf>,
    ansible_limit: Option<String>,
    ansible_verbosity: Option<String>,
    ansible_passthrough: Option<String>,
    watch_logs: bool,
    kafka_audit_opt: Option<&Kafka>,
    k8s: &K8sDetails,
) -> Result<(String, String), Error> {
    let (cfs_configuration_name, cfs_session_name) = backend
        .i_apply_session(
            gitea_token,
            gitea_base_url,
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            // k8s_api_url,
            cfs_conf_sess_name,
            playbook_yaml_file_name_opt,
            hsm_group_opt,
            repos_paths,
            ansible_limit.clone(),
            ansible_verbosity,
            ansible_passthrough,
            // watch_logs,
            /* kafka_audit,
            k8s, */
        )
        .await?;

    // FIXME: refactor becase this code is duplicated in command `manta apply sat-file` and also in
    // `manta logs`
    if watch_logs {
        log::info!("Fetching logs ...");

        let mut cfs_session_log_stream = backend
            .get_session_logs_stream(shasta_token, site, &cfs_session_name, k8s)
            .await?
            .lines();

        while let Some(line) = cfs_session_log_stream.try_next().await.unwrap() {
            println!("{}", line);
        }
    }

    // Audit
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(shasta_token).unwrap();
        let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": ansible_limit}, "group": vec![hsm_group_opt], "message": "Apply session"});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
    }

    Ok((cfs_configuration_name, cfs_session_name))
}
