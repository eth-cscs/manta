# Manta User Guide

Practical walkthroughs for common cluster management tasks. This guide assumes manta is already installed and configured. See [README.md](README.md) for deployment instructions.

---

## Table of contents

1. [Checking cluster status](#1-checking-cluster-status)
2. [Managing groups](#2-managing-groups)
3. [Deploying with a SAT file](#3-deploying-with-a-sat-file)
4. [Running a CFS session from a local repo](#4-running-a-cfs-session-from-a-local-repo)
5. [Managing boot parameters](#5-managing-boot-parameters)
6. [Managing kernel parameters](#6-managing-kernel-parameters)
7. [Power management](#7-power-management)
8. [Console access](#8-console-access)
9. [Moving nodes between groups](#9-moving-nodes-between-groups)
10. [Cleaning up old configurations](#10-cleaning-up-old-configurations)
11. [Working with multiple sites](#11-working-with-multiple-sites)
12. [Non-interactive and scripted use](#12-non-interactive-and-scripted-use)

---

## 1. Checking cluster status

**List all groups:**

```bash
manta get groups
```

**Show nodes in a group with their current status:**

```bash
manta get cluster compute
manta get cluster compute -o summary          # counts per status
manta get cluster compute --status ON         # only powered-on nodes
```

**Get a flat list of xnames (useful for scripting):**

```bash
manta get cluster compute --xnames-only-one-line
```

**Check specific nodes:**

```bash
manta get nodes x3000c0s1b0n[0-7]
manta get nodes nid001313,nid001314 -o json
```

**Check recent CFS sessions:**

```bash
manta get sessions --hsm-group compute --status running
manta get sessions --most-recent
manta get sessions --limit 10 -o json
```

**Stream logs for the most recent session:**

```bash
manta log
```

---

## 2. Managing groups

**Create a group:**

```bash
manta add group --label gpu-cluster --description "A100 GPU nodes"
```

**Create a group with initial members:**

```bash
manta add group --label gpu-cluster --nodes x3000c0s1b0n[0-7]
```

**Add nodes to an existing group:**

```bash
manta add-nodes-to-groups --group gpu-cluster --nodes x3000c0s9b0n[0-3]
```

**Remove nodes from a group:**

```bash
manta remove-nodes-from-groups --group gpu-cluster --nodes x3000c0s9b0n[0-3]
```

**Delete a group** (must be empty first):

```bash
manta remove-nodes-from-groups --group gpu-cluster --nodes x3000c0s1b0n[0-7]
manta delete group gpu-cluster
```

---

## 3. Deploying with a SAT file

The SAT file is the primary deployment mechanism. A SAT file is a YAML document with up to three sections: `configurations`, `images`, and `session_templates`.

**Full deployment** (build image, then apply to nodes):

```bash
manta apply sat-file -t cluster.yaml --watch-logs
```

**Using a Jinja2 template with a values file:**

```bash
manta apply sat-file -t cluster.yaml.j2 -f values.yaml --watch-logs
```

**Override individual Jinja2 values inline:**

```bash
manta apply sat-file -t cluster.yaml.j2 \
  -V image_version=2024.1 \
  -V ansible_repo=my-config \
  --watch-logs
```

**Build image only** (skip session_templates):

```bash
manta apply sat-file -t cluster.yaml -i --watch-logs
```

**Apply session templates only** (skip image build, use existing image):

```bash
manta apply sat-file -t cluster.yaml -s --reboot
```

**Dry run to validate without making changes:**

```bash
manta apply sat-file -t cluster.yaml --dry-run
```

**Run pre/post hooks:**

```bash
manta apply sat-file -t cluster.yaml \
  --pre-hook "echo Starting deployment" \
  --post-hook "notify-team.sh deployed"
```

> The post-hook only runs on success.

---

## 4. Running a CFS session from a local repo

Use this to run Ansible from a local git repository without going through the full SAT file workflow.

**Prerequisites:** verify the repo's tags are pushed to Gitea:

```bash
manta validate-local-repo -r ~/repos/csm-config
```

**Run a session targeting a group:**

```bash
manta apply session \
  --name my-session \
  --repo-path ~/repos/csm-config \
  --hsm-group compute \
  --watch-logs
```

**Run a session targeting specific nodes:**

```bash
manta apply session \
  --name my-session \
  --repo-path ~/repos/csm-config \
  --ansible-limit x3000c0s1b0n[0-3] \
  --watch-logs
```

**Use a non-default playbook:**

```bash
manta apply session \
  --name my-session \
  --repo-path ~/repos/csm-config \
  --hsm-group compute \
  --playbook-name custom.yml \
  --ansible-verbosity 4
```

**Stream logs from an existing session:**

```bash
manta log my-session
manta log my-session --timestamps
```

---

## 5. Managing boot parameters

Boot parameters control which kernel, initrd, and image a node uses on next boot, and what kernel command-line arguments are passed.

**View current boot parameters for a group:**

```bash
manta get boot-parameters --hsm-group compute
```

**Update boot image for a whole cluster** (looks up image by CFS config name):

```bash
manta apply boot cluster compute \
  --boot-image-configuration csm-config-2024 \
  --runtime-configuration csm-config-2024 \
  --assume-yes
```

**Update boot image using a specific image ID:**

```bash
manta apply boot cluster compute \
  --boot-image 93b4ea2a-1234-5678-abcd-ef0123456789 \
  --assume-yes
```

**Update boot parameters without rebooting:**

```bash
manta apply boot cluster compute \
  --boot-image-configuration csm-config-2024 \
  --do-not-reboot \
  --assume-yes
```

**Update specific nodes instead of the whole cluster:**

```bash
manta apply boot nodes x3000c0s1b0n[0-3] \
  --boot-image-configuration csm-config-2024 \
  --assume-yes
```

**Manually set raw boot parameters:**

```bash
manta add boot-parameters \
  --hosts x3000c0s1b0n0 \
  --kernel s3://boot-images/kernel \
  --initrd s3://boot-images/initrd \
  --params "console=ttyS0,115200 ip=dhcp"
```

---

## 6. Managing kernel parameters

**View kernel parameters for a group:**

```bash
manta get kernel-parameters --hsm-group compute
```

**Filter to specific parameters:**

```bash
manta get kernel-parameters --hsm-group compute --filter console,loglevel
```

**Add a parameter** (merges, does not replace existing values):

```bash
manta add kernel-parameters "loglevel=7" --hsm-group compute
```

**Overwrite an existing parameter:**

```bash
manta add kernel-parameters "console=ttyS0,115200" --hsm-group compute --overwrite
```

**Replace all kernel parameters** (full replacement):

```bash
manta apply kernel-parameters \
  "console=ttyS0,115200 loglevel=3 ip=dhcp" \
  --hsm-group compute \
  --assume-yes
```

**Remove a specific parameter:**

```bash
manta delete kernel-parameters "loglevel" --hsm-group compute --assume-yes
```

**Skip the automatic reboot after any kernel parameter change:**

```bash
manta add kernel-parameters "loglevel=7" --hsm-group compute --do-not-reboot
```

---

## 7. Power management

**Power off a cluster gracefully:**

```bash
manta power off cluster compute --graceful --assume-yes
```

**Power on a cluster:**

```bash
manta power on cluster compute --assume-yes
```

**Power-cycle specific nodes:**

```bash
manta power reset nodes x3000c0s1b0n[0-3] --graceful --assume-yes
```

**Check power status after the operation:**

```bash
manta get cluster compute --status OFF
manta get cluster compute -o summary
```

---

## 8. Console access

**Open an interactive serial console to a node:**

```bash
manta console node x3000c0s1b0n0
```

Use the xname or NID. Press `Ctrl-]` (or the configured escape sequence) to disconnect.

**Open a shell inside the Ansible container of a running CFS session** (useful for debugging a stuck session):

```bash
manta console target-ansible my-session
```

**Launch a temporary container from an IMS image** for inspection or testing:

```bash
manta apply ephemeral-environment --image-id 93b4ea2a-1234-5678-abcd-ef0123456789
```

---

## 9. Moving nodes between groups

**Move nodes from one group to another:**

```bash
manta migrate nodes x3000c0s1b0n[0-3] \
  --from nodes_free \
  --to gpu-cluster
```

**Dry run first:**

```bash
manta migrate nodes x3000c0s1b0n[0-3] \
  --from nodes_free \
  --to gpu-cluster \
  --dry-run
```

**Backup a virtual cluster before major changes:**

```bash
manta migrate vCluster backup \
  --bos my-cluster-template \
  --destination ~/backups/my-cluster-2024-01-15
```

**Restore from backup:**

```bash
manta migrate vCluster restore \
  --bos-file ~/backups/my-cluster-2024-01-15/bos.json \
  --cfs-file ~/backups/my-cluster-2024-01-15/cfs.json \
  --hsm-file ~/backups/my-cluster-2024-01-15/hsm.json
```

---

## 10. Cleaning up old configurations

Deleting a CFS configuration also deletes all its derivatives: associated BOS session templates and IMS images.

**Delete by name pattern:**

```bash
manta delete configurations --configuration-name "old-config-*"
```

**Delete configurations in a date range:**

```bash
manta delete configurations --since 2024-01-01 --until 2024-06-01
```

**Dry run to preview what would be deleted:**

```bash
manta delete configurations --configuration-name "old-config-*" --dry-run
```

**List configurations first to confirm:**

```bash
manta get configurations --pattern "old-config-*"
```

---

## 11. Working with multiple sites

Sites are configured on **the server**. The CLI just picks which one
to address via the `X-Manta-Site` header (driven by its `site = "..."`
setting, overridable with `--site`).

Add the per-site backend connection details to
`~/.config/manta/server.toml`:

```toml
[sites.cscs_prod]
backend           = "csm"
shasta_base_url   = "https://api.cscs.ch"
root_ca_cert_file = "/etc/manta/certs/cscs_root_cert.pem"

[sites.local_test]
backend           = "ochami"
shasta_base_url   = "https://foobar.openchami.cluster:8443"
root_ca_cert_file = "/etc/manta/certs/ochami_root_cert.pem"
```

In the CLI (`~/.config/manta/cli.toml`), just point at the
`manta-server` and name the active site:

```toml
site             = "cscs_prod"
manta_server_url = "https://manta-server.example.com:8443"
```

**Switch the default site:**

```bash
manta config set site local_test
```

**Override the site for a single command:**

```bash
manta --site local_test get cluster compute
```

---

## 12. Non-interactive and scripted use

Most write commands prompt for confirmation. Suppress prompts for scripted use:

```bash
manta apply sat-file -t cluster.yaml --assume-yes
manta power off cluster compute --graceful --assume-yes
manta delete configurations --configuration-name "old-*" --assume-yes
```

**JSON output for scripting:**

```bash
manta get sessions --hsm-group compute -o json | jq '.[].name'
manta get cluster compute -o json | jq '.[].xname'
```

**Get a flat xname list:**

```bash
NODES=$(manta get cluster compute --xnames-only-one-line)
echo "Nodes: $NODES"
```

**Run manta as a server and call it via curl:**

```bash
manta serve --cert server.pem --key server-key.pem &

curl -sk -H "Authorization: Bearer $TOKEN" \
  https://localhost:8443/api/v1/sessions | jq .
```

See [API.md](API.md) for the full HTTP API reference.
