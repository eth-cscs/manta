# Cluster management with SAT file

The sat file is used by the `sat bootprep` command to create configurations, images and configure/deploy clusters. The entry points are 2 files, one used as a template and another with the values.

Manta is compatible and extends the `sat bootprep` file features.

## SAT files cannot have configurations only

Currently, the management plane does not track which cluster belongs to which configuration unless there is an `image` or a `session template` linked to it. The reason is `images` and `session templates` are linked to both configurations and nodes or clusters, therefore we need to list those in order to be able which cluster is using which configuration and be able to find out if it is relevant to the user. Due to this behaviour, manta won't process SAT files with only configurations unless they are esplicitly used in a `image` or a `session template`.

#### The example below won't work

```bash
cat my_sat_template.yml
---
configurations:
- name: "my-configuration"
  layers:
  - name: test_layer
    playbook: site.yml
    git:
      url: https://api-gw-service-nmn.local/vcs/cray/test_layer.git
      tag: 1.0  # Using git tags
```

The file neither has a `image` nor a `session_template` section, therefore there is no way for manta to track which cluster this configuration belongs to or which user has access to it.

#### The example below is correct

```bash
cat my_sat_template.yml
---
schema_version: 1.0.2
configurations:
- name: "runtime-{{vcluster.name}}-mc-{{default.suffix}}-{{vcluster.version}}"
  layers:
  - name: test_layer
    playbook: site.yml
    git:
      url: https://api-gw-service-nmn.local/vcs/cray/test_layer.git
      tag: {{test_layer.tag}}

session_templates:
- name: "{{vcluster.name}}-mc-{{default.suffix}}-{{vcluster.version}}.x86_64"
  ims:
    id: dbc5300c-3c98-4384-a7a7-28e628cbff43
  configuration: "runtime-{{vcluster.name}}-mc-{{default.suffix}}-{{vcluster.version}}"
  bos_parameters:
    boot_sets:
      compute:
        arch: X86
        kernel_parameters: ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}
        node_groups:
          {{ vcluster.sessiontemplate_group_list }}
        rootfs_provider_passthrough: "dvs:api-gw-service-nmn.local:300:hsn0,nmn0:0"
```

## Extra features

### jinja2 template engine

While sat bootprep templating seems very simple, manta follows [ansible](https://www.ansible.com/) approach and uses a jinja2 template engine.

???+ warning "Using `-` in variable names is not recommended"
      A common mistake when dealing with jinja2 files is using `-` in variable names. This won't work since jinja will expand this as a math operator, to avoid this issue, try to use `_` instead.
      
      eg:
      
      When a jinja template engine sees this `{{ template-version }}` it will try to resolve `template-version` as a mathematical expression (substract) of 2 variables (variable `template` and variable `version`). If those variables don't exists in the sessions var, then they will become `undefined` and the jinja template engine will try to substract the 2 undefined values `undefined - undefine`, then, the operation will fail. This however would work if the session vars contains:
      
      ```bash
      $ cat my_sat_vars.yml
      template: 4
      version: 3
      ```
      
      And as a result, the session template name would become `my_template_1` as a result of `4 - 3 = 1`

#### The example below will fail

template file:

```bash
cat my_sat_template.yml
---
schema_version: 1.0.2
configurations:
- name: "my_config"
  layers:
  - name: test_layer
    playbook: site.yml
    git:
      url: https://api-gw-service-nmn.local/vcs/cray/test_layer.git
      tag: v0.1.0

session_templates:
- name: "my-template-{{ template-version }}" # Won't work. Error when resolving `session-version` because `-` is a math operator. Don't use `-` in variable names
  ims:
    id: dbc5300c-3c98-4384-a7a7-28e628cbff43
  configuration: "runtime_my_config_mc"
  bos_parameters:
    boot_sets:
      compute:
        arch: X86
        kernel_parameters: ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.sct_pid_mask=0xf spire_join_token=${SPIRE_JOIN_TOKEN}
        node_groups:
          - fora
        rootfs_provider_passthrough: "dvs:api-gw-service-nmn.local:300:hsn0,nmn0:0"
```

var session file

```bash
cat my_sat_vars.yml
---
session-version: 1.0 # Won't work. Variable names should not have `-`
```

And the error will be very explicit

```bash
manta a sat -t my_sat_template.yml -f my_sat_vars.yml
...
called `Result::unwrap()` on an `Err` value: Error { kind: InvalidOperation, detail: "tried to use - operator on unsupported types undefined and undefined", name: "<string>", line: 13 }

---------------------------------- <string> -----------------------------------
  10 |       tag: {{test_layer.tag}}
  11 |
  12 | session_templates:
  13 > - name: "my-template-{{ template-version }}" # Won't work. Error when resolving `session-version` because `-` is a math operator. Don't use `-` in variable names
     i                         ^^^^^^^^^^^^^^^^ invalid operation
  14 |   ims:
  15 |     id: dbc5300c-3c98-4384-a7a7-28e628cbff43
  16 |   configuration: "runtime_my_config_mc"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
...
```

#### The example below is correct

template file:

```yaml
session_templates:
- name: "my_template_{{ template_version }}"
  ims:
    id: dbc5300c-3c98-4384-a7a7-28e628cbff43
  configuration: "runtime_my_config_mc"
...
```

var session file

```yaml
session_version: 1.0
...
```

### Git tags in configuration layers

Manta can resolve git tags in configuration layers

```yaml
---
configurations:
- name: "my-configuration"
  layers:
  - name: test_layer
    playbook: site.yml
    git:
      url: https://api-gw-service-nmn.local/vcs/cray/test_layer.git
      tag: 1.0  # manta will resolve this git tag for this repo and replace this line with `commit: <commit id/SHA>`
...
```

In the example above, we are telling manta we want Alps to use the ansible in repo `test_layer` based on git tag `1.0`. As a consequence, Manta will resolve the git tag to its commit id, set this value in the sat file configuration layer and submit the configuration craetion task to Alps management plane.

### Expand SAT template file variables with CLI arguments

Manta can expand the variables in the template file with a vars file or through CLI. Users can submit values in both vars file and as CLI arguments, if a variable value exists in both (vars file and as a CLI argument) then the CLI argument will take preference and overwrite the value in the vars file.

eg

Submit a sat file without the vars file, the sat template file only has 1 variable `version` which value is sent through CLI argument

```bash
manta a sat -t my_sat_file.yml -v "version=v1.2.0"
```

eg

This second example shows an example where a user submits 2 different values for the `version` variable. CLI arguments take preference and the final value for the `version` variable is `v1.2.0`.

```bash
cat my_sat_vars.yml
version: 0.0.5

manta a sat -t my_sat_file.yml -f my_sat_vars.yaml -v "version=v1.2.0"
```

### the `__DATE__` wildcard

Since configurations are immutable in manta, having to change the vars file to update the version may be painful to deal with (specially during development), if this is the case, then manta has a special wildcard `__DATE__` to simplify this process.

eg

Having the vars and template files below

```bash
cat my_sat_vars.yml
version: __DATE__
```

```bash
cat my_sat_file.yml
---
configurations:
- name: "my-configuration-{{ version }}"
...
```

The vars file use the `__DATE__` wildcard to substitute the version variable with current timestame everytime `manta a sat` runs, the configuration name will resolve to something like `my-configuration-20240608201045` this forces to have a different configuration name every time `manta a sat` runs

!!! info "This is only recommended during development or test/dev clusters"
    "In production, the user of `__DATE__` is discarded and a proper version name should be used instead"
