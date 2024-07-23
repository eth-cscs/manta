# Basic operations

Manta commands have a nomal and a short version

## List configurations

A configuration is a list of layers, each being a git repository containing ansible playbooks, these playbooks will run in order to create an image or to configure nodes during runtime.

List/filter the configurations in the system

Normal version

```bash
manta get configurations ...
```

Short version

```bash
manta g c ...
```

eg:

Get all configurations for cluster 'zinal' which name contains `cta_test`

```bash
manta g c -H zinal -p '*cta_test*'
+--------------------------------------------------------------+----------------------+----------------------------------------------------+
| Config Name                                                  | Last updated         | Layers                                             |
+==========================================================================================================================================+
| runtime-zinal_cta_test_hook-mc-cscs-24.3.0.r1-20240602153320 | 2024-06-02T15:33:31Z | Name:     test_layer                               |
|                                                              |                      | Playbook: site.yml                                 |
|                                                              |                      | Commit:   abd379d2aeb1d920da33392d610d3012a6ef06a2 |
|--------------------------------------------------------------+----------------------+----------------------------------------------------|
| runtime-zinal_cta_test_hook-mc-cscs-24.3.0.r1-20240608192847 | 2024-06-08T19:29:08Z | Name:     test_layer                               |
|                                                              |                      | Playbook: site.yml                                 |
|                                                              |                      | Commit:   abd379d2aeb1d920da33392d610d3012a6ef06a2 |
|--------------------------------------------------------------+----------------------+----------------------------------------------------|
| runtime-zinal_cta_test_hook-mc-cscs-24.3.0.r1-20240608201045 | 2024-06-08T20:29:28Z | Name:     test_layer                               |
|                                                              |                      | Playbook: site.yml                                 |
|                                                              |                      | Commit:   abd379d2aeb1d920da33392d610d3012a6ef06a2 |
|--------------------------------------------------------------+----------------------+----------------------------------------------------|
| runtime-zinal_cta_test_hook-mc-cscs-24.3.0.r1-20240608203940 | 2024-06-08T20:39:52Z | Name:     test_layer                               |
|                                                              |                      | Playbook: site.yml                                 |
|                                                              |                      | Commit:   abd379d2aeb1d920da33392d610d3012a6ef06a2 |
+--------------------------------------------------------------+----------------------+----------------------------------------------------+
```

> Note: the argument `-p` matches a [blob pattern](https://mywiki.wooledge.org/glob)

Get last 2 most recent configurations created or updated

```bash
manta g c -H zinal -l 2
+--------------------------------------------------------------+----------------------+----------------------------------------------------+
| Config Name                                                  | Last updated         | Layers                                             |
+==========================================================================================================================================+
| runtime-zinal_cta_test_hook-mc-cscs-24.3.0.r1-20240608201045 | 2024-06-08T20:29:28Z | Name:     test_layer                               |
|                                                              |                      | Playbook: site.yml                                 |
|                                                              |                      | Commit:   abd379d2aeb1d920da33392d610d3012a6ef06a2 |
|--------------------------------------------------------------+----------------------+----------------------------------------------------|
| runtime-zinal_cta_test_hook-mc-cscs-24.3.0.r1-20240608203940 | 2024-06-08T20:39:52Z | Name:     test_layer                               |
|                                                              |                      | Playbook: site.yml                                 |
|                                                              |                      | Commit:   abd379d2aeb1d920da33392d610d3012a6ef06a2 |
+--------------------------------------------------------------+----------------------+----------------------------------------------------+
```

If manta realise only one configuration is returned, then it will fetch extra information for each configuration layers, the example below will fetch the details of all configuration layers related to the most recent configuration created across all clusters I have access.

```bash
manta g c -m
+--------------------------------------------+----------------------+----------------------------------------------------+--------------------------------------------------------+
| Configuration Name                         | Last updated         | Layers                                             | Derivatives                                            |
+=================================================================================================================================================================================+
| eiger-mc-compute-config-cscs-24.3.0.r1-0.1 | 2024-06-14T17:29:38Z | Name:     csm-packages-1.5.0                       | CFS sessions:                                          |
|                                            |                      | Branch:   cray/csm/1.16.26                         |  - batcher-3e4e38eb-db3c-45b3-bc00-992ffe4a7792        |
|                                            |                      | Tag:                                               |  - batcher-b5329f53-8121-49ab-b7dd-7de7f17bf4bb        |
|                                            |                      | Date:     2024-03-22T10:54:39Z                     |                                                        |
|                                            |                      | Author:   crayvcs - cf-gitea-import                | BOS sessiontemplates:                                  |
|                                            |                      | Commit:   6423c550ea38a3b6befc4867ea7319157b48c554 |  - eiger-mc-compute-template-cscs-24.3.0.r1-0.1.x86_64 |
|                                            |                      | Playbook: csm_packages.yml                         |                                                        |
|                                            |                      |                                                    | IMS images:                                            |
|                                            |                      | Name:     shs-cassini_install-cscs-24.3.0          |  - eiger-mc-compute-cscs-24.3.0.r1-0.1                 |
|                                            |                      | Branch:   cscs-24.3.0cscs-24.5.0                   |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-05-03T10:24:28+02:00                |                                                        |
|                                            |                      | Author:   Riccardo Di Maria                        |                                                        |
|                                            |                      | Commit:   4dead2f7ebd1080d6a65d4b374618cf727624215 |                                                        |
|                                            |                      | Playbook: shs_cassini_install.yml                  |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     cos-compute-cscs-24.3.0                  |                                                        |
|                                            |                      | Branch:   cscs-24.3.0                              |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-06-14T15:44:11+02:00                |                                                        |
|                                            |                      | Author:   Gennaro Oliva                            |                                                        |
|                                            |                      | Commit:   f8247821fa3c220159f31694cc995f24e07bb8f1 |                                                        |
|                                            |                      | Playbook: cos-compute.yml                          |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     csm-diags-compute-1.5.26                 |                                                        |
|                                            |                      | Branch:   cray/csm-diags/1.5.26                    |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-03-22T17:11:47Z                     |                                                        |
|                                            |                      | Author:   crayvcs - cf-gitea-import                |                                                        |
|                                            |                      | Commit:   3b59bd64682b8e55c7f49c6317442b48cea6bb53 |                                                        |
|                                            |                      | Playbook: csm-diags-compute.yml                    |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     sma-ldms-compute-1.9.14                  |                                                        |
|                                            |                      | Branch:   cray/sma/1.9.14                          |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-03-22T16:24:29Z                     |                                                        |
|                                            |                      | Author:   crayvcs - cf-gitea-import                |                                                        |
|                                            |                      | Commit:   09b922e6a608f273dd28dfbc9eaf089b355a56b8 |                                                        |
|                                            |                      | Playbook: sma-ldms-compute.yml                     |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     cscs                                     |                                                        |
|                                            |                      | Branch:   cscs-24.3.0                              |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-06-14T19:26:31+02:00                |                                                        |
|                                            |                      | Author:   Gennaro Oliva                            |                                                        |
|                                            |                      | Commit:   a58fc02eb298cd2b472c616d2ef641ee2c456181 |                                                        |
|                                            |                      | Playbook: site.yml                                 |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     cpe-pe_deploy-cscs-24.3.0                |                                                        |
|                                            |                      | Branch:   cscs-24.5.0                              |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-06-14T13:51:32+02:00                |                                                        |
|                                            |                      | Author:   Gennaro Oliva                            |                                                        |
|                                            |                      | Commit:   f9105c8172af7521910e849d274c91103bfbde75 |                                                        |
|                                            |                      | Playbook: site-cscs.yml                            |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     slurm-site-cscs-24.3.0                   |                                                        |
|                                            |                      | Branch:   cscs-24.3.0                              |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-06-05T15:58:19+02:00                |                                                        |
|                                            |                      | Author:   Gennaro Oliva                            |                                                        |
|                                            |                      | Commit:   e75324a1a35f20fda131fded61a486025fce6d41 |                                                        |
|                                            |                      | Playbook: site-cscs.yml                            |                                                        |
|                                            |                      |                                                    |                                                        |
|                                            |                      | Name:     cos-compute-last-cscs-24.3.0             |                                                        |
|                                            |                      | Branch:                                            |                                                        |
|                                            |                      | Tag:                                               |                                                        |
|                                            |                      | Date:     2024-06-04T18:31:44+02:00                |                                                        |
|                                            |                      | Author:   Chris Gamboni                            |                                                        |
|                                            |                      | Commit:   6bf3f82608686da8dbf857656f1e37c4c84054d1 |                                                        |
|                                            |                      | Playbook: cos-compute-last.yml                     |                                                        |
+--------------------------------------------+----------------------+----------------------------------------------------+--------------------------------------------------------+
```

> Note: manta will try to resolve the `git tag` related to each `configuration layer`

## List sessions

A session is a job against a configuration to create an image or configure a compute node during runtime.

List/filter the sessions in the system

Normal version

```bash
manta get sessions ...
```

Short version

```bash
manta g s ...
```

eg:

Get all configurations for cluster 'eiger'

```bash
manta get sessions -H eiger
+----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+---------------+----------+
| Session Name                                 | Configuration Name                         | Start               | Status   | Succeeded | Target Def | Target        | Image ID |
+================================================================================================================================================================================+
| batcher-3e4e38eb-db3c-45b3-bc00-992ffe4a7792 | eiger-mc-compute-config-cscs-24.3.0.r1-0.1 | 2024-06-15T14:28:58 | complete | true      | dynamic    | x1000c0s0b1n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s0b0n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s0b0n1 |          |
|----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+---------------+----------|
| batcher-b5329f53-8121-49ab-b7dd-7de7f17bf4bb | eiger-mc-compute-config-cscs-24.3.0.r1-0.1 | 2024-06-15T14:29:00 | complete | true      | dynamic    | x1000c0s3b1n1 |          |
|                                              |                                            |                     |          |           |            | x1000c0s3b0n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s3b1n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b0n1 |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b1n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s0b1n1 |          |
|                                              |                                            |                     |          |           |            | x1002c0s7b0n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s2b0n1 |          |
|                                              |                                            |                     |          |           |            | x1000c0s2b0n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b1n1 |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b0n0 |          |
|                                              |                                            |                     |          |           |            | x1000c0s2b1n1 |          |
|                                              |                                            |                     |          |           |            | x1000c0s3b0n1 |          |
+----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+---------------+----------+
```
eg

Get last 3 sessions created among all the clusters the user has access

```bash
manta get sessions -l 3
+----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+----------------+----------+
| Session Name                                 | Configuration Name                         | Start               | Status   | Succeeded | Target Def | Target         | Image ID |
+=================================================================================================================================================================================+
| batcher-3f4da728-2160-45d2-9109-26313eb1ae1b | adula-uan-config-24.5.0-2                  | 2024-06-15T14:29:00 | complete | false     | dynamic    | x3100c0s28b0n0 |          |
|----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+----------------+----------|
| batcher-b5329f53-8121-49ab-b7dd-7de7f17bf4bb | eiger-mc-compute-config-cscs-24.3.0.r1-0.1 | 2024-06-15T14:29:00 | complete | true      | dynamic    | x1000c0s3b1n1  |          |
|                                              |                                            |                     |          |           |            | x1000c0s3b0n0  |          |
|                                              |                                            |                     |          |           |            | x1000c0s3b1n0  |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b0n1  |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b1n0  |          |
|                                              |                                            |                     |          |           |            | x1000c0s0b1n1  |          |
|                                              |                                            |                     |          |           |            | x1002c0s7b0n0  |          |
|                                              |                                            |                     |          |           |            | x1000c0s2b0n1  |          |
|                                              |                                            |                     |          |           |            | x1000c0s2b0n0  |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b1n1  |          |
|                                              |                                            |                     |          |           |            | x1000c0s1b0n0  |          |
|                                              |                                            |                     |          |           |            | x1000c0s2b1n1  |          |
|                                              |                                            |                     |          |           |            | x1000c0s3b0n1  |          |
|----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+----------------+----------|
| batcher-9dfea67f-6638-4acf-8429-4d242e264bdb | adula-uan-config-24.5.0-2                  | 2024-06-15T14:46:51 | complete | false     | dynamic    | x3100c0s28b0n0 |          |
+----------------------------------------------+--------------------------------------------+---------------------+----------+-----------+------------+----------------+----------+
```

## List images

Images are created through sessions against configurations. Images can be assigned to nodes to boot. Manta tracks down the configuration used to build an image through the sessions used to build it, withtout this information the image content is not traceable and the nodes using it are subject to errors or misconfiguration.

List/filter the images in the system

Normal version

```bash
manta get images ...
```

Short version

```bash
manta g i ...
```

eg

List images all images created for cluster eiger

```bash
manta g i -H eiger
+--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------+
| Image ID                             | Name                                | Creation time                    | CFS configuration | HSM groups |
+================================================================================================================================================+
| 39c7603b-805b-4aeb-9c4b-27f960400489 | eiger-uan-3.1.0                     | 2023-03-31T08:52:39.210677+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 44d34cf9-01d1-4600-bbf3-a69b303bf464 | eiger-cos-3.1.0                     | 2023-03-31T08:53:43.665937+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 5024e45e-d1f5-4cca-bf7b-0b0b4e7f0785 | eiger-uan-3.1.1                     | 2023-06-13T11:03:21.448332+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 5e7c2e99-7386-4db5-b39b-6e5f107936bd | eiger-cos-3.1.1                     | 2023-06-13T11:03:51.481619+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| cbaf89c5-d5be-4251-b154-c81b05bd5e59 | eiger-uan-3.1.2                     | 2023-10-19T19:09:42.689921+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 09b986aa-2ee1-44a2-8358-71d11a772f5f | eiger-cos-3.1.2                     | 2023-10-19T19:10:04.608446+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 83f0224d-e1db-4925-8de6-55cdf0b3bbf1 | eiger-uan-3.1.4                     | 2023-11-22T14:51:39.867557+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| a279cfc0-88db-48db-af18-b044326d4d9e | eiger-cos-3.1.4                     | 2023-11-22T14:52:41.057211+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 866dc945-f7e7-475f-9ce2-b8480bc070d6 | eiger-uan-3.1.6                     | 2024-02-28T08:55:05.596347+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 3cd22417-0a13-4e55-afd1-645ebfd00805 | eiger-cos-3.1.6                     | 2024-02-28T08:55:19.685122+00:00 | Not found         |            |
|--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------|
| 9503d311-91a1-40bc-a484-0ea18a9167e7 | eiger-mc-compute-cscs-24.3.0.r1-0.1 | 2024-06-05T15:10:54.924288+00:00 | Not found         |            |
+--------------------------------------+-------------------------------------+----------------------------------+-------------------+------------+
```

## List cluster details/summary

## List hardware information

List the hardware information in the system

Normal version

```bash
manta get hardware cluster ...
manta get hardware node ...
```

Short version

```bash
manta g hw c ...
manta g hw n ...
```
eg

Get hardware components summary for cluster eiger

```bash
manta g hw cluster eiger
Getting hw components for node 'x1002c0s7b0n0' [16/17]
+------------------------------------+----------+
| HW Component                       | Quantity |
+===============================================+
| Memory (GiB)                       | 4608     |
|------------------------------------+----------|
| SS11 200Gb 2P NIC Mezz REV02 (HSN) | 17       |
|------------------------------------+----------|
| AMD EPYC 7742 64-Core Processor    | 34       |
+------------------------------------+----------+
```

eg

Get hardware component details for cluster eiger

```bash
manta g hw cluster eiger --output details
Getting hw components for node 'x1002c0s7b0n0' [16/17]
+---------------+-----------+-----------+---------------------------------+------------------------------------+
| Node          | 16384 MiB | 32768 MiB | AMD EPYC 7742 64-Core Processor | SS11 200Gb 2P NIC Mezz REV02 (HSN) |
+==============================================================================================================+
| x1000c0s0b0n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s0b0n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s0b1n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s0b1n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s1b0n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s1b0n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s1b1n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s1b1n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s2b0n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s2b0n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s2b1n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s2b1n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s3b0n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s3b0n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s3b1n0 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1000c0s3b1n1 |  ✅ (16)  |     ❌    |              ✅ (2)             |               ✅ (1)               |
|---------------+-----------+-----------+---------------------------------+------------------------------------|
| x1002c0s7b0n0 |     ❌    |  ✅ (16)  |              ✅ (2)             |               ✅ (1)               |
+---------------+-----------+-----------+---------------------------------+------------------------------------+
```

## Get session logs

It is possible to inspect the ansible output when a session is running.

Normal version

```bash
manta get log ...
```

Short version

```bash
manta l ...
```

eg

Get the logs of a session

```shell
manta log --session-name batcher-cef892ee-39af-444a-b32c-89478a100e4d
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

## Connect to node console

Sometimes, the node is unrecheable or can't be accessed, for these cases, the console may help to diagnose hardware issues.

eg

```
manta console x1500c2s4b0n1
Connected to x1500c2s4b0n1!
Use &. key combination to exit the console.

<ConMan> Connection to console [x1500c2s4b0n1] opened.

<ConMan> Console [x1500c2s4b0n1] joined with <nobody@localhost> on pts/511 at 10-30 02:14.

nid003129 login:
```

## Power management

### Power on cluster

Normal version

```bash
manta power on cluster <cluster name>
```

Short version

```bash
manta p on c <cluster name>
```

eg:

Power on cluster `zinal`

```bash
manta p on c zinal
```

### Power on nodes

Normal version

```bash
manta power on nodes <list of nodes>
```

Short version

```bash
manta p on n <list of nodes>
```

eg:

Power on a list of nodes

```bash
manta p on n x1001c1s5b0n0,x1001c1s5b0n1
```

### Power off cluster

Normal version

```bash
manta power off cluster <cluster name>
```

Short version

```bash
manta p off c <cluster name>
```

eg:

Power off cluster `zinal`

```bash
manta p off c zinal
```

### Power off nodes

Normal version

```bash
manta power off nodes <list of nodes>
```

Short version

```bash
manta p off n <list of nodes>
```

eg:

Power off a list of nodes

```bash
manta p off n x1001c1s5b0n0,x1001c1s5b0n1
```

### Power reset cluster

Normal version

```bash
manta power reset cluster <cluster name>
```

Short version

```bash
manta p reset c <cluster name>
```

eg:

Power reset cluster `zinal`

```bash
manta p reset c zinal
```

### Power reset nodes

Normal version

```bash
manta power reset nodes <list of nodes>
```

Short version

```bash
manta p reset n <list of nodes>
```

eg:

Power reset a list of nodes

```bash
manta p reset n x1001c1s5b0n0,x1001c1s5b0n1
```

## Set runtime configuration

Normal version

```bash
manta set runtime-configuration ...
```

Short version

```bash
manta s rc ...
```

eg:

Set/update runtime configuration for all nodes in a cluster

```bash
manta set runtime-configuration --hsm-group zinal --configuration my_configuration
```

Set/update runtime configuration for a list of nodes

```bash
manta set runtime-configuration --xnames x1001c1s5b0n0,x1001c1s5b0n1 --configuration my_configuration
```

## Set boot image

???+ warning "**Highly discourage**"
      Use this command only if `manta set boot-configuration` fails because it can't find the sessions linking the configuration with the image (meaning information in the system has been deleted)

Normal version

```bash
manta set boot-image ...
```

Short version

```bash
manta s bi ...
```

eg:

Set/update boot image for all nodes in a cluster

```bash
manta set boot-image --hsm-group zinal --image-id e2ce82f0-e7ba-4f36-9f5c-750346599600
```

Set/update runtime configuration for a list of nodes

```bash
manta set boot-image --xnames x1001c1s5b0n0,x1001c1s5b0n1 --image-id e2ce82f0-e7ba-4f36-9f5c-750346599600
```

## Set boot configuration

Normal version

```bash
manta set boot-configuration ...
```

Short version

```bash
manta s bc ...
```

eg:

Set/update boot configuration for all nodes in a cluster

```bash
manta set boot-configuration --hsm-group zinal --configuration my-configuration
```

Set/update boot configuration for a list of nodes

```bash
manta set boot-configuration --xnames x1001c1s5b0n0,x1001c1s5b0n1 --configuration my-configuration
```

## Set kernel parameters

???+ info "**Do not use rootfs**"
      The `rootfs` kernel parameter is managed automatically when changing the boot image, do not add `rootfs` kernel param in this command

Normal version

```bash
manta set kernel-parameters ...
```

Short version

```bash
manta s kp ...
```

eg:

Set/update boot configuration for all nodes in a cluster

```bash
manta set kernel-parameters --hsm-group zinal --kernel-parameters "console=ttyS0,115200 bad_page=panic crashkernel=512M hugepagelist=2m-2g intel_pstate=disable iommu.passthrough=on numa_balancing=disable numa_interleave_omit=headless oops=panic pageblock_order=14 pcie_ports=native rd.retry=10 rd.shell split_lock_detect=off systemd.unified_cgroup_hierarchy=1 ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}"
```

Set/update boot configuration for a list of nodes

```bash
manta set kernel-parameters --xnames x1001c1s5b0n0,x1001c1s5b0n1 --kernel-parameters "console=ttyS0,115200 bad_page=panic crashkernel=512M hugepagelist=2m-2g intel_pstate=disable iommu.passthrough=on numa_balancing=disable numa_interleave_omit=headless oops=panic pageblock_order=14 pcie_ports=native rd.retry=10 rd.shell split_lock_detect=off systemd.unified_cgroup_hierarchy=1 ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}
```
