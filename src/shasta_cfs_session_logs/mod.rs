pub mod client {

    use hyper::{client::HttpConnector, Uri};
    use k8s_openapi::api::core::v1::Pod;
    use kube::{client::ConfigExt, Api};

    use futures_util::{StreamExt, TryStreamExt};

    use crate::{shasta_cfs_session};

    pub async fn session_logs(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, session_name: &Option<String>, limit_number: &Option<u8>, layer_id: u8) -> core::result::Result<(), Box<dyn std::error::Error>> {

        // Get CFS sessions
        let mut cfs_sessions = shasta_cfs_session::http_client::get(shasta_token, shasta_base_url, cluster_name, session_name, limit_number).await?;

        cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));

        if cfs_sessions.is_empty() {
            log::info!("No CFS session found");
            return Ok(())
        }

        let cfs_session_name: &str = cfs_sessions.last().unwrap()["name"].as_str().unwrap();

        let client = get_k8s_client().await?;

        // Get CFS session logs
        get_pod_logs(client, cfs_session_name, &layer_id.to_string()).await?; // TODO: do we need this method to be async?

        return Ok(())
    }

    pub async fn get_k8s_client() -> core::result::Result<kube::Client, Box<dyn std::error::Error>> {

        let config = kube::Config::infer().await?;
        // log::info!("{:#?}", config);
    
        let connector = {
            let mut http = HttpConnector::new();
            http.enforce_http(false);
            let proxy = hyper_socks2::SocksConnector {
                proxy_addr: Uri::from_static("socks5://localhost:1080"),
                auth: None,
                connector: http,
            };
            let mut native_tls_builder = native_tls::TlsConnector::builder();
            native_tls_builder.danger_accept_invalid_certs(true);
            native_tls_builder.danger_accept_invalid_hostnames(true);
            native_tls_builder.use_sni(false);
    
            // let native_tls_connector = native_tls_builder.build().unwrap();
    
            // let tls = tokio_native_tls::TlsConnector::from(native_tls_connector);
            let tls = tokio_native_tls::TlsConnector::from(config.native_tls_connector()?);
            hyper_tls::HttpsConnector::from((proxy, tls))
        };
    
        let service = tower::ServiceBuilder::new()
            .layer(config.base_uri_layer())
            .option_layer(config.auth_layer()?)
            .service(hyper::Client::builder().build(connector));
    
        let client = kube::Client::new(service, config.default_namespace);
    
        Ok(client)
    }

    // pub async fn get_client() -> core::result::Result<kube::Client, Box<dyn std::error::Error>> {

    //     let config = kube::Config::infer().await?;
    //     // log::info!("{:#?}", config);
    
    //     let connector = {
    //         let mut http = HttpConnector::new();
    //         http.enforce_http(false);
    //         let proxy = hyper_socks2::SocksConnector {
    //             proxy_addr: Uri::from_static("socks5://localhost:1080"),
    //             auth: None,
    //             connector: http,
    //         };
    //         let mut native_tls_builder = native_tls::TlsConnector::builder();
    //         native_tls_builder.danger_accept_invalid_certs(true);
    //         native_tls_builder.danger_accept_invalid_hostnames(true);
    //         native_tls_builder.use_sni(false);
    
    //         // let native_tls_connector = native_tls_builder.build().unwrap();
    
    //         // let tls = tokio_native_tls::TlsConnector::from(native_tls_connector);
    //         let tls = tokio_native_tls::TlsConnector::from(config.native_tls_connector()?);
    //         hyper_tls::HttpsConnector::from((proxy, tls))
    //     };
    
    //     let service = tower::ServiceBuilder::new()
    //         .layer(config.base_uri_layer())
    //         .option_layer(config.auth_layer()?)
    //         .service(hyper::Client::builder().build(connector));
    
    //     let client = kube::Client::new(service, config.default_namespace);
    
    //     Ok(client)
    // }
    
    pub async fn get_pod_logs(client: kube::Client, cfs_session_name: &str, layer_id: &str) -> core::result::Result<(), Box<dyn std::error::Error>> {
    
        let pods: Api<Pod> = Api::namespaced(client, "services");
    
        let params = kube::api::ListParams::default().limit(1).labels(format!("cfsession={}", cfs_session_name).as_str());
    
        let pod = pods.list(&params).await?;
    
        log::info!("Pod name: {:#?}", pod.items.get(0).unwrap().metadata.name.as_ref().unwrap());
    
        let pod_name = pod.items.get(0).unwrap().metadata.name.as_ref().unwrap();
    
        let mut logs = pods
        .log_stream(&pod_name, &kube::api::LogParams {
            follow: true,
            // tail_lines: Some(1),
            container: Some(format!("ansible-{}", layer_id)),
            ..kube::api::LogParams::default()
        }).await?.boxed();
    
        while let Some(line) = logs.try_next().await? {
            print!("{}", std::str::from_utf8(&line).unwrap());
        }
    
        Ok(())
    }
}