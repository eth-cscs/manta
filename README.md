# MANTA

Another CLI tool for [Shasta](https://apidocs.giuv.cscs.ch/)

It basically translates your manta command into http calls against Shasta related APIs (Manta does not use other tools like cray cli or kubectl)

**NOTE:** current implementation does not creates/updates/deletes any information in the Shasta mgmt plane

## Features

- List and filter CFS configurations based on cluster name or configuration name
- List and filter CFS sessions based on cluster name or session name
- CFS session layer log watcher
  
## Prerequisites

Install Rust toolchain [ref](https://www.rust-lang.org/tools/install)

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Start a socks5 proxy (this is needed so manta http client can reach required apis like cvs, shasta mgmt and k8s api server).

```shell
ssh -D 1080 $USER@mgmt-alps -J bastion.cscs.ch
```

Get shasta api token api (check with your shasta administrator)

Get a k8s config file to connect to shasta k8s api (check with your shasta administrator)

Run

```shell
RUST_LOG=info KUBECONFIG=<path to k8s config file with connection details to shasta k8s api server> SHASTA_ADMIN_PWD=<shasta api admin token> GITEA_TOKEN=<shasta gitea auth token> cargo run -- --help
```

or

```shell
cargo bild
RUST_LOG=info KUBECONFIG=<Shasta k8s config> SHASTA_ADMIN_PWD=<shasta api admin token> GITEA_TOKEN=<shasta gitea auth token>  target/debug/manta --help
```

|env var|Description|
|-------|-----------|
|RUST_LOG|log details/verbosity|
|KUBECONFIG|path to file with kubernetes configuration to reach Shasta k8s api server. Manta will use this file to talk to k8s api server in same fashion as kubectl does|
|SHASTA_ADMIN_PWD|admin-client-auth secret in Shasta k8s|
|GITEA_TOKEN|user authentitacion token for gitea|

## Deployment

cargo build --release
