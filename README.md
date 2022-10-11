# MANTA

Another CLI tool for [Shasta](https://apidocs.giuv.cscs.ch/)

Manta is an aggregator from multiple sources (Shasta API, K8s API, local git repo, Gitlab API and Gitea API). Manta's goal is to simplify daily Shasta operations.

## Features

- List and filter CFS configurations based on cluster name or configuration name
- List and filter CFS sessions based on cluster name or session name
- Watch logs of a CFS session
- Create CFS session out of a repository

## Example

Get lastest (most recent) session

```shell
$ manta get session --most-recent
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] *** CFS SESSIONS ***
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] ================================
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] name: batcher-cef892ee-39af-444a-b32c-89478a100e4d
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] configuration: rigi-cos-config-3.0.2
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] target definition: dynamic
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] target groups name: 
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] ansible - limit: "x1500c7s2b0n0"
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] start time: 2022-09-27T12:17:38
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] status: complete
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] succeeded: true
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] job: cfs-b49cdc2b-d6cb-4477-b502-6be479472546
[2022-09-27T12:41:34Z INFO  manta::cfs_utils] ================================
```

Get logs for a session/layer

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

Create a CFS session and watch logs

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

Power off a node

```
$ manta apply node off --xnames "x1004c1s4b0n1" --force
```

Shutdown a node

```
$ manta apply node on --xnames "x1004c1s4b0n1"
```

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

## Run

```shell
RUST_LOG=info KUBECONFIG=<path to k8s config file with connection details to shasta k8s api server> SHASTA_ADMIN_PWD=<shasta api admin token> GITEA_TOKEN=<shasta gitea auth token> cargo run -- --help
```

or

```shell
cargo build
RUST_LOG=info KUBECONFIG=<Shasta k8s config> SHASTA_ADMIN_PWD=<shasta api admin token> GITEA_TOKEN=<shasta gitea auth token>  target/debug/manta --help
```

|env var|Description|
|-------|-----------|
|RUST_LOG|log details/verbosity|
|KUBECONFIG|path to file with kubernetes configuration to reach Shasta k8s api server. Manta will use this file to talk to k8s api server in same fashion as kubectl does|
|SHASTA_ADMIN_PWD|admin-client-auth secret in Shasta k8s|
|GITEA_TOKEN|user authentitacion token for gitea|

## Deployment

```shell
cargo build --release
```