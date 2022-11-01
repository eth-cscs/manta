# MANTA

Another CLI tool for [Alps](https://confluence.cscs.ch/display/ALPSINFRA/Alps+Home).

Manta is an aggregator from multiple sources:
 - Shasta API
 - K8s API, 
 - local git repo, 
 - Gitlab API, 
 - Gitea API and Hashicorp Vault). 

Manta's goals:
 - release operators from repetitive tasks.
 - provide quick system feedback.

## Features

- List and filter CFS configurations based on cluster name or configuration name
- List and filter CFS sessions based on cluster name or session name
- List and filter BOS session templates based on cluster name or session name
- Create CFS session out of a repository
- Watch logs of a CFS session
- Open an interactive session to a node console using remote's terminal and shell
- Power On/Off nodes

## Configuration

|Name|Type|Description|Example|
|----|----|-----------|-------|
|RUST_LOG|env|log details/verbosity|info|
|socks5_proxy|config file|socks proxy to access the services|socks5h://127.0.0.1:1080|
|shasta_base_url|config file|Shasta base URL|https://api-gw-service-nmn.local/apis|

## Example

##### Get lastest (most recent) session

```shell
$ manta get session --most-recent
+----------------------------------------------+-------------------------+---------+---------------+---------------+---------------------+----------+-----------+------------------------------------------+
| Name                                         | Configuration           | Target  | Target groups | Ansible limit | Start               | Status   | Succeeded | Job                                      |
+==========================================================================================================================================================================================================+
| batcher-bab0cd68-5c61-4774-a685-bd57f744f62d | eiger-cos-config-3.0.24 | dynamic |               | x1002c6s6b0n0 | 2022-10-29T15:50:19 | complete | true      | cfs-cd39e25e-5b66-4ee9-be1c-027f5cd00683 |
+----------------------------------------------+-------------------------+---------+---------------+---------------+---------------------+----------+-----------+------------------------------------------+
```

##### Get logs for a session/layer

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

##### Create a CFS session and watch logs

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

##### Create an interactive session to a node

```
$ manta console -x x1500c2s4b0n1
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

##### Power off a node

```
$ manta apply node off --xnames "x1004c1s4b0n1" --force
```

##### Power on a node

```
$ manta apply node on --xnames "x1004c1s4b0n1"
```

## Deployment

##### Build container image

This repo contains a Dockerfile to build a Container with manta cli.

##### Build container image 

```
docker build -t manta .
```

##### Run

```
$ docker run -it --network=host -v ~:/root/ --env RUST_LOG=info manta --help
Another CLI for basic/simple Shasta operations

Usage: manta <COMMAND>

Commands:
  get      Get information from Shasta system
  apply    Make changes to Shata clusters/nodes
  log      Print session logs
  console  WIP Access node console
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```