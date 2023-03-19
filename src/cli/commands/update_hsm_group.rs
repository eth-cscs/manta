use clap::ArgMatches;

use crate::shasta::{bss, cfs, hsm, ims};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    cli_update_node: &ArgMatches,
    hsm_group_name: &String,
) {
    let hsm_group_details =
        hsm::http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group_name).await;

    let xnames = match hsm_group_details {
        Err(_) => {
            eprintln!("HSM group {} not found. Exit", hsm_group_name);
            std::process::exit(1);
        }
        Ok(hsm_group_details) => hsm::utils::get_member_ids(&hsm_group_details),
    };

    /* if hsm_group_name.is_some() {
        let hsm_group_details =
            hsm::http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group_name.unwrap())
            .await;

        let hsm_group_members = hsm::utils::get_member_ids(&hsm_group_details.unwrap());

        if !hsm_group_members.iter().any(|x| x == "xname") {
            eprintln!("xname {} does not belongs to HSM group {:?}. Exit", xname, hsm_group_members);
            std::process::exit(1);
        }
    } */

    // Get most recent CFS session target image for the node
    let cfs_sessions_details = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        Some(hsm_group_name),
        None,
        Some(&1),
        Some(true),
    )
    .await
    .unwrap();

    let result_id = cfs_sessions_details.first().unwrap()["status"]["artifacts"]
        .as_array()
        .unwrap()
        .first()
        .unwrap()["result_id"]
        .as_str()
        .unwrap();

    let image_details =
        ims::image::http_client::get(shasta_token, shasta_base_url, result_id).await;

    let ims_image_etag = image_details.as_ref().unwrap()["link"]["etag"]
        .as_str()
        .unwrap()
        .to_string();

    let params = format!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{image_id}/rootfs:{etag}-226:dvs:api-gw-service-nmn.local:300:nmn0 nmd_data=url=s3://boot-images/{image_id}/rootfs,etag={etag}-226", image_id=result_id, etag=ims_image_etag );

    let kernel = format!("s3://boot-images/{image_id}/kernel", image_id = result_id);

    let initrd = format!("s3://boot-images/{image_id}/initrd", image_id = result_id);

    let _update_node_boot_params_response = bss::http_client::put(
        shasta_base_url,
        shasta_token,
        &xnames,
        &params,
        &kernel,
        &initrd,
    )
    .await;

    println!(
        "Nodes {:?} boot params have been updated to image_id {}",
        xnames, result_id
    );
}
