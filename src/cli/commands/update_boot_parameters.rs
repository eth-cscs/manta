use manta_backend_dispatcher::{
  error::Error, interfaces::bss::BootParametersTrait,
  types::bss::BootParameters,
};

use crate::{
  common::{audit::Audit, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  xnames: &String,
  nids: Option<&String>,
  macs: Option<&String>,
  params: Option<&String>,
  kernel: Option<&String>,
  initrd: Option<&String>,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  println!("Update boot parameters");

  let hosts: Vec<String> = xnames.split(',').map(String::from).collect();
  let macs: Option<Vec<String>> =
    macs.map(|x| x.split(',').map(String::from).collect());
  let nids: Option<Vec<u32>> = nids.cloned().map(|x| {
    x.split(',')
      .map(|x| x.to_string().parse::<u32>().unwrap_or_default())
      .collect()
  });
  let params: String = params.cloned().unwrap_or_default().to_string();
  let kernel: String = kernel.cloned().unwrap_or_default().to_string();
  let initrd: String = initrd.cloned().unwrap_or_default().to_string();

  let boot_parameters = BootParameters {
    hosts: hosts.clone(),
    macs,
    nids,
    params,
    kernel,
    initrd,
    cloud_init: None,
  };

  log::debug!("new boot params: {:#?}", boot_parameters);

  backend
    .update_bootparameters(shasta_token, &boot_parameters)
    .await?;

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).unwrap_or_default();
    let user_id =
      jwt_ops::get_preferred_username(shasta_token).unwrap_or_default();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": hosts}, "message": format!("Update boot parameters")});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }

  Ok(())
}
