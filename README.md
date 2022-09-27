# MANTA

Another CLI tool for Shasta

## Prerequisites

Install Rust toolchain [ref](https://www.rust-lang.org/tools/install)

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Start a socks5 proxy layer to access shasta mgmt and shasta k8s api nodes. This is needed so manta http client can reach shasta cvs, mgmt and k8s rest apis.

```shell
ssh -D 1080 msopena@mgmt-alps -J bastion.cscs.ch
```

Get shasta api token api (check with your shasta administrator)

Get a k8s config file to connect to shasta k8s api (check with your shasta administrator)

Run

```shell
RUST_LOG=info KUBECONFIG=<k8s config file to connect to shasta> SHASTA_ADMIN_PWD=<shasta api admin token> GITEA_TOKEN=<shasta gitea auth token> cargo run -- --help
```

## Deployment

cargo build --release