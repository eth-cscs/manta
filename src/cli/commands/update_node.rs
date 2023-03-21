use clap::ArgMatches;

use crate::shasta::{bss, cfs, hsm, ims};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    cli_update_node: &ArgMatches,
    hsm_group_name: Option<&String>,
) {
    let xnames_params = cli_update_node.get_one::<String>("XNAMES").unwrap();

    // TODO: deal when user sends a string with multiple xnames... what should I do???
    if xnames_params.split(',').count() != 1 {
        // user has sent a string with multiple xnames!!!!
    }

    let xname = xnames_params;

    let hsm_group_aux =
        hsm::utils::get_hsm_group_from_xname(shasta_token, shasta_base_url, xname).await;

    if hsm_group_aux.is_none()
        || (hsm_group_name.is_some()
            && !hsm_group_name.unwrap().eq(hsm_group_aux.as_ref().unwrap()))
    {
        eprintln!(
            "xname {} does not belongs to HSM group {}. Exit",
            xname,
            hsm_group_name.unwrap()
        );
        std::process::exit(1);
    }

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
    let mut cfs_sessions_details = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        Some(&1),
        Some(true),
    )
    .await
    .unwrap();

    log::info!("cfs_sessions_details:\n{:#?}", cfs_sessions_details);

    // Filter CFS sessions of target definition "image"
    cfs_sessions_details
        .retain(|cfs_session_details| cfs_session_details["target"]["definition"].eq("image"));

    log::info!("cfs_sessions_details:\n{:#?}", cfs_sessions_details);

    if cfs_sessions_details.is_empty() {
        eprintln!("Can't continue can't find any CFS session target definition 'image' linked to this node. Exit");
        std::process::exit(1);
    }

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
        &vec![xname.to_string()],
        &params,
        &kernel,
        &initrd,
    )
    .await;

    println!(
        "Node {} boot params have been updated to image_id {}",
        xname, result_id
    );
}
