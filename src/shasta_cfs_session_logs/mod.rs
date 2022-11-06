pub mod client {

    use core::time;
    use std::{thread, str::FromStr, error::Error};

    use hyper::{client::HttpConnector, Uri};
    use k8s_openapi::api::core::v1::{Pod, ContainerState};
    use kube::{client::ConfigExt, Api, Client, config::{KubeConfigOptions, Cluster, NamedCluster, NamedAuthInfo, AuthInfo, Kubeconfig, NamedContext, Context}};

    use futures_util::{StreamExt, TryStreamExt};
    use secrecy::SecretString;

    use crate::{shasta_cfs_session, vault::http_client::fetch_shasta_k8s_secrets};

    // pub async fn get_k8s_client_from_env() -> Result<kube::Client, Box<dyn Error>> {

    //     let config = kube::Config::infer().await?;
    //     // log::info!("{:#?}", config);
    
    //     let client;

    //     if std::env::var("SOCKS5").is_ok() {
    //         let connector = {
    //             let mut http = HttpConnector::new();
    //             http.enforce_http(false);
    //             let proxy = hyper_socks2::SocksConnector {
    //                 proxy_addr: std::env::var("SOCKS5").unwrap().parse::<Uri>().unwrap(),
    //                 auth: None,
    //                 connector: http,
    //             };
    //             let mut native_tls_builder = native_tls::TlsConnector::builder();
    //             native_tls_builder.danger_accept_invalid_certs(true);
    //             native_tls_builder.danger_accept_invalid_hostnames(true);
    //             native_tls_builder.use_sni(false);
        
    //             // let native_tls_connector = native_tls_builder.build().unwrap();
        
    //             // let tls = tokio_native_tls::TlsConnector::from(native_tls_connector);
    //             let tls = tokio_native_tls::TlsConnector::from(config.native_tls_connector()?);
    //             hyper_tls::HttpsConnector::from((proxy, tls))
    //         };

    //         let service = tower::ServiceBuilder::new()
    //         .layer(config.base_uri_layer())
    //         .option_layer(config.auth_layer()?)
    //         .service(hyper::Client::builder().build(connector));
    
    //         client = kube::Client::new(service, config.default_namespace);

    //     } else {

    //         client = Client::try_default().await?;

    //     }
    
    //     Ok(client)
    // }

    pub async fn get_k8s_client_programmatically() -> Result<kube::Client, Box<dyn Error>> {

        let shasta_k8s_secrets = fetch_shasta_k8s_secrets().await?;
        // log::info!("\n{:#?}", fetch_shasta_k8s_secrets().await);

        let mut shasta_cluster = Cluster {
            server: String::from("https://10.252.1.12:6442"),
            // server: String::from("https://api.alps.cscs.ch:6442"),
            insecure_skip_tls_verify: Some(true),
            certificate_authority: None,
            certificate_authority_data: Some(String::from(shasta_k8s_secrets["certificate-authority-data"].as_str().unwrap())),
            proxy_url: None,
            extensions: None,
        };

        match std::env::var("SOCKS5") {
            Ok(socks_proxy) => shasta_cluster.proxy_url = Some(socks_proxy),
            Err(_) => log::info!("socks proxy not provided")
        }

        let shasta_named_cluster = NamedCluster {
            name: String::from("shasta"),
            cluster: shasta_cluster,
        };

        let shasta_auth_info = AuthInfo {
            username: None,
            password: None,
            token: None,
            token_file: None,
            client_certificate: None,
            client_certificate_data: Some(String::from(shasta_k8s_secrets["client-certificate-data"].as_str().unwrap())),
            client_key: None,
            client_key_data: Some(SecretString::from_str(shasta_k8s_secrets["client-key-data"].as_str().unwrap()).unwrap()),
            impersonate: None,
            impersonate_groups: None,
            auth_provider: None,
            exec: None,
        };

        let shasta_named_auth_info = NamedAuthInfo {
            name: String::from("kubernetes-admin"),
            auth_info: shasta_auth_info,
        };

        let shasta_context = Context {
            cluster: String::from("shasta"),
            user: String::from("kubernetes-admin"),
            namespace: None,
            extensions: None,
        };

        let shasta_named_context = NamedContext {
            name: String::from("kubernetes-admin@kubernetes"),
            context: shasta_context,
        };

        let kube_config = Kubeconfig {
            preferences: None,
            clusters: vec![shasta_named_cluster],
            auth_infos: vec![shasta_named_auth_info],
            contexts: vec![shasta_named_context],
            current_context: Some(String::from("kubernetes-admin@kubernetes")),
            extensions: None,
            kind: None,
            api_version: None
        };

        let kube_config_options = KubeConfigOptions {
            context: Some(String::from("kubernetes-admin@kubernetes")),
            cluster: Some(String::from("shasta")),
            user: Some(String::from("kubernetes-admin"))
        };

        let config = kube::Config::from_custom_kubeconfig(kube_config, &kube_config_options).await?;
    
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
    
    pub async fn session_logs_proxy(shasta_token: &str, shasta_base_url: &str, cluster_name: &Option<String>, session_name: &Option<String>, layer_id: Option<u8>) -> Result<(), Box<dyn Error>> {
        
        // Get CFS sessions
        let cfs_sessions = shasta_cfs_session::http_client::get(shasta_token, shasta_base_url, cluster_name, session_name, &None).await?;

        // cfs_sessions.sort_by(|a, b| a["status"]["session"]["startTime"].to_string().cmp(&b["status"]["session"]["startTime"].to_string()));

        if cfs_sessions.is_empty() {
            log::info!("No CFS session found");
            return Ok(())
        }

        let cfs_session_name: &str = cfs_sessions.last().unwrap()["name"].as_str().unwrap();

        let client = get_k8s_client_programmatically().await?;

        // Get CFS session logs
        get_pod_logs(client, cfs_session_name, layer_id).await?;

        Ok(())
    }

    pub async fn session_logs(cfs_session_name: &str, layer_id: Option<u8>) -> core::result::Result<(), Box<dyn std::error::Error>> {
        
        let client = get_k8s_client_programmatically().await?;

        // Get CFS session logs
        get_pod_logs(client, cfs_session_name, layer_id).await?;

        Ok(())
    }

    fn get_container_state(pod: &Pod, container_name: &String) -> Option<ContainerState> {
        let container_status = pod.status.as_ref().unwrap().container_statuses.as_ref().unwrap()
        .iter().filter(|container_status| container_status.name.eq(container_name))
        .next();

        match container_status {
            Some(container_status_aux) => container_status_aux.state.clone(),
            None => None
        }
    }

    pub async fn get_pod_logs(client: kube::Client, cfs_session_name: &str, layer_id: Option<u8>) -> Result<(), Box<dyn Error>> {
    
        let pods_api: Api<Pod> = Api::namespaced(client, "services");

        log::debug!("cfs session: {}", cfs_session_name);
    
        let params = kube::api::ListParams::default().limit(1).labels(format!("cfsession={}", cfs_session_name).as_str());
        
        let mut pods = pods_api.list(&params).await?;
        
        let mut i = 0;

        // Waiting for pod to start
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

        let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
        log::info!("Pod name: {}", cfs_session_pod_name);

        if layer_id.is_some() { // Printing logs of 1 CFS session layer

            let layer = layer_id.unwrap().to_string();

            let container_name = format!("ansible-{}", layer);
        
            // Check if container exists in pod
            let container_exists = cfs_session_pod.spec.as_ref().unwrap().containers.iter().find(|x| x.name.eq(&container_name));
    
            if container_exists.is_none() {
                println!("Container {} (layer {}) does not exists. Aborting", container_name, layer);
                std::process::exit(0);
            }
    
            let mut container_state = get_container_state(&cfs_session_pod, &container_name);
    
            let mut i = 0;
    
            // Waiting for container ansible-x to start
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
            .log_stream(
                &cfs_session_pod_name, 
                &kube::api::LogParams {
                    follow: true,
                    // tail_lines: Some(1),
                    container: Some(container_name),
                    ..kube::api::LogParams::default()
                }
            ).await?.boxed();
            
            while let Some(line) = logs.try_next().await? {
                print!("{}", std::str::from_utf8(&line).unwrap());
            }
        } else { // Printing logs of all CFS layers

            // Get ansible-x containers
            let ansible_containers = cfs_session_pod.spec.as_ref().unwrap().containers.iter().filter(|x| x.name.contains("ansible-"));

            for layer in 0..ansible_containers.count() {
                
                let container_name = format!("ansible-{}", layer);

                log::info!("Printing logs for container {}", container_name);
                
                // Check if container exists in pod
                let container_exists = cfs_session_pod.spec.as_ref().unwrap().containers.iter().find(|x| x.name.eq(&container_name));
        
                if container_exists.is_none() {
                    println!("Container {} (layer {}) does not exists. Aborting", container_name, layer);
                    std::process::exit(0);
                }
        
                let mut container_state = get_container_state(&cfs_session_pod, &container_name);
        
                let mut i = 0;
        
                // Waiting for container ansible-x to start
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
                .log_stream(
                    &cfs_session_pod_name, 
                    &kube::api::LogParams {
                        follow: true,
                        // tail_lines: Some(1),
                        container: Some(container_name),
                        ..kube::api::LogParams::default()
                    }
                ).await?.boxed();
                
                while let Some(line) = logs.try_next().await? {
                    print!("{}", std::str::from_utf8(&line).unwrap());
                }
            }
        }

        Ok(())
    }
}