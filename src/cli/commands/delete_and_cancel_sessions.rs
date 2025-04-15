use backend_dispatcher::interfaces::cfs::CfsTrait;

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{audit::Audit, jwt_ops, kafka::Kafka},
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_available_vec: Vec<String>,
    cfs_session_name: &str,
    dry_run: &bool,
    assume_yes: bool,
    kafka_audit_opt: Option<&Kafka>,
) {
    let _ = backend
        .delete_and_cancel_session(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_available_vec,
            cfs_session_name,
        )
        .await;

    // Audit
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(shasta_token).unwrap();
        let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "message": format!("delete session '{}'", cfs_session_name)});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
    }
}
