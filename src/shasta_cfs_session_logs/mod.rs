pub mod client {

    use core::time;
    use std::thread;

    use hyper::{client::HttpConnector, Uri};
    use k8s_openapi::api::core::v1::{Pod, ContainerState};
    use kube::{client::ConfigExt, Api, Client};

    use futures_util::{StreamExt, TryStreamExt};

    use crate::{shasta_cfs_session};

    pub async fn get_k8s_client() -> core::result::Result<kube::Client, Box<dyn std::error::Error>> {

        let config = kube::Config::infer().await?;
        // log::info!("{:#?}", config);
    
        let client;

        if std::env::var("SOCKS5").is_ok() {
            let connector = {
                let mut http = HttpConnector::new();
                http.enforce_http(false);
                let proxy = hyper_socks2::SocksConnector {
                    proxy_addr: std::env::var("SOCKS5").unwrap().parse::<Uri>().unwrap(),
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
    
            client = kube::Client::new(service, config.default_namespace);

        } else {

            client = Client::try_default().await?;

        }
    
        Ok(client)
    }
    
    pub async fn session_logs_proxy(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, session_name: &Option<String>, layer_id: u8) -> core::result::Result<(), Box<dyn std::error::Error>> {
        
        // Get CFS sessions
        let cfs_sessions = shasta_cfs_session::http_client::get(shasta_token, shasta_base_url, cluster_name, session_name, &None).await?;

        // cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));

        if cfs_sessions.is_empty() {
            log::info!("No CFS session found");
            return Ok(())
        }

        let cfs_session_name: &str = cfs_sessions.last().unwrap()["name"].as_str().unwrap();

        let client = get_k8s_client().await?;

        // Get CFS session logs
        get_pod_logs(client, cfs_session_name, &layer_id.to_string()).await?; // TODO: do we need this method to be async?

        Ok(())
    }

    pub async fn session_logs(cfs_session_name: &str, layer_id: u8) -> core::result::Result<(), Box<dyn std::error::Error>> {
        
        let client = get_k8s_client().await?;

        // Get CFS session logs
        get_pod_logs(client, cfs_session_name, &layer_id.to_string()).await?; // TODO: do we need this method to be async?

        Ok(())
    }

    // /// Returns True is container is ready/running
    // fn is_container_ready(pod: &Pod, container_name: &String) -> bool {
    //     pod.status.as_ref().unwrap().container_statuses.as_ref().unwrap().iter().filter(|container_status| container_status.name.eq(container_name)).next().unwrap().ready
    // }

    // /// Returns true if container ran and already finished
    // fn is_container_terminated(pod: &Pod, container_name: &String) -> bool {
    //     pod.status.as_ref().unwrap().container_statuses.as_ref().unwrap().iter().filter(|container_status| container_status.name.eq(container_name)).next().unwrap().state.is_some()
    // }

    fn get_container_state(pod: &Pod, container_name: &String) -> Option<ContainerState> {
        pod.status.as_ref().unwrap().container_statuses.as_ref().unwrap()
            .iter().filter(|container_status| container_status.name.eq(container_name))
            .next().unwrap().state.clone()
    }

    // fn is_container_waiting(pod: Pod, container_name: &String) -> bool {
    //     let container_state = get_container_state(&pod, container_name);

    //     container_state.is_some() && container_state.as_ref().unwrap().waiting.is_some()
    // }

    // fn is_container_running(pod: Pod, container_name: &String) -> bool {
    //     let container_state = get_container_state(&pod, container_name);
        
    //     container_state.is_some() && container_state.as_ref().unwrap().running.is_some()
    // }


    pub async fn get_pod_logs(client: kube::Client, cfs_session_name: &str, layer_id: &str) -> core::result::Result<(), Box<dyn std::error::Error>> {
    
        let pods_api: Api<Pod> = Api::namespaced(client, "services");

        log::debug!("cfs session: {}", cfs_session_name);
    
        let params = kube::api::ListParams::default().limit(1).labels(format!("cfsession={}", cfs_session_name).as_str());
        
        let mut pods = pods_api.list(&params).await?;
        
        let mut i = 0;

        while pods.items.is_empty() && i < 10 {
            log::info!("Pod for cfs session {} not ready. Trying again in 2 secs. Attempt {} of 10", cfs_session_name, i + 1);
            i += 1;
            thread::sleep(time::Duration::from_secs(2));
            pods = pods_api.list(&params).await?;
        }

        if pods.items.is_empty() {
            log::info!("Pod for cfs session {} not ready. Aborting operation", cfs_session_name);
            std::process::exit(1);
        }

        let mut cfs_session_pod = &pods.items[0];

        let container_name = format!("ansible-{}", layer_id);

        let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();

        log::info!("Pod name: {}", cfs_session_pod_name);

        // let mut container_ready = is_container_terminated(cfs_session_pod, &container_name);

        // log::info!("Container state:\n{:#?}", container_state(cfs_session_pod, &container_name));

        let mut container_state = get_container_state(&cfs_session_pod, &container_name);

        let mut i = 0;

        while container_state.as_ref().unwrap().waiting.is_some() && i < 10 {
            log::info!("Waiting for container {} to be ready. Checking again in 2 secs. Attempt {} of 10", container_name, i + 1);
            i += 1;
            thread::sleep(time::Duration::from_secs(2));
            pods = pods_api.list(&params).await?;
            cfs_session_pod = &pods.items[0];
            container_state = get_container_state(cfs_session_pod, &container_name);
            log::debug!("Container state:\n{:#?}", container_state.as_ref().unwrap());
        }

        if container_state.as_ref().unwrap().waiting.is_some() {
            log::info!("Container {} not ready. Aborting operation", container_name);
            std::process::exit(1);
        }

        let mut logs = pods_api
        .log_stream(&cfs_session_pod_name, &kube::api::LogParams {
            follow: true,
            // tail_lines: Some(1),
            container: Some(container_name),
            ..kube::api::LogParams::default()
        }).await?.boxed();
        
        while let Some(line) = logs.try_next().await? {
            print!("{}", std::str::from_utf8(&line).unwrap());
        }
        
        Ok(())
    }
}