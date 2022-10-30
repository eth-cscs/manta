use std::{error::Error, io::{Write, Read, stdout}};

use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, api::{AttachParams, AttachedProcess}};
use serde_json::Value;
use termion::{raw::IntoRawMode, color};
use tokio::{io::{AsyncWriteExt}, runtime::Runtime};

use crate::shasta_cfs_session_logs::client::get_k8s_client_programmatically;

pub async fn connect_to_console(xname: &str) -> Result<(), Box<dyn Error>> {

    let client = get_k8s_client_programmatically().await?;

    let pods_api: Api<Pod> = Api::namespaced(client, "services");

    let params = kube::api::ListParams::default().limit(1).labels("app.kubernetes.io/name=cray-console-operator");
        
    let pods = pods_api.list(&params).await?;

    let console_operator_pod = &pods.items[0];
    let console_operator_pod_name = console_operator_pod.metadata.name.clone().unwrap();
    
    let mut attached = pods_api.exec(
        &console_operator_pod_name, 
        vec!["sh", "-c", &format!("/app/get-node {}", xname)], 
        &AttachParams::default().container("cray-console-operator").stderr(false)
    ).await?;

    // let output = get_exec_command_output(attached).await;
    let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let next_stdout = stdout_stream.next().await;
    let stdout_str = String::from_utf8(next_stdout.unwrap().unwrap().to_vec()).unwrap();
    let output_json: Value = serde_json::from_str(&stdout_str)?;

    let console_pod_name = output_json["podname"].as_str().unwrap();

    let command = vec!["conman", "-j", xname];
    // let command = vec!["bash"];

    log::info!("Alternatively run - kubectl -n services exec -it {} -c cray-console-node -- {}", console_pod_name, command.iter().map(|x| x.to_string() + " ").collect::<String>());

    log::info!("Connecting to console {}", xname);

    let mut attached = pods_api.exec(
        &console_pod_name, 
        command,
        &AttachParams::default()
            .container("cray-console-node")
            .stdin(true)
            .stdout(true)
            .stderr(false) // Note to self: tty and stderr cannot both be true
            .tty(true)
    ).await?;
    
    println!("Connected to {}{}{}!", color::Fg(color::Blue), xname, color::Fg(color::Reset));
    println!("Use {}&.{} key combination to exit the console.", color::Fg(color::Green), color::Fg(color::Reset));


    let mut stdin_writer = attached.stdin().unwrap();
    let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());

    let mut stdin = std::io::stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let rt = Runtime::new().unwrap();
    rt.spawn(async move {

        let mut next_stdout;
        let mut stdout_str;

        loop {

            next_stdout = stdout_stream.next().await;
            stdout_str = String::from_utf8(next_stdout.unwrap().unwrap().to_vec()).unwrap();
            print!("{}", stdout_str);
            stdout.flush().unwrap();
        }
    });

    loop {
        let mut buffer = [0; 1];

        // read up to 10 bytes
        let n = stdin.read(&mut buffer[..])?;

        stdin_writer.write_all(format!("{}", String::from_utf8((&buffer[..n]).to_vec()).unwrap()).as_bytes()).await?;
    }

    // let mut stdin_writer = attached.stdin().unwrap();
    // let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    
    // let stdin = std::io::stdin(); // We get `Stdin` here.
    // let rt = Runtime::new().unwrap();
    // rt.spawn(async move {
    //     let mut user_input = String::new();
    //     loop {
    //         stdin.read_line(&mut user_input).unwrap();
    //         stdin_writer.write_all(user_input.as_bytes()).await.unwrap();
    //     }
    // });

    // // let mut stdin = termion::async_stdin().keys();
    // // let rt = Runtime::new().unwrap();
    // // rt.spawn(async move {
    // //     loop {
    // //         let user_input = stdin.next();
    // //         if user_input.is_some() {
    // //             println!("Hi");
    // //             let user_input_b = user_input.unwrap().unwrap();
    // //             println!("{:?}", user_input_b);
    // //             stdin_writer.write_all(&vec![user_input_b, String::from("\n").as_bytes()[0]]).await.unwrap();
    // //         }
    // //     }
    // // });

    // let mut next_stdout;
    // let mut stdout_str;

    // loop {
    //     // Read command output
    //     next_stdout = stdout_stream.next().await;
    //     stdout_str = String::from_utf8(next_stdout.unwrap()?.to_vec()).unwrap();
    //     print!("{}", stdout_str);
    //     std::io::stdout().flush().unwrap();
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
