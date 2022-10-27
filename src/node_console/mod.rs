use std::{error::Error, io::Write};

use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, api::{AttachParams, AttachedProcess}};
use serde_json::Value;
use tokio::{io::{AsyncWriteExt}, runtime::Runtime};

use crate::shasta_cfs_session_logs::client::get_k8s_client_programmatically;

pub async fn connect_to_console(xname: &str) -> Result<(), Box<dyn Error>> {

    let client = get_k8s_client_programmatically().await?;

    let pods_api: Api<Pod> = Api::namespaced(client, "services");

    let params = kube::api::ListParams::default().limit(1).labels("app.kubernetes.io/name=cray-console-operator");
        
    let pods = pods_api.list(&params).await?;

    let console_operator_pod = &pods.items[0];
    let console_operator_pod_name = console_operator_pod.metadata.name.clone().unwrap();
    
    let attached = pods_api.exec(
        &console_operator_pod_name, 
        vec!["sh", "-c", &format!("/app/get-node {}", xname)], 
        &AttachParams::default().container("cray-console-operator").stderr(false)
    ).await?;

    let output = get_exec_command_output(attached).await;
    let output_json: Value = serde_json::from_str(&output)?;

    let console_pod_name = output_json["podname"].as_str().unwrap();

    log::info!("Connecting to console {} - kubectl -n services exec -it {} -c cray-console-node -- sh", xname, console_pod_name);

    let mut attached = pods_api.exec(
        &console_pod_name, 
        vec!["conman", "-j", xname], 
        // vec!["sh"],
        &AttachParams::default()
            .container("cray-console-node")
            .stdin(true)
            .stdout(true)
            .stderr(false) // Note to self: tty and stderr cannot both be true
            .tty(true)
    ).await?;

    let mut user_input;
    let stdin = std::io::stdin(); // We get `Stdin` here.
    
    let mut stdin_writer = attached.stdin().unwrap();
    let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());

    let mut next_stdout;
    let mut stdout;

    user_input = String::new();
    
    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        loop {
            // println!("Waiting user input");

            stdin.read_line(&mut user_input).unwrap();

            // println!("Writting {} in string and {:?} in bytes", user_input.as_str(), user_input.as_bytes());

            stdin_writer.write_all(format!("{}", user_input).as_bytes()).await.unwrap();
        }
    }); 

    loop {
        // Read command output
        // println!("START READING CMD OUTPUT??");
        next_stdout = stdout_stream.next().await;
        stdout = String::from_utf8(next_stdout.unwrap()?.to_vec()).unwrap();
        print!("{}", stdout);
        std::io::stdout().flush().unwrap();
        // println!("END READING CMD OUTPUT??");
    }
 
        // // Read echo
        // if (!user_input.is_empty()) {
        //     println!("START READING ECHO??");
        //     next_stdout = stdout_stream.next().await;
        //     stdout = String::from_utf8(next_stdout.unwrap()?.to_vec()).unwrap();
        //     // print!("{}", stdout);
        //     // std::io::stdout().flush().unwrap();
        //     println!("END READING ECHO??");
        // }

        // // Read command output
        // println!("START READING CMD OUTPUT??");
        // next_stdout = stdout_stream.next().await;
        // stdout = String::from_utf8(next_stdout.unwrap()?.to_vec()).unwrap();
        // print!("{}", stdout);
        // std::io::stdout().flush().unwrap();
        // println!("END READING CMD OUTPUT??");

        // // Read shell prompt
        // println!("START READING SHELL PROMPT??");
        // next_stdout = stdout_stream.next().await;
        // stdout = String::from_utf8(next_stdout.unwrap()?.to_vec()).unwrap();
        // print!("{}", stdout);
        // std::io::stdout().flush().unwrap();
        // println!("END READING SHELL PROMPT??");
    // }

}

async fn get_exec_command_output(mut attached: AttachedProcess) -> String {
    let stdout = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let out = stdout
        .filter_map(|r| async { r.ok().and_then(|v| String::from_utf8(v.to_vec()).ok()) })
        .collect::<Vec<_>>()
        .await
        .join("");
    attached.join().await.unwrap();
    out
}
