# Create a configuration file

## Configuration

By default, Manta follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) ([folder mapping per OS](https://gist.github.com/roalcantara/107ba66dfa3b9d023ac9329e639bc58c#correlations)) to find files. The configuration file needs to be under `$XDG_CONFIG_HOME/manta/` folder.

By default, Manta configuration file can be found under one of the following locations:

> - Linux: $HOME/.config/manta/config.toml
> - MacOS: $HOME/Library/Application Support/manta/config.toml

```bash
mkdir -p /home/msopena/.config/manta

cat >> ~/.config/manta/config.toml <<EOF
log = "info"

site = "alps"

[sites]

[sites.alps]
socks5_proxy = "socks5h://127.0.0.1:1080"
shasta_base_url = "https://api.cmn.alps.cscs.ch/apis"
keycloak_base_url = "https://api.cmn.alps.cscs.ch/keycloak"
gitea_base_url = "https://api.cmn.alps.cscs.ch/vcs"
k8s_api_url = "https://10.252.1.12:6442"
vault_base_url = "https://hashicorp-vault.cscs.ch:8200"
vault_secret_path = "shasta"
vault_role_id = "b15517de-cabb-06ba-af98-633d216c6d99" # vault in hashicorp-vault.cscs.ch
root_ca_cert_file = "alps_root_cert.pem"
EOF
```

Alternatively, an environment variable could be used to tell Manta where to find the configuration file `MANTA_CONFIG=/home/msopena/my_config.toml manta config show`

## Legend:

| Name                                | mandatory   | Type                          | Description                                                                                                                                                          | Example                               |
| ----------------------------------- | ----------- | ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| MANTA_CONFIG                        | no          | env                           | path to manta configuration file. If missing, then `$XDG_CONFIG/manta/confog.toml` will be used                                                                      | $HOME/my_confog.toml                  |
| MANTA_CSM_TOKEN                     | no          | env                           | CSM authentication token, if this env var is missing, then manta will prompt use for credentials against CSM keycloak                                                |                                       |
| log                                 | no          | config file                   | log details/verbosity                                                                                                                                                | off/error/warn/info/debug/trace       |
| hsm_group                           | no          | config                        | If exists, then it will filter/restrict the hsm groups and/or xnames targeted by the cli command                                                                     | psi-dev                               |
| site                                | yes         | config file                   | CSM instance manta comunicates with. Requires to have the right site in the "sites" section                                                                          | alps | prealps | alpsm                |
| sites.site_name.socks5_proxy        | yes         | config file                   | socks proxy to access the services (only needed if using manta from outside a Shasta management node. Need VPN. Need to ope your VPN IP in hashicorp  vault approle) | socks5h://127.0.0.1:1080              |
| sites.site_name.keycloak_base_url   | yes         | config file                   | Keycloak base URL for authentication                                                                                                                                 | https://api.cmn.alps.cscs.ch/keycloak |
| sites.site_name.gitea_base_url      | yes         | config file                   | Gitea base URL to fetch CFS layers git repo details                                                                                                                  | https://api.cmn.alps.cscs.ch/vcs      |
| sites.site_name.k8s_api_url         | yes         | config file                   | Shasta k8s API URL                                                                                                                                                   | https://10.252.1.12:6442              |
| sites.site_name.vault_base_url      | yes         | config file                   | Hashicorp Vault base URL storing secrets to authenticate to external services                                                                                        | https://hashicorp-vault.cscs.ch       |
| sites.site_name.vault_role_id       | yes         | config file                   | role id related to Hashicorp Vault base URL approle authentication                                                                                                   | b15517de-cabb-06ba-af98-633d216c6d99  |
| sites.site_name.vault_secret_path   | yes         | config file                   | path in vault to find secrets                                                                                                                                        | shasta | prealps                      |
| sites.site_name.shasta_base_url     | yes         | config file                   | Shasta API base URL for Shasta related jobs submission                                                                                                               | https://api-gw-service-nmn.local/apis |
| root_ca_cert_file                   | yes         | config file                   | file name with the CSM root CA. This certificate is used to trust the CSM server                                                                                     | alps_root_cert.pem                   |

## Add CSM root certificate token

To configure Manta, you need the CSM CA public root certificate. Please request this certificate from your CSM system administrator. Once you have the certificate file, place it in the same directory as the Manta configuration file. Ensure that the file name matches the value specified for `root_ca_cert_file` in the configuration file (e.g., `alps_root_cert.pem`)

## Manage multiple sites

From Manta's perspective, each site functions as an independent CSM instance. To enable Manta to communicate with different geographically distributed HPE Cray Ex systems, you need to replicate the [site.<site name>] section in the configuration file with the correct values for each site.

## Test

Run the command below to test manta installation/configuration

```bash
$ manta --help
Usage: manta [COMMAND]

Commands:
  power                Command to submit commands related to cluster/node power management
  get                  Get information from Shasta system
  add                  WIP - Add hw components to cluster
  remove               WIP - Remove hw components from cluster
  apply                Make changes to Shasta system
  migrate              WIP - Migrate vCluster
  update               Update nodes power status or boot params
  log                  get cfs session logs
  console              Opens an interective session to a node or CFS session ansible target
                           container
  delete               Deletes CFS configurations, CFS sessions, BOS sessiontemplates, BOS
                           sessions and images related to CFS configuration/s.
  validate-local-repo  Check all tags and HEAD information related to a local repo exists in
                           Gitea
  config               Manta's configuration
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

To test manta undertands your configuration file

> The output of the command below may be different than yours, this is normal since manta can deal with different systems/clusters

```bash
$ manta config show
Sites: ["alps", "prealps", "alpsm"]
Current site: alps
HSM available: ["adula", "alps", "burst", "burst_gh", "burst_mc", "eiger", "fora", "nodes_free", "psidev", "psidev_cn", "psidev_uan", "psitds", "psitds_cn", "psitds_uan", "zinal", "zinal_cta", "zinal_moleson_tds", "zinal_tds", "zinal_zinal"]
Current HSM:
Parent HSM: nodes_free
```
