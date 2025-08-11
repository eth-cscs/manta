use manta_backend_dispatcher::{
  interfaces::cfs::CfsTrait,
  types::cfs::cfs_configuration_details::{ConfigurationDetails, LayerDetails},
  types::cfs::cfs_configuration_response::CfsConfigurationResponse,
};

use crate::{
  common::cfs_configuration_utils::print_table_struct,
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use chrono::NaiveDateTime;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  gitea_base_url: &str,
  gitea_token: &str,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  configuration_name: Option<&String>,
  configuration_name_pattern: Option<&String>,
  hsm_group_name_vec: &Vec<String>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  limit: Option<&u8>,
  output_opt: Option<&String>,
  site_name: &str,
) {
  let cfs_configuration_vec: Vec<CfsConfigurationResponse> = backend
    .get_and_filter_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration_name.map(|elem| elem.as_str()),
      configuration_name_pattern.map(|elem| elem.as_str()),
      hsm_group_name_vec,
      since_opt,
      until_opt,
      limit,
    )
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not fetch configurations. Reason:\n{:#?}", e);
      std::process::exit(1);
    });

  if cfs_configuration_vec.is_empty() {
    println!("No CFS configuration found!");
    std::process::exit(0);
  }

  if output_opt.is_some() && output_opt.unwrap().eq("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&cfs_configuration_vec).unwrap()
    );
  } else {
    if cfs_configuration_vec.len() == 1 {
      // Get CFS configuration details with data from VCS/Gitea
      let most_recent_cfs_configuration: &CfsConfigurationResponse =
        &cfs_configuration_vec.first().unwrap();

      let mut layer_details_vec: Vec<LayerDetails> = vec![];

      for layer in &most_recent_cfs_configuration.layers {
        let layer_details: LayerDetails = backend
                    .get_configuration_layer_details(
                        shasta_root_cert,
                        gitea_base_url,
                        gitea_token,
                        layer.clone(),
                        site_name,
                    )
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!(
                            "ERROR - Could not fetch configuration layer details. Reason:\n{:#?}",
                            e
                        );
                        std::process::exit(1);
                    });

        layer_details_vec.push(layer_details);
      }

      let (cfs_session_vec_opt, bos_sessiontemplate_vec_opt, image_vec_opt) = backend
                .get_derivatives(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &most_recent_cfs_configuration.name,
                )
                .await
                .unwrap_or_else(|e| {
                    eprintln!(
                        "ERROR - Could not fetch configuration derivatives. Reason:\n{:#?}",
                        e
                    );
                    std::process::exit(1);
                });

      crate::common::cfs_configuration_utils::print_table_details_struct(
        ConfigurationDetails::new(
          &most_recent_cfs_configuration.name,
          &most_recent_cfs_configuration.last_updated,
          layer_details_vec,
        ),
        cfs_session_vec_opt,
        bos_sessiontemplate_vec_opt,
        image_vec_opt,
      );
    } else {
      print_table_struct(&cfs_configuration_vec);
    }
  }
}
