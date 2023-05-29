pub mod http_client {

    use serde_json::Value;

    use std::error::Error;

    use core::result::Result;

    /// Change nodes boot params, ref --> https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/put/
    pub async fn put(
        shasta_base_url: &str,
        shasta_token: &str,
        xnames: &Vec<String>,
        params: &String,
        kernel: &String,
        initrd: &String,
    ) -> Result<Vec<Value>, Box<dyn Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            log::debug!("SOCKS5 enabled");
            let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;

            // rest client to authenticate
            client = client_builder.proxy(socks5proxy).build()?;
        } else {
            client = client_builder.build()?;
        }

        let api_url = format!("{}/bss/boot/v1/bootparameters", shasta_base_url);

        let resp = client
            .put(api_url)
            .json(&serde_json::json!({"hosts": xnames, "params": params, "kernel": kernel, "initrd": initrd})) // Encapsulating configuration.layers
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

    /// Get node boot params, ref --> https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/get/
    pub async fn get_boot_params(
        shasta_token: &str,
        shasta_base_url: &str,
        xnames: &[String],
    ) -> Result<Vec<Value>, Box<dyn Error>> {
        let client;

        let client_builder = reqwest::Client::builder().danger_accept_invalid_certs(true);

        // Build client
        if std::env::var("SOCKS5").is_ok() {
            // socks5 proxy
            log::debug!("SOCKS5 enabled");
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
