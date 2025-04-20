use mesa::{
    bos::{self, template::http_client::v2::types::BosSessionTemplate},
    bss::{self, types::BootParameters},
    cfs::{
        self, component::http_client::v2::types::Component,
        configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse,
        session::http_client::v2::types::CfsSessionGetResponse,
    },
    error::Error,
    ims::{self, image::http_client::types::Image},
};
use tokio::time::Instant;

pub async fn get_configurations_sessions_bos_sessiontemplates_images_components_bootparameters(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
) -> (
    Result<Vec<Component>, Error>,
    Result<Vec<CfsConfigurationResponse>, Error>,
    Result<Vec<CfsSessionGetResponse>, Error>,
    Result<Vec<BosSessionTemplate>, Error>,
    Result<Vec<Image>, Error>,
    Result<Vec<BootParameters>, Error>,
) {
    let start = Instant::now();

    let values = tokio::join!(
        cfs::component::http_client::v2::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
        ),
        cfs::configuration::http_client::v2::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
        ),
        cfs::session::get_and_sort(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
            None,
            None,
            None,
        ),
        bos::template::http_client::v2::get_all(shasta_token, shasta_base_url, shasta_root_cert,),
        ims::image::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert,),
        bss::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, &[],)
    );

    let duration = start.elapsed();
    log::info!("Time elapsed to get CFS configurations, CFS sessions, BOS sessiontemplate, IMS images and BSS bootparameters bundle is: {:?}", duration);

    values
}
