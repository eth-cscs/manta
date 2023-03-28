pub mod http_client {

    use std::error::Error;

    use serde_json::Value;

    /// Fetch IMS image ref --> https://apidocs.svc.cscs.ch/paas/ims/operation/get_v3_image/
    pub async fn get(
        shasta_token: &str,
        shasta_base_url: &str,
        image_id: &str,
    ) -> Result<Value, Box<dyn Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let api_url = shasta_base_url.to_owned() + "/ims/v3/images/" + image_id;

        let resp = client
            .get(api_url)
            // .get(format!("{}{}", shasta_base_url, "/cfs/v2/configurations"))
            .bearer_auth(shasta_token)
            .send()
            .await?;

        if resp.status().is_success() {
            let response = &resp.text().await?;
            Ok(serde_json::from_str(response)?)
        } else {
            eprintln!("FAIL request: {:#?}", resp);
            let response: String = resp.text().await?;
            eprintln!("FAIL response: {:#?}", response);
            Err(response.into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }
}

pub mod utils {
    pub async fn get_image_etag_from_image_id(
        shasta_token: &str,
        shasta_base_url: &str,
        image_id: &str,
    ) -> String {
        let ims_image_etag =
            crate::shasta::ims::image::http_client::get(shasta_token, shasta_base_url, image_id)
                .await
                .unwrap();

        ims_image_etag["link"]["etag"]
            .as_str()
            .map(|etag| etag.to_string())
            .unwrap()
    }
}
