use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
struct PowerStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    xnames: Vec<String>,
    force: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    recursive: Option<bool>,
}

impl PowerStatus {
    pub fn new(
        reason: Option<String>,
        xnames: Vec<String>,
        force: bool,
        recursive: Option<bool>,
    ) -> Self {
        Self {
            reason,
            xnames,
            force,
            recursive,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct NodeStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xnames: Option<Vec<String>>,
}

impl NodeStatus {
    pub fn new(
        filter: Option<String>,
        xnames: Option<Vec<String>>,
        source: Option<String>,
    ) -> Self {
        Self {
            filter,
            source,
            xnames,
        }
    }
}

pub mod http_client {

    pub mod node_power_off {

        use core::time;
        use std::{error::Error, thread};

        use serde_json::Value;

        use crate::shasta::{capmc::PowerStatus, hsm};

        pub async fn post(
            shasta_token: &str,
            shasta_base_url: &str,
            xnames: Vec<String>,
            reason: Option<String>,
            force: bool,
        ) -> Result<Value, Box<dyn Error>> {
            log::info!("Shutting down nodes: {:?}", xnames);

            let power_off = PowerStatus::new(reason, xnames, force, None);

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

            let api_url = shasta_base_url.to_owned() + "/capmc/capmc/v1/xname_off";

            let resp = client
                .post(api_url)
                .bearer_auth(shasta_token)
                .json(&power_off)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(resp.json::<Value>().await?)
            } else {
                Err(resp.json::<Value>().await?["detail"]
                    .as_str()
                    .unwrap()
                    .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
            }
        }
        /// Shut down a node
        /// This is  sync call meaning it won't return untill the target is down
        pub async fn post_sync(
            shasta_token: &str,
            shasta_base_url: &str,
            xnames: Vec<String>,
            reason: Option<String>,
            force: bool,
        ) -> Result<Value, Box<dyn Error>> {
            let xname_list: Vec<String> = xnames.into_iter().collect();
            // Create CAPMC operation shutdown
            let capmc_shutdown_nodes_resp = post(
                shasta_token,
                shasta_base_url,
                xname_list.clone(),
                reason,
                force,
            )
            .await;

            log::info!("Shutdown nodes resp:\n{:#?}", capmc_shutdown_nodes_resp);

            // Check Nodes are shutdown
            let mut nodes_status = hsm::http_client::get_components_status(
                shasta_token,
                shasta_base_url,
                xname_list.clone(),
            )
            .await;

            log::info!("nodes_status:\n{:#?}", nodes_status);

            // Check all nodes are OFF
            let mut i = 0;
            let max = 60;
            while i <= max
                && !nodes_status.as_ref().unwrap()["Components"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|node| node["State"].as_str().unwrap().to_string().eq("Off"))
            {
                print!(
                    "\rWaiting nodes to shutdown. Trying again in 2 seconds. Attempt {} of {}",
                    i + 1,
                    max
                );
                thread::sleep(time::Duration::from_secs(2));
                i += 1;
                log::info!("nodes_status:\n{:#?}", nodes_status);
                nodes_status = hsm::http_client::get_components_status(
                    shasta_token,
                    shasta_base_url,
                    xname_list.clone(),
                )
                .await;
            }

            println!();

            log::info!("node status resp:\n{:#?}", nodes_status);

            capmc_shutdown_nodes_resp
        }
    }

    pub mod node_power_on {
        use std::error::Error;

        use serde_json::Value;

        use crate::shasta::capmc::PowerStatus;

        pub async fn post(
            shasta_token: &str,
            shasta_base_url: &str,
            xnames: Vec<String>,
            reason: Option<String>,
            force: bool,
        ) -> Result<Value, Box<dyn Error>> {
            log::info!("Powering on nodes: {:?}", xnames);

            let power_on = PowerStatus::new(reason, xnames, force, None);

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

            let api_url = shasta_base_url.to_owned() + "/capmc/capmc/v1/xname_on";

            let resp = client
                .post(api_url)
                .bearer_auth(shasta_token)
                .json(&power_on)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(resp.json::<Value>().await?)
            } else {
                Err(resp.json::<Value>().await?["detail"]
                    .as_str()
                    .unwrap()
                    .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
            }
        }
    }

    pub mod node_power_restart {

        use std::error::Error;

        use serde_json::Value;

        use crate::shasta::capmc::PowerStatus;

        pub async fn post(
            shasta_token: &str,
            shasta_base_url: &str,
            reason: Option<&String>,
            xnames: Vec<String>,
            force: bool,
        ) -> Result<Value, Box<dyn Error>> {
            log::info!("Restarting nodes: {:?}", xnames);

            let node_restart = PowerStatus::new(reason.cloned(), xnames, force, None);

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

            let api_url = shasta_base_url.to_owned() + "/capmc/capmc/v1/xname_reinit";

            let resp = client
                .post(api_url)
                .bearer_auth(shasta_token)
                .json(&node_restart)
                .send()
                .await?;

            if resp.status().is_success() {
                Ok(resp.json::<Value>().await?)
            } else {
                Err(resp.json::<Value>().await?["detail"]
                    .as_str()
                    .unwrap()
                    .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
            }
        }
    }

    pub mod node_power_status {

        use std::error::Error;

        use serde_json::Value;

        use crate::shasta::capmc::NodeStatus;

        pub async fn post(
            shasta_token: &str,
            shasta_base_url: &str,
            xnames: &Vec<String>,
        ) -> core::result::Result<Value, Box<dyn Error>> {
            log::info!("Checking nodes status: {:?}", xnames);

            let node_status_payload =
                NodeStatus::new(None, Some(xnames.clone()), Some("hsm".to_string()));

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

            let url_api = shasta_base_url.to_owned() + "/capmc/capmc/v1/get_xname_status";

            let resp = client
                .post(url_api)
                .bearer_auth(shasta_token)
                .json(&node_status_payload)
                .send()
                .await?;

            if resp.status().is_success() {
                let resp_json = &resp.json::<Value>().await?;
                Ok(resp_json.clone())
            } else {
                Err(resp.json::<Value>().await?["detail"]
                    .as_str()
                    .unwrap()
                    .into()) // Black magic conversion from Err(Box::new("my error msg")) which does not
            }
        }
    }
}
