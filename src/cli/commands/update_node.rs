use crate::common::{ims_ops::get_image_id_from_cfs_configuration_name, node_ops};

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::shasta::{capmc, ims};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    boot_image_configuration_opt: Option<&String>,
    desired_configuration_opt: Option<&String>,
    xnames: Vec<&str>,
) {
    let need_restart = boot_image_configuration_opt.is_some();

    // Check desired configuration exists
    if let Ok(desired_configuration_detail_list) =
        mesa::shasta::cfs::configuration::http_client::get(
            shasta_token,
            shasta_base_url,
            desired_configuration_opt,
            Some(&1),
        )
        .await
    {
        log::debug!(
            "CFS configuration resp:\n{:#?}",
            desired_configuration_detail_list
        );

        desired_configuration_detail_list.first().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string()
    } else {
        eprintln!(
            "Desired configuration {} does not exists. Exit",
            desired_configuration_opt.unwrap()
        );
        std::process::exit(1);
    };

    // Check user has provided valid XNAMES
    if hsm_group_name.is_some()
        && !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group_name).await
    {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    if need_restart {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "This operation will reboot nodes {:?}. Do you want to continue?",
                xnames
            ))
            .interact()
            .unwrap()
        {
            log::info!("Continue",);
        } else {
            println!("Cancelled by user. Aborting.");
            std::process::exit(0);
        }
    }

    // Process boot parameters
    if let Some(boot_image_cfs_configuration_name) = boot_image_configuration_opt {
        let image_id_opt = get_image_id_from_cfs_configuration_name(
            shasta_token,
            shasta_base_url,
            boot_image_cfs_configuration_name.clone(),
        )
        .await;

        let image_details_value_vec = if let Some(image_id) = image_id_opt {
            ims::image::http_client::get(shasta_token, shasta_base_url, None, Some(&image_id), None)
                .await
                .unwrap()
        } else {
            eprintln!(
                "Image ID related to CFS configuration name {} not found. Exit",
                boot_image_cfs_configuration_name
            );
            std::process::exit(1);
        };

        log::debug!("image_details:\n{:#?}", image_details_value_vec);

        log::info!("image_details_value_vec:\n{:#?}", image_details_value_vec);

        let image_path = image_details_value_vec.first().unwrap()["link"]["path"]
            .as_str()
            .unwrap()
            .to_string();

        let image_id = image_path
            .strip_prefix("s3://boot-images/")
            .unwrap()
            .strip_suffix("/manifest.json")
            .unwrap()
            .to_string();

        let component_patch_rep = mesa::shasta::bss::http_client::patch(
            shasta_base_url,
            shasta_token,
            &xnames.iter().map(|&xname| xname.to_string()).collect(),
            Some(&format!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{}/rootfs:37df9a2dc2c4b50679def2193c193c40-230:dvs:api-gw-service-nmn.local:300:nmn0", image_id)),
            Some(&format!("s3://boot-images/{}/kernel", image_id)),
            Some(&format!("s3://boot-images/{}/initrd", image_id)),
        )
        .await;

        log::debug!(
            "Component boot parameters resp:\n{:#?}",
            component_patch_rep
        );
    }

    // Update desired configuration

    if let Some(desired_configuration_name) = desired_configuration_opt {
        log::info!(
            "Updating desired configuration. Need restart? {}",
            need_restart
        );

        mesa::shasta::cfs::component::utils::update_component_list_desired_configuration(
            shasta_token,
            shasta_base_url,
            xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: modify function signature
            // for this field so it accepts
            // Vec<&str> instead of
            // Vec<String>
            desired_configuration_name,
            !need_restart,
        )
        .await;
    }

    // Check if need to reboot
    if need_restart {
        log::info!("Restarting nodes");

        let nodes: Vec<String> = xnames.into_iter().map(|xname| xname.to_string()).collect();

        // Create CAPMC operation shutdown
        let capmc_shutdown_nodes_resp = capmc::http_client::node_power_off::post_sync(
            shasta_token,
            shasta_base_url,
            nodes.clone(),
            Some("Update node boot params and/or desired configuration".to_string()),
            true,
        )
        .await;

        log::debug!(
            "CAPMC shutdown nodes response:\n{:#?}",
            capmc_shutdown_nodes_resp
        );

        // Create CAPMC operation to start
        let capmc_start_nodes_resp = capmc::http_client::node_power_on::post(
            shasta_token,
            shasta_base_url,
            nodes,
            Some("Update node boot params and/or desired configuration".to_string()),
            false,
        )
        .await;

        log::debug!(
            "CAPMC starting nodes response:\n{:#?}",
            capmc_start_nodes_resp
        );
    }
}
