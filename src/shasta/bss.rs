pub mod http_client {

    use serde_json::Value;

    use std::error::Error;

    use core::result::Result;

    /// Get node boot params, uses https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/get/
    pub async fn get_boot_params(
        shasta_token: &String,
        shasta_base_url: &String,
        xnames: &[String],
    ) -> Result<Vec<Value>, Box<dyn Error>> {
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

        let url_api = format!("{}/bss/boot/v1/bootparameters", shasta_base_url);

        let params: Vec<_> = xnames.iter().map(|xname| ("name", xname)).collect();

        let resp = client
            .get(url_api)
            .query(&params)
            .bearer_auth(shasta_token)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json::<Value>().await?.as_array().unwrap().clone())
        } else {
            Err(resp.json::<Value>().await?.as_str().unwrap().into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
        }
    }
}
