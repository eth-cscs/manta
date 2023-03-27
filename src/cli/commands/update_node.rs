use crate::{
    common::node_ops,
    shasta::{bss, cfs, ims},
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    // cli_update_node: &ArgMatches,
    xnames: Vec<&str>,
    cfs_configuration: Option<&String>,
    hsm_group_name: Option<&String>,
) {
    // Check user has provided valid XNAMES
    if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group_name).await {

        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    // Get most recent CFS session target image for the node
    let mut cfs_sessions_details = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        None,
        Some(true),
    )
    .await
    .unwrap();

    cfs_sessions_details
        .retain(|cfs_session_details| cfs_session_details["target"]["definition"].eq("image")); // We
                                                                                                // could
                                                                                                // also
                                                                                                // do
                                                                                                // filter(...)
                                                                                                // and
                                                                                                // collect() here

    if cfs_configuration.is_some() {
        // Filter CFS sessions of target definition "image" and configuration
        cfs_sessions_details.retain(|cfs_session_details| {
            // println!("cfs_session_details:\n{:#?}", cfs_session_details);
            cfs_session_details["configuration"]["name"]
                .as_str()
                .unwrap()
                .to_string()
                .eq(cfs_configuration.unwrap())
        });

        // log::info!("cfs_sessions_details:\n{:#?}", cfs_sessions_details);

        if cfs_sessions_details.is_empty() {
            eprintln!("No image found related to the CFS configuration provided. Exit",);
            std::process::exit(1);
        }
    } else {
        cfs_sessions_details = vec![cfs_sessions_details.last().unwrap().to_owned()];
    }

    log::info!("cfs_sessions_details:\n{:#?}", cfs_sessions_details);

    let result_id = cfs_sessions_details.first().unwrap()["status"]["artifacts"]
        .as_array()
        .unwrap()
        .first()
        .unwrap()["result_id"]
        .as_str()
        .unwrap();

    /* let image_details =
        ims::image::http_client::get(shasta_token, shasta_base_url, result_id).await;

    let ims_image_etag = image_details.as_ref().unwrap()["link"]["etag"]
        .as_str()
        .unwrap()
        .to_string(); */

    let ims_image_etag =
        ims::image::utils::get_image_etag_from_image_id(shasta_token, shasta_base_url, result_id)
            .await;

    let boot_params = format!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{image_id}/rootfs:{etag}-226:dvs:api-gw-service-nmn.local:300:nmn0 nmd_data=url=s3://boot-images/{image_id}/rootfs,etag={etag}-226", image_id=result_id, etag=ims_image_etag );

    let kernel = format!("s3://boot-images/{image_id}/kernel", image_id = result_id);

    let initrd = format!("s3://boot-images/{image_id}/initrd", image_id = result_id);

    for xname in xnames {
        // let xname = xnames_params;

        let xname = xname.trim().to_string();

        /* let hsm_group_aux =
        hsm::utils::get_hsm_group_from_xname(shasta_token, shasta_base_url, &xname.to_string())
            .await; */

        let _update_node_boot_params_response = bss::http_client::put(
            shasta_base_url,
            shasta_token,
            &vec![xname.to_string()],
            &boot_params,
            &kernel,
            &initrd,
        )
        .await;

        println!(
            "Node {} boot params have been updated to image_id {}",
            xname, result_id
        );
    }
}
