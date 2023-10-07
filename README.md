# MANTA

Another CLI tool for [Alps](https://www.cscs.ch/science/computer-science-hpc/2021/cscs-hewlett-packard-enterprise-and-nvidia-announce-worlds-most-powerful-ai-capable-supercomputer).

Manta is a frontend cli to interact with Shasta, it uses [mesa](https://crates.io/crates/mesa) for all Shasta interaction.

Manta's goals:

 - release operators from repetitive tasks.
 - provide quick system feedback.

Manta aggregates information from multiple sources:

 - Shasta Keycloak
 - Shasta API
 - Shasta K8s API
 - local git repo
 - Gitea API (Shasta VCS)
 - Hashicorp Vault

## Features

- List and filter CFS configurations based on cluster name or configuration name
- List and filter CFS sessions based on cluster name or session name
- List and filter BOS session templates based on cluster name or session name
- List nodes in HSM groups
- Create CFS configuration and session (target dynamic) from local repository
- Create CFS configuration and session (target image) from CSCS SAT input file
- Watch logs of a CFS session
- Connect to a node's console
- Power On/Off or restart nodes individually, in a list or per cluster
- Restrict operations to nodes belonging to a specific HSM group
- Filter information to a HSM group
- Update node boot image based on CFS configuration name
- Audit/Log
- Delete all data related to CFS configuration

## Configuration

Manta needs a configuration file in `$HOME/.config/manta/config.toml` like shown below

```bash
log = "error"
socks5_proxy = "socks5h://127.0.0.1:1080"
shasta_base_url = "https://api.cmn.alps.cscs.ch/apis"
keycloak_base_url = "https://api.cmn.alps.cscs.ch/keycloak"
gitea_base_url = "https://api.cmn.alps.cscs.ch/vcs"
k8s_api_url = "https://10.252.1.12:6442"
vault_base_url = "https://hashicorp-vault.cscs.ch:8200"
vault_role_id = "b15517de-cabb-06ba-af98-633d216c6d99" # vault in hashicorp-vault.cscs.ch
vault_secret_path = "shasta"
hsm_group = "psitds"
```

Manta logs user's operations in /var/log/manta/ folder, please make sure this folder exists and all users have rwx access to it

```bash
mkdir /var/log/manta
chmod 777 -R /var/log/manta
```

### Legend:

|Name|mandatory|Type|Description|Example|
|----|---------|----|-----------|-------|
|MANTA_CSM_TOKEN|no|env|CSM authentication token, if this env var is missing, then manta will prompt use for credentials against CSM keycloak||
|log|no|config file|log details/verbosity|off/error/warn/info/debug/trace|
|socks5_proxy|yes|config file|socks proxy to access the services (only needed if using manta from outside a Shasta management node. Need VPN. Need to ope your VPN IP in hashicorp  vault approle)|socks5h://127.0.0.1:1080|RE
|keycloak_base_url|yes|config file|Keycloak base URL for authentication|https://api.cmn.alps.cscs.ch/keycloak|
|gitea_base_url|yes|config file|Gitea base URL to fetch CFS layers git repo details|https://api.cmn.alps.cscs.ch/vcs|
|k8s_api_url|yes|config file|Shasta k8s API URL|https://10.252.1.12:6442|
|vault_base_url|yes|config file|Hashicorp Vault base URL storing secrets to authenticate to external services|https://hashicorp-vault.cscs.ch|
vault_role_id|yes|config file|role id related to Hashicorp Vault base URL approle authentication|yes|b15517de-cabb-06ba-af98-633d216c6d99|
|vault_secret_path|config file|path in vault to find secrets|shasta or prealps|
|shasta_base_url|yes|config file|Shasta API base URL for Shasta related jobs submission|https://api-gw-service-nmn.local/apis|
|hsm_group|no|config|If exists, then it will filter/restrict the hsm groups and/or xnames targeted by the cli command|psi-dev|
|hsm_available|no|config|List of HSM groups avaiable to use|psi-dev,psitds|

## Example

### Get latest (most recent) session

```shell
$ manta get session --most-recent
+----------------------------------------------+-------------------------+---------+---------------+---------------+---------------------+----------+-----------+------------------------------------------+
| Name                                         | Configuration           | Target  | Target groups | Ansible limit | Start               | Status   | Succeeded | Job                                      |
+==========================================================================================================================================================================================================+
| batcher-bab0cd68-5c61-4774-a685-bd57f744f62d | eiger-cos-config-3.0.24 | dynamic |               | x1002c6s6b0n0 | 2022-10-29T15:50:19 | complete | true      | cfs-cd39e25e-5b66-4ee9-be1c-027f5cd00683 |
+----------------------------------------------+-------------------------+---------+---------------+---------------+---------------------+----------+-----------+------------------------------------------+
```

### Get logs for a session/layer

```shell
$ manta log --session-name batcher-cef892ee-39af-444a-b32c-89478a100e4d --layer-id 0
[2022-09-27T12:41:49Z INFO  manta::shasta_cfs_session_logs::client] Pod name: "cfs-b49cdc2b-d6cb-4477-b502-6be479472546-2jrlg"
Waiting for Inventory
Waiting for Inventory
Waiting for Inventory
Waiting for Inventory
Waiting for Inventory
Waiting for Inventory
Waiting for Inventory
Inventory generation completed
SSH keys migrated to /root/.ssh
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
  0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0
HTTP/1.1 200 OK
content-type: text/html; charset=UTF-8
cache-control: no-cache, max-age=0
x-content-type-options: nosniff
date: Tue, 27 Sep 2022 12:18:16 GMT
server: envoy
transfer-encoding: chunked

Sidecar available
[WARNING]: Invalid characters were found in group names but not replaced, use
-vvvv to see details

PLAY [Compute] *****************************************************************

PLAY [Application] *************************************************************
skipping: no hosts matched

PLAY [Management_Worker] *******************************************************
skipping: no hosts matched

PLAY RECAP *********************************************************************
x1500c7s2b0n0              : ok=1    changed=0    unreachable=0    failed=0    skipped=33   rescued=0    ignored=0   
```

### Create a CFS session and watch logs

```
$ manta apply session --repo-path /home/msopena/ownCloud/Documents/ALPSINFRA/vcluster_shasta_scripts/muttler/muttler_orchestrator/ --watch-logs --ansible-limit x1500c3s4b0n1
[2022-10-08T22:56:31Z INFO  manta::create_session_from_repo] Checking repo /home/msopena/ownCloud/Documents/ALPSINFRA/vcluster_shasta_scripts/muttler/muttler_orchestrator/.git/ status
[2022-10-08T22:56:32Z INFO  manta::create_session_from_repo] CFS configuration name: m-muttler-orchestrator
[2022-10-08T22:56:35Z INFO  manta::create_session_from_repo] CFS session name: m-muttler-orchestrator-20221008225632
[2022-10-08T22:56:35Z INFO  manta] cfs session: m-muttler-orchestrator-20221008225632
[2022-10-08T22:56:35Z INFO  manta] Fetching logs ...
[2022-10-08T22:56:35Z INFO  manta::shasta_cfs_session_logs::client] Pod for cfs session m-muttler-orchestrator-20221008225632 not ready. Trying again in 2 secs. Attempt 1 of 10
[2022-10-08T22:56:38Z INFO  manta::shasta_cfs_session_logs::client] Pod name: cfs-f1588924-f791-4bb8-a565-f61563a4274b-n7bbn
[2022-10-08T22:56:38Z INFO  manta::shasta_cfs_session_logs::client] Container ansible-0 not ready. Trying again in 2 secs. Attempt 1 of 10
[2022-10-08T22:56:40Z INFO  manta::shasta_cfs_session_logs::client] Container ansible-0 not ready. Trying again in 2 secs. Attempt 2 of 10
[2022-10-08T22:56:42Z INFO  manta::shasta_cfs_session_logs::client] Container ansible-0 not ready. Trying again in 2 secs. Attempt 3 of 10
Waiting for Inventory
Waiting for Inventory
Inventory generation completed
SSH keys migrated to /root/.ssh
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
HTTP/1.1 200 OK
content-type: text/html; charset=UTF-8
cache-control: no-cache, max-age=0
x-content-type-options: nosniff
date: Sat, 08 Oct 2022 22:56:49 GMT
server: envoy
transfer-encoding: chunked

  0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0
Sidecar available
[WARNING]: Invalid characters were found in group names but not replaced, use
-vvvv to see details

PLAY [Compute:Application] *****************************************************

PLAY RECAP *********************************************************************
x1500c3s4b0n1              : ok=8    changed=0    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0
```

### Create an interactive session to a node

```
$ manta console x1500c2s4b0n1
[2022-10-30T02:14:44Z INFO  manta::node_console] Alternatively run - kubectl -n services exec -it cray-console-node-2 -c cray-console-node -- conman -j x1500c2s4b0n1 
[2022-10-30T02:14:44Z INFO  manta::node_console] Connecting to console x1500c2s4b0n1
Connected to x1500c2s4b0n1!
Use &. key combination to exit the console.

<ConMan> Connection to console [x1500c2s4b0n1] opened.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/452 at 10-30 02:14.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/453 at 10-30 02:14.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/454 at 10-30 02:14.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/455 at 10-30 02:14.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/468 at 10-30 02:14.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/510 at 10-30 02:14.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/511 at 10-30 02:14.

nid003129 login: 
```

### Power off a node

```
$ manta apply node off --force "x1004c1s4b0n1"
```

### Power on a node

```
$ manta apply node on "x1004c1s4b0n1"
```

## Deployment

### Build container image

This repo contains a Dockerfile to build a Container with manta cli.

```
docker build -t manta .
```

#### Run

```
$ docker run -it --network=host -v ~:/root/ manta --help
```

### Build from sources

Install Rust toolchain https://www.rust-lang.org/tools/install

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install cross to be able to complile on different platforms

```shell
cargo install cross
```

Generate binary (cross compilation)

```shell
cross build --target x86_64-unknown-linux-gnu --release
```

or

```shell
rustup target add x86_64-unknown-linux-gnu
cargo build --target=x86_64-unknown-linux-gnu
```

### Development

#### Prerequisites

Install 'cargo dist' and 'cargo release'

```
cargo install cargo-dist
cargo install cargo-release
```

Configure cargo-dist. Accept default options and only target linux assets

```
cargo dist init -t x86_64-unknown-linux-gnu
```

Then remove the assets for macos and windows

Make sure a github workflow is created in `.github/workflows/release.yml`

#### Deployment

```
cargo dist patch --execute
```

If everything went well, then binary should be located in `manta/target/x86_64-unknown-linux-gnu/release/manta`

### Profiling

#### Enable capabilities

```bash
sudo sysctl -w kernel.perf_event_paranoid=-1
```

#### Install perf

```bash
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`
```

#### Grant access to kernel address map

```bash
sudo sh -c " echo 0 > /proc/sys/kernel/kptr_restrict"
```

#### Create perf data

```bash
perf stat -ad -r 100 target/release/manta get session
```

#### Identify bottlenecks and get hotspots for those events


```bash
perf record -g --call-graph=dwarf -F max target/release/manta get session
```

#### Convert perf data file to a format firefox profiles understands

```bash
perf script -F +pid > manta.perf
```

Go to https://profiler.firefox.com/ and open manta.perf file


#### DHAT mem alloction profiling

> https://docs.rs/dhat/latest/dhat/
> NOTE: lto in Cargo.toml needs to be disabled 

##### Run 

```bash
cargo run -r --features dhat-heap -- get session
```

##### View results (dhat-heap.json file)

https://nnethercote.github.io/dh_view/dh_view.html
