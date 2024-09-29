# FAQ

!!! question "What should we put here?"
    - List most common questions when using manta (eg: like a howto guide)
    - Don't put errors here

**Q: Can I run manta from my laptop?**

**A:** Depends. Alps system management APIs are not accessible from the public network.

- External users need to run manta from a hardened environment within CSCS.

- CSCS staff can create a SOCKS5 proxy through bastion and run manta from outside CSCS.

---

**Q: `manta get ...` only shows information for 1 cluster, I am missing other clusters data**

**A:** make sure manta is not locked to a cluster, run `manta config show` and make sure there is no `Current HSM` value.

In this example:
```bash
$ manta config show
Sites: ["prealps", "alps", "alpsm"]
Current site: alps
HSM available: ["adula", "alps", "burst", "burst_gh", "burst_mc", "eiger", "fora", "nodes_free", "psidev", "psidev_cn", "psidev_uan", "psitds", "psitds_cn", "psitds_uan", "zinal", "zinal_cta", "zinal_moleson_tds", "zinal_tds", "zinal_zinal"]
Current HSM: adula
Parent HSM: nodes_free
```
manta is locked to cluster `adula`, therefore it will ignore other cluster's information, to remove this property run command `manta config unset hsm`

---

**Q: I have been given access to a new cluster, however I can't see it.**

**A:** The user authentication token keeps the number of clusters the user has access to. Any changes related to the user access needs a token refresh. To do this run the following commands:

Delete token for the site hosting the cluster the user needs access to

```bash
$ manta config unset auth
Please chose the site token to delete from the list below:
> prealps_auth
  alps_auth
```

Select the site which auth token needs to be deleted (eg prealps_auth)

Manta will ask the user to authenticate upon the next operation against `prealps`, if authentication is successful, then the user should receive a new token with the new list of cluster he has access to this site/system

---

**Q: My sessions are failing due to an error while running ansible playbook. How can I run my ansible outside the management plane?**

**A:** You can start an `ephemeral session`, ephemeral sessions tries to replicate the host targeting the ansible playbook started by the session. To start an ephemeral environment run the following command

```bash
$ manta apply ephemeral-environment --image-id 4bf91021-8d99-4adf-945f-46de2ff50a3d
1eff1535-c13c-4d21-ba3f-0c852abd713b.ims.cmn.alps.cscs.ch
```

> Note: the `<image id>` should be the same base image used by the session

The command will return a hostname (`1eff1535-c13c-4d21-ba3f-0c852abd713b.ims.cmn.alps.cscs.ch`) based on the image id provided, after this, wait a few minutes for the environment to be ready and then create an ansible inventory with this hostname. With this, you should be able to run your ansible scripts locally if you need to troubleshoot your playbook.

To test the ephemeral environment

```bash
$ ssh root@1eff1535-c13c-4d21-ba3f-0c852abd713b.ims.cmn.alps.cscs.ch
The authenticity of host '1eff1535-c13c-4d21-ba3f-0c852abd713b.ims.cmn.alps.cscs.ch (148.187.109.148)' can't be established.
ED25519 key fingerprint is SHA256:/np4nEZWYztGbG/6cDI6vTsLU/9uWEXZQ9Msd/DJGNY.
This key is not known by any other names
Are you sure you want to continue connecting (yes/no/[fingerprint])? yes
Warning: Permanently added '1eff1535-c13c-4d21-ba3f-0c852abd713b.ims.cmn.alps.cscs.ch' (ED25519) to the list of known hosts.
Have a lot of fun...
cray-ims-1eff1535-c13c-4d21-ba3f-0c852abd713b-customize-xxl7f:/root #
```

---

**Q: How can I add a new site to manta?**

**A:** Each site needs to be included in your [configuration file](configuration.md) under the `sites` section. Please get in contact with an Alps administrator to help you configuring a new site since manta needs to know the APIs to talk to.

---

**Q: How can I list the number of sites `site` I have available?**

**A:** To check all sites available, please run the command below.

```bash
manta config show
```

---

**Q: How can I tell manta to switch from one `site` to another?**

**A:** To tell manta to talk to `site name` please run the command below.

```bash
manta config set site <site name>
```

---

**Q: What is the easiest way to configure my cluster (build an image, assign image to nodes to boot nodes and configure nodes during runtime) with manta?**

**A:** The easiest way to deploy a cluster using manta is through a [SAT a template and SAT vars files](cluster_mgmt_with_sat_file.yml).

```bash
$ ls my_cluster_sat
template.yaml session_vars.yaml
```

To tell manta to use those files run the following command:

```bash
$ manta apply sat --sat-template-file my_cluster_sat/template.yaml --values-file my_cluster_sat/session_vars.yaml
```

---

**Q: How to move nodes from cluster `A` to cluster `B`?**

**A:** To move nodes `form nodes_free` to `my_cluster` run the command below

```bash
$ manta add nodes --target-cluster my_cluster --parent-cluster nodes_free --no-dryrun x1003c1s7b0n0,x1003c1s7b0n1,x1003c1s7b1n0
```

---

**Q: How to apply changes during runtime without generating an image?**

**A:** To apply changes to a cluster during runtime, run the command below

```bash
$ manta set runtime-configuration -H my_cluster -c my_configuration
```

---

**Q: How can I list all configurations available in the system?**

**A:** The simplest command to get all configurations available to a user is `manta get configurations`. If the user has system wide access, then this command will return all configurations in the system, otherwise, it will return only the configurations used by the cluster(s) the user has access to.

---

**Q: How can I filter configurations by cluster name and configuration name?**

**A:** `manta get configurations -H my_cluster --name my_configuration`

---

**Q:  How can I list the most recent configuration?**

**A:** `manta get configurations --most-recent`

---

**Q: How can I get details of all configuration layers for the most recent configuration?**

**A:** `manta get configurations --most-recent`

> Note: manta will automatically return configuration layer details when the `manta get configurations` command returns only 1 configuration

---

**Q: How to list all sessions in the system?**

**A:** `mange get sessions`

---

**Q: How to get the logs of a specific session?**

**A:** `manta logs my_session`


---

**Q: How to connect to the console of a node?**

**A:** `manta console node x1003c1s7b0n0`

---

**Q: How to power on/off or reset a node?**

**A:** `manta power on node x1003c1s7b0n0`

---

**Q: How to list the hardware components of a cluster?**

**A:** `manta get hardware cluster my_cluster`

---

**Q: How to apply changes during runtime without generating an image?**

**A:** 

`manta set runtime-configuration` command won't reboot nodes

---

**Q: How to set kernel parameters for a cluster?**

**A:** 

`manta set kernel -H my_cluster -k "ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}"`

!!!+ info "No rootfs"
      Do not add the nay parameter related to the `root filesystem` Alps compute nodes are diskless and the root filesystem is automatically configured when boot image is set

---

**Q: How to update node boot images?**

**A:** 

To update the boot image based on a configuration name (recomended) for all the nodes in a cluster `manta set boot-image -H my_cluster -c my_configuration_name`
To update the boot image based on an image id for all the nodes in a cluster `manta set boot-image -H my_cluster -i dbc5300c-3c98-4384-a7a7-28e628cbff43`

---

**Q: How to refresh the user authentication token?**

**A:** 

1. Delete the auth token for the intended system/site `manta config unset auth`
2. Check manta is using the right site `manta config show`. This command should ask user to authenticate again since the auth token was deleted in the previous command


---

**Q: error: failed to load manifest for dependency `mesa`**

**A:** 

This is caused because cargo tries to access mesa library code from local filesystem instead of crates.io repository. To fix this, make sure mesa dependency in Cargo.toml looks as below:

```

...

[dependencies]
# mesa = "0.37.7"
mesa = { path = "../mesa" } # Only for development purposes

...

```
