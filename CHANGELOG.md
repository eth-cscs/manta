# Changelog

All notable changes to this project will be documented in this file.

## [1.54.102] - 2025-07-09

### Bug Fixes

- Error conversion

### Miscellaneous Tasks

- Update cargo.toml

## [1.54.101] - 2025-06-14

### Bug Fixes

- There are BOS sessiontemplates without CFS configuration and this creates a bug when filtering BOS sessiontemplates by CFS configuration

### Miscellaneous Tasks

- Release manta version 1.54.101

### Refactor

- Clean  code

## [1.54.100] - 2025-05-25

### Miscellaneous Tasks

- Give up musl artifact
- Release manta version 1.54.100

## [1.54.99] - 2025-05-25

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.99

## [1.54.98] - 2025-05-25

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.98

## [1.54.97] - 2025-05-25

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.97

## [1.54.96] - 2025-05-25

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.96

## [1.54.95] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.95

## [1.54.94] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.94

## [1.54.93] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.93

## [1.54.92] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.92

## [1.54.91] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.91

## [1.54.90] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.90

## [1.54.89] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.89

## [1.54.88] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.88

## [1.54.87] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.87

## [1.54.86] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.86

## [1.54.85] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.85

## [1.54.84] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.84

## [1.54.83] - 2025-05-24

### Miscellaneous Tasks

- Add musl
- Release manta version 1.54.83

## [1.54.82] - 2025-05-23

### Miscellaneous Tasks

- Update Cargo.toml
- Release manta version 1.54.82

## [1.54.81] - 2025-05-23

### Bug Fixes

- When processing a SAT file, manta hardcodes the value of field 'rootfs_provider' to 'cpss3' this was working for CSM < 1.6 but in CSM 1.6.x this value may change, therefore, we need to take this value from the SAT file

### Miscellaneous Tasks

- Release manta version 1.54.81

## [1.54.80] - 2025-05-23

### Bug Fixes

- Force ubuntu-22.04 to build artifacts

### Miscellaneous Tasks

- Release manta version 1.54.80

## [1.54.79] - 2025-05-23

### Features

- Add new artifacts for apple and linux musl

### Miscellaneous Tasks

- Release manta version 1.54.79

## [1.54.78] - 2025-05-22

### Miscellaneous Tasks

- Musl target compilation/build fails because openssl/libssl can't be found on host. To fix this, add openssl vendored feature to git2 dependency
- Release manta version 1.54.78

## [1.54.77] - 2025-05-22

### Miscellaneous Tasks

- Manta fails in bastion-alps because the version of GLIBC is too old there, the glibc version in the github runner is 2.39 but the one in bastion-alps is 2.38. This patch will set the github runner version to ubuntu-22.04 instead of ubuntu-latest
- Release manta version 1.54.77

## [1.54.76] - 2025-05-21

### Bug Fixes

- Github workflow complains it can't compile openssl on macos images, I am removing apple artifacts to I can compile because I need the artifact to create prealps images

### Miscellaneous Tasks

- Release manta version 1.54.76

## [1.54.75] - 2025-05-21

### Bug Fixes

- GitHub workflow fails because it tries to use an old ubuntu image, this fix is to update the ubuntu image used to run the GitHub workflow

### Miscellaneous Tasks

- Release manta version 1.54.75

## [1.54.74] - 2025-05-21

### Bug Fixes

- GitHub workflow fails because it tries to use an old ubuntu image, this fix is to update the ubuntu image used to run the GitHub workflow

### Miscellaneous Tasks

- Release manta version 1.54.74

## [1.54.73] - 2025-05-21

### Bug Fixes

- Rename module mesa to csm-rs
- Rename module form mesa to csm-rs
- Logic to get repo name from gitea repo url needs to know the site manta is using otherwise it won't be able to trim the repo url prefix to get the repo name

### Miscellaneous Tasks

- Update Cargo.toml
- Release manta version 1.54.73

## [1.54.72] - 2025-05-21

### Bug Fixes

- Image creation date format

### Miscellaneous Tasks

- Release manta version 1.54.72

## [1.54.71] - 2025-04-18

### Bug Fixes

- Sat file schema compatibility

### Miscellaneous Tasks

- Release manta version 1.54.71

## [1.54.70] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.70

## [1.54.69] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.69

## [1.54.68] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.68

## [1.54.67] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.67

## [1.54.66] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.66

## [1.54.65] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.65

## [1.54.64] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.64

## [1.54.63] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.63

## [1.54.62] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.62

## [1.54.61] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.61

## [1.54.60] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.60

## [1.54.59] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.59

## [1.54.58] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.58

## [1.54.57] - 2025-04-17

### Miscellaneous Tasks

- Fix github workflow
- Release manta version 1.54.57

## [1.54.56] - 2025-04-17

### Bug Fixes

- Update ubuntu image version in github runner for github workflow

### Miscellaneous Tasks

- Release manta version 1.54.56

## [1.54.55] - 2025-04-16

### Miscellaneous Tasks

- Check github workflow
- Release manta version 1.54.55

## [1.54.54] - 2025-04-16

### Miscellaneous Tasks

- Update ubuntu image version in github workflow
- Release manta version 1.54.54

## [1.54.53] - 2025-04-16

### Bug Fixes

- Revert back CFS to v2

### Miscellaneous Tasks

- Release manta version 1.54.53

## [1.54.52] - 2025-04-11

### Miscellaneous Tasks

- Update Cargo.toml
- Release manta version 1.54.52

### Refactor

- Clean code

## [1.54.51] - 2025-04-03

### Bug Fixes

- Remove argument '--nodes' in command 'power off nodes'

### Miscellaneous Tasks

- Release manta version 1.54.51

## [1.54.50] - 2025-03-28

### Features

- Add command 'apply kernel-parameters'

### Miscellaneous Tasks

- Release manta version 1.54.50

## [1.54.49] - 2025-03-27

### Bug Fixes

- Command 'delete kernel-parameters -H <group name>:wa' only updating one node

### Miscellaneous Tasks

- Release manta version 1.54.49

## [1.54.48] - 2025-03-27

### Bug Fixes

- Command 'add kernel-parameters -H <group name>' only updating one node

### Miscellaneous Tasks

- Release manta version 1.54.48

## [1.54.47] - 2025-03-27

### Bug Fixes

- Command 'add kernel-parameters' overwriting existing kernel parameters with new ones

### Miscellaneous Tasks

- Release manta version 1.54.47

### Refactor

- Clean code

## [1.54.46] - 2025-03-25

### Bug Fixes

- Bug in command 'get hw-component alps'

### Miscellaneous Tasks

- Release manta version 1.54.46

## [1.54.45] - 2025-03-25

### Bug Fixes

- Pa_admin have access to all groups

### Miscellaneous Tasks

- Release manta version 1.54.45

## [1.54.44] - 2025-03-22

### Features

- Add group to audit messages

### Miscellaneous Tasks

- Release manta version 1.54.44

## [1.54.43] - 2025-03-21

### Features

- Homebrew tap github deployment

### Miscellaneous Tasks

- Release manta version 1.54.43

## [1.54.42] - 2025-03-21

### Bug Fixes

- Command 'delete session' ignoring argument 'assume-yes'

### Miscellaneous Tasks

- Release manta version 1.54.42

## [1.54.41] - 2025-03-21

### Features

- Command 'delete session' has a new argument 'assume-yes' to avoid asking user for confirmation

### Miscellaneous Tasks

- Release manta version 1.54.41

## [1.54.40] - 2025-03-21

### Features

- Command 'apply template' not accepts 'limit' as a mandatory argument

### Miscellaneous Tasks

- Release manta version 1.54.40

## [1.54.39] - 2025-03-11

### Miscellaneous Tasks

- Update Cargo.toml
- Release manta version 1.54.39

## [1.54.38] - 2025-03-10

### Miscellaneous Tasks

- Update Cargo.toml
- Release manta version 1.54.38

## [1.54.37] - 2025-03-10

### Miscellaneous Tasks

- Update Cargo.toml
- Release manta version 1.54.37

## [1.54.36] - 2025-03-10

### Bug Fixes

- Command 'apply sat-file' process CFS session groups in a better way by cleaning both available groups (jwt/auth) token and groups in SAT images group section to validate if SAT file ran by user has access to the groups the user has access to

### Miscellaneous Tasks

- Release manta version 1.54.36

## [1.54.35] - 2025-03-06

### Features

- Clean code

### Miscellaneous Tasks

- Release manta version 1.54.35

## [1.54.34] - 2025-03-05

### Features

- Add new command to delete configurations and cascade

### Miscellaneous Tasks

- Release manta version 1.54.34

## [1.54.33] - 2025-03-05

### Bug Fixes

- Fail processing 'delete session'

### Miscellaneous Tasks

- Release manta version 1.54.33

## [1.54.32] - 2025-03-04

### Bug Fixes

- Add nodes to hsm group command works with nodes that are members of a system wide hsm group (eg alps, prealps, aplsm, etc)

### Miscellaneous Tasks

- Release manta version 1.54.32

## [1.54.31] - 2025-03-03

### Bug Fixes

- Manta always going to gitea instance in alps to get repo information
- In alpsb there are repos without refs, this was making manta to panic, from now on refs in gitea repos are not mandatory and will be ignored if repo does ont have them

### Miscellaneous Tasks

- Fix Cargo.toml
- Release manta version 1.54.31

## [1.54.30] - 2025-03-03

### Miscellaneous Tasks

- Clean keycloak roles
- Release manta version 1.54.30

## [1.54.29] - 2025-02-28

### Miscellaneous Tasks

- Show error when fetching a CFS session while waiting session to finish
- Release manta version 1.54.29

## [1.54.28] - 2025-02-27

### Features

- When powering off and reset --force if the default

### Miscellaneous Tasks

- Release manta version 1.54.28

## [1.54.27] - 2025-02-22

### Bug Fixes

- Apply sat file fails while watching logs because it won't wait teardown stage or the CFS session to complete

### Miscellaneous Tasks

- Release manta version 1.54.27

## [1.54.26] - 2025-02-21

### Bug Fixes

- Filter cfs session including generic sessions

### Miscellaneous Tasks

- Release manta version 1.54.26

## [1.54.25] - 2025-02-21

### Bug Fixes

- 'get session' command ignoring "generic" sessions

### Miscellaneous Tasks

- Release manta version 1.54.25

## [1.54.24] - 2025-02-13

### Features

- Commands add/delete group now audits their operations to kafka

### Miscellaneous Tasks

- Release manta version 1.54.24

## [1.54.23] - 2025-02-12

### Bug Fixes

- Disable build script

### Miscellaneous Tasks

- Release manta version 1.54.23

## [1.54.22] - 2025-02-12

### Features

- New functionality to add and delete groups
- Add new command to add new group
- Add new command to delete a group

### Miscellaneous Tasks

- Release manta version 1.54.22

## [1.54.21] - 2025-02-11

### Features

- Add cli hints for shell autocomplete
- Update strum version

### Miscellaneous Tasks

- Release manta version 1.54.21

### Refactor

- Rename command to generate autocomplete script

## [1.54.20] - 2025-02-11

### Features

- Add shell autocomplete hints

### Miscellaneous Tasks

- Release manta version 1.54.20

## [1.54.19] - 2025-02-09

### Bug Fixes

- Disable x86_64-unknown-linux-musl in github workload untill we fix the kafka dependency to musl-gcc

### Miscellaneous Tasks

- Release manta version 1.54.19

## [1.54.18] - 2025-02-08

### Features

- Update audit messages:
- Power off nodes/cluster
- Power on nodes/cluster
- Power reset nodes/cluster
- Add kernl parameters
- Add nodes to group
- Apply sat file
- Apply session
- Delete kernel parameters
- Delete session
- Migrate nodes between groups
- Remove nodes from group
- Set boot configuration
- Set boot image

### Miscellaneous Tasks

- Release manta version 1.54.18

## [1.54.17] - 2025-02-06

### Features

- Add kafka auditing to 'power on nodes' command
- Update kafka library

### Miscellaneous Tasks

- Release manta version 1.54.17

### Feag

- Add audit functionality to the following operations:

## [1.54.16] - 2025-02-05

### Features

- Shell autocomplete scripts generated on compilation time

### Miscellaneous Tasks

- Release manta version 1.54.16

## [1.54.15] - 2025-02-05

### Bug Fixes

- Cli typo

### Miscellaneous Tasks

- Release manta version 1.54.15

## [1.54.14] - 2025-02-05

### Features

- Remove aliases in cli commands

### Miscellaneous Tasks

- Release manta version 1.54.14

## [1.54.13] - 2025-02-04

### Features

- Get logs from host (xname or nid)

### Miscellaneous Tasks

- Release manta version 1.54.13

## [1.54.12] - 2025-02-04

### Bug Fixes

- Bug in cli

### Miscellaneous Tasks

- Release manta version 1.54.12

## [1.54.11] - 2025-02-03

### Features

- Add autocomplete command

### Miscellaneous Tasks

- Release manta version 1.54.11

### Refactor

- Clean code

## [1.54.10] - 2025-01-30

### Miscellaneous Tasks

- Release manta version 1.54.10

## [1.54.9] - 2025-01-30

### Miscellaneous Tasks

- Release manta version 1.54.9

## [1.54.8] - 2025-01-30

### Features

- Add feature to include 'siblings' in 'get nodes' command

### Miscellaneous Tasks

- Release manta version 1.54.8

## [1.54.7] - 2025-01-29

### Features

- Command 'get nodes' now accepts nids

### Miscellaneous Tasks

- Release manta version 1.54.7

## [1.54.6] - 2025-01-28

### Miscellaneous Tasks

- Release manta version 1.54.6

### Refactor

- Clean code

## [1.54.5] - 2025-01-28

### Bug Fixes

- Update github workflow

### Miscellaneous Tasks

- Release manta version 1.54.5

## [1.54.4] - 2025-01-28

### Bug Fixes

- Update github workflow

### Features

- Clean code
- Node console now accepts nid

### Miscellaneous Tasks

- Release manta version 1.54.4

## [1.54.3] - 2025-01-23

### Features

- Restrict admin users to the ones with kc role pa_admin

### Miscellaneous Tasks

- Release manta version 1.54.3

## [1.54.2] - 2025-01-22

### Bug Fixes

- Power management operations exit if list of nodes after expanding user input is empty

### Features

- Upgrade github pipeline

### Miscellaneous Tasks

- Release manta version 1.54.2

## [1.54.1] - 2025-01-22

### Features

- Power commands now accepts nid nodes
- Power commands now accepts nid nodes

### Miscellaneous Tasks

- Release manta version 1.54.1

## [1.54.0] - 2024-11-23

### Bug Fixes

- Boot parameter operations
- Bug changing boot image

### Miscellaneous Tasks

- Release manta version 1.54.0

### Refactor

- Cli operations

## [1.53.21] - 2024-11-12

### Features

- Set kernel parameters was changing the kernel value

### Miscellaneous Tasks

- Release manta version 1.53.21

## [1.53.20] - 2024-11-11

### Miscellaneous Tasks

- Release manta version 1.53.20

### Refactor

- Cfs_configuration.branch and cfs_configuration.tag are now based on a list of values a specific commit may be related to

## [1.53.19] - 2024-11-11

### Miscellaneous Tasks

- Release manta version 1.53.19

### Refactor

- Cfs_configuration.branch and cfs_configuration.tag are now based on a list of values a specific commit may be related to

## [1.53.18] - 2024-11-09

### Features

- Update manta with new operations to manage kernel parameters

### Miscellaneous Tasks

- Release manta version 1.53.18

### Refactor

- Clean code

## [1.53.18-alpha.1] - 2024-11-08

### Features

- Improve command 'apply boot' understanding when nodes needs to be
- Booted

### Miscellaneous Tasks

- Release manta version 1.53.18-alpha.1

### Refactor

- Clean code/modules

## [1.53.17] - 2024-11-07

### Bug Fixes

- Update mesa to fix bug with apply sat command

### Miscellaneous Tasks

- Release manta version 1.53.17

## [1.53.16] - 2024-11-06

### Bug Fixes

- Command 'set runtime-configuration' validates list of xnames
- Command 'set boot-configuration' validates list of xnames
- Command 'set boot-image' validates list of xnames
- Command 'apply boot' validates list of xnames

### Miscellaneous Tasks

- Release manta version 1.53.16

## [1.53.15] - 2024-11-06

### Bug Fixes

- Argument 'limit' in apply template subcommand should not be mandatory

### Miscellaneous Tasks

- Release manta version 1.53.15

## [1.53.14] - 2024-11-05

### Features

- Apply sat-file command can now show logs when creating images

### Miscellaneous Tasks

- Release manta version 1.53.14

## [1.53.13] - 2024-11-04

### Miscellaneous Tasks

- Update cicd pipleine
- Release manta version 1.53.13

## [1.53.12] - 2024-11-04

### Features

- Sat templates now accepts lines starting with '#' as comments
- Sat template rendering fails if values are missing
- Sat template rendering ebug enabled for better errors

### Miscellaneous Tasks

- Release manta version 1.53.12

## [1.53.11] - 2024-11-04

### Bug Fixes

- Add debug messages with rendering jinja templates

### Features

- Update mesa

### Miscellaneous Tasks

- Release manta version 1.53.11

### Refactor

- Clean code
- Improve jinja template rendering error message

## [1.53.10] - 2024-10-31

### Features

- Improve performance in get cluster command

### Miscellaneous Tasks

- Release manta version 1.53.10

## [1.53.9] - 2024-10-31

### Miscellaneous Tasks

- Release manta version 1.53.9

### Refactor

- Clean code

## [1.53.8] - 2024-10-28

### Miscellaneous Tasks

- Release manta version 1.53.8

### Refactor

- Fix lint warning messages

## [1.53.7] - 2024-10-28

### Bug Fixes

- Apply sessions not filtering CFS sessions

### Features

- Update mesa

### Miscellaneous Tasks

- Release manta version 1.53.7

### Refactor

- Clean lint warning messages

## [1.53.6] - 2024-10-28

### Bug Fixes

- Fail in validating HSM group user has access to

### Miscellaneous Tasks

- Release manta version 1.53.6

## [1.53.5] - 2024-10-28

### Bug Fixes

- Fetch commit id details

### Miscellaneous Tasks

- Release manta version 1.53.5

## [1.53.4] - 2024-10-28

### Bug Fixes

- Compilation error

### Miscellaneous Tasks

- Release manta version 1.53.4

## [1.53.3] - 2024-10-28

### Features

- Add new command 'delete images'

### Miscellaneous Tasks

- Release manta version 1.53.3

## [1.53.2] - 2024-10-27

### Bug Fixes

- Update mesa to integrate CFS sessions type dynamic creation:wa

### Features

- Add argument 'ansible-playbook-name' to command 'apply session'

### Miscellaneous Tasks

- Release manta version 1.53.2

## [1.53.1] - 2024-10-25

### Bug Fixes

- Version number

### Miscellaneous Tasks

- Release manta version 1.53.1

## [1.53.0] - 2024-10-25

### Bug Fixes

- Command apply session

### Feature

- Answer yes to questions during apply sat file. (#90)

### Features

- Power commands now support hostlist and regex
- Power commands now shows a dialog asking for permission to proceed
- [**breaking**] Command power on cluster has new argument to skip prompts
- [**breaking**] Command power on nodes has new argument to skip prompts
- [**breaking**] Command power off cluster has new argument to skip prompts
- [**breaking**] Command power off nodes has new argument to skip prompts
- [**breaking**] Command power reset cluster has new argument to skip prompts
- [**breaking**] Command power reset nodes has new argument to skip prompts
- [**breaking**] Command apply boot nodes has new argument to skip prompts
- [**breaking**] Command apply set boot image has new argument to skip prompts
- [**breaking**] Command apply apply boot cluster has new argument to skip prompts
- Add log level information to command config show
- Images containing 'generic' in their names are now available to all users
- Power commands now support hostlist and regex
- Power commands now shows a dialog asking for permission to proceed
- [**breaking**] Command power on cluster has new argument to skip prompts
- [**breaking**] Command power on nodes has new argument to skip prompts
- [**breaking**] Command power off cluster has new argument to skip prompts
- [**breaking**] Command power off nodes has new argument to skip prompts
- [**breaking**] Command power reset cluster has new argument to skip prompts
- [**breaking**] Command power reset nodes has new argument to skip prompts
- [**breaking**] Command apply boot nodes has new argument to skip prompts
- [**breaking**] Command apply set boot image has new argument to skip prompts
- [**breaking**] Command apply apply boot cluster has new argument to skip prompts
- Add log level information to command config show
- Images containing 'generic' in their names are now available to all users

### Miscellaneous Tasks

- Release manta version 1.53.0

## [1.52.2] - 2024-10-21

### Features

- [**breaking**] Dryrun features in commands `add nodes to group` and `remove
- Nodes to group` inverted
- [**breaking**] Remove feature to create hsm group in command `add nodes to group`
- [**breaking**] Remove feature to clean hsm group in command `remove nodes to group`
- Update mesa
- Command to `add nodes to group` now accepts regex
- Command to `remove nodes to group` now accepts regex
- [**breaking**] Command `add nodes to group` to shows a dialog asking user for
- Configuration
- [**breaking**] Command `remove nodes from group` to shows a dialog asking user for
- Configuration

### Miscellaneous Tasks

- Release manta version 1.52.2

### Refactor

- JWT operations
- Clean code

## [1.52.1] - 2024-10-18

### Features

- Update mesa to wait CFS sessions longer

### Miscellaneous Tasks

- Release manta version 1.52.1

## [1.52.0] - 2024-10-18

### Features

- Add new command 'add-nodes-to-groups' to add list of nodes to a list of groups
- Add new command 'remove-nodes-to-groups' to remove list of nodes to a list of groups

### Miscellaneous Tasks

- Release manta version 1.52.0

## [1.51.3] - 2024-10-14

### Features

- 'get template' command now prints data in json format

### Miscellaneous Tasks

- Release manta version 1.51.3

## [1.51.2] - 2024-10-14

### Bug Fixes

- Update mesa

### Miscellaneous Tasks

- Release manta version 1.51.2

## [1.51.1] - 2024-10-14

### Bug Fixes

- Improve output message

### Miscellaneous Tasks

- Release manta version 1.51.1

## [1.51.0] - 2024-10-14

### Features

- Add new command 'apply template' to crate a new BOS session from a BOS sessiontemplate

### Miscellaneous Tasks

- Release manta version 1.51.0

## [1.50.18] - 2024-10-14

### Bug Fixes

- Set mandatory arguments to migrate nodes command

### Features

- Prepare HSM goup operations for next version

### Miscellaneous Tasks

- Release manta version 1.50.18

### Refactor

- Create new function to get a curated list of hosts from a
- Hostslist

## [1.50.17] - 2024-10-11

### Features

- Migration node command now accepts a hostlist as list of input nodes

### Miscellaneous Tasks

- Release manta version 1.50.17

### Refactor

- Update cli docs

## [1.50.16] - 2024-10-04

### Features

- Migrate to CFS configuration v3

### Miscellaneous Tasks

- Release manta version 1.50.16

## [1.50.15] - 2024-10-04

### Bug Fixes

- Remove cli commands deprecated
- Command 'apply boot node'
- Command 'set kernel-parameters'

### Miscellaneous Tasks

- Release manta version 1.50.15

## [1.50.14] - 2024-10-03

### Bug Fixes

- Set kernel parameters

### Miscellaneous Tasks

- Release manta version 1.50.14

## [1.50.13] - 2024-10-03

### Bug Fixes

- Argument parsing in 'power on cluster' command

### Miscellaneous Tasks

- Release manta version 1.50.13

## [1.50.12] - 2024-10-01

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.50.12

## [1.50.11] - 2024-10-01

### Miscellaneous Tasks

- Release manta version 1.50.11

### Refactor

- Clean cli commands

## [1.50.10] - 2024-09-29

### Bug Fixes

- Command 'migrate nodes' validate HSM groups

### Miscellaneous Tasks

- Release manta version 1.50.10

### Refactor

- Housekeeping with module files

## [1.50.9] - 2024-09-29

### Miscellaneous Tasks

- Release manta version 1.50.9

### Refactor

- Clean code

## [1.50.8] - 2024-09-28

### Features

- Init code for apply sat file dry-run
- New command 'migrate nodes' to move compute nodes from one cluster

### Miscellaneous Tasks

- Release manta version 1.50.8

### Refactor

- Clean code
- Remove commands 'add nodes' and 'remove nodes' since they have been changed to 'migrate nodes'

## [1.50.7] - 2024-09-28

### Miscellaneous Tasks

- Release manta version 1.50.7

### Refactor

- Apply_sat_file code

## [1.50.6] - 2024-09-27

### Bug Fixes

- Error checking cli help if manta not fully configured

### Miscellaneous Tasks

- Release manta version 1.50.6

## [1.50.5] - 2024-09-27

### Features

- Upgrade mesa version

### Miscellaneous Tasks

- Release manta version 1.50.4
- Release manta version 1.50.5

## [1.50.4] - 2024-09-27

### Bug Fixes

- Fix unit tests
- Unit tests
- Imports
- Mesa repo

### Features

- Debug is always while creating an image form sat file
- Command 'delete-session' has a new 'dry-run' command
- 'delete session' command also cleans images when target is 'image'
- Improve cli description message for command 'delete session:wa'

### Miscellaneous Tasks

- Release manta version 1.50.4

### Refactor

- Organise modules

## [1.50.3] - 2024-09-23

### Bug Fixes

- Command 'get kernel-parameters' for a cluster combined with filter not grouping hsm groups correctly

### Miscellaneous Tasks

- Release manta version 1.50.3

## [1.50.2] - 2024-09-22

### Bug Fixes

- Delete CFS session

### Features

- Update mesa library

### Miscellaneous Tasks

- Release manta version 1.50.2

## [1.50.1] - 2024-09-22

### Bug Fixes

- Command 'console target-ansible' breaking local terminal when

### Features

- Add 'output' argument to 'get kernel-parameters' command
- Add 'debug' argument to 'apply sat' command
- Exiting
- Migrate CFS API to v3
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.50.1

## [1.50.0] - 2024-09-18

### Features

- New command 'get nodes' to query a list of xnames from different HSM groups

### Miscellaneous Tasks

- Release manta version 1.50.0

## [1.49.5] - 2024-09-18

### Features

- Command 'get cluster' sorts the HSM groups each node belongs to

### Miscellaneous Tasks

- Release manta version 1.49.5

## [1.49.4] - 2024-09-18

### Features

- 'get cluster' command now displays the list of HSM groups in multiple lines to make better use of screen real estate

### Miscellaneous Tasks

- Release manta version 1.49.4

## [1.49.3] - 2024-09-18

### Features

- Add HSM group name in 'get cluster' command output

### Miscellaneous Tasks

- Release manta version 1.49.3

## [1.49.2] - 2024-09-17

### Features

- Improve performance when running command "get cluster"

### Miscellaneous Tasks

- Release manta version 1.49.2

## [1.49.1] - 2024-09-16

### Features

- Subcommand get kernel parameters group kernel parameters by xnames

### Miscellaneous Tasks

- Release manta version 1.49.1

## [1.49.0] - 2024-09-09

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.49.0

## [1.48.0] - 2024-09-06

### Bug Fixes

- Command "set boot-image" argument mismatch
- Error message when 'root_ca_cert' param in config file is missing
- And provides a more useful message to user
- Output messages

### Features

- Command 'set boot-image' use PCS module to restart nodes
- Command 'set'boot-image' filters boot parameters and nodes to boot
- Command 'set boot-image' accepts new argument 'output' to print

### Miscellaneous Tasks

- Release manta version 1.48.0

### Fex

- Command 'set boot-image' output argument default value

### Rafactor

- Clean code

## [1.47.2] - 2024-09-05

### Bug Fixes

- Improve cli interface of 'get kernel-parameters' subcommand

### Miscellaneous Tasks

- Release manta version 1.47.2

## [1.47.1] - 2024-08-25

### Bug Fixes

- Update Cargo.toml

### Miscellaneous Tasks

- Release manta version 1.47.1

## [1.47.0] - 2024-08-25

### Features

- Power management operations report
- Set kernel parameters command asks user for confirmation
- Add pcs utils
- Power management commands now accept a new argument 'output' to change the output format
- Update mesa
- Power management commands now accept a new argument 'output' to change the output format
- Add pcs utils
- Power management operations report
- Set kernel parameters command asks user for confirmation

### Miscellaneous Tasks

- Release manta version 1.47.0

### Refactor

- Clean code
- Integration with PCS
- Clean code
- Integration with PCS

## [1.46.20] - 2024-08-22

### Miscellaneous Tasks

- Release manta version 1.46.20

### Refactor

- Rename get kernel-parameters argument

## [1.46.19] - 2024-08-22

### Bug Fixes

- But in set subcommand

### Miscellaneous Tasks

- Release manta version 1.46.19

### Refactor

- Clean code

## [1.46.18] - 2024-08-21

### Bug Fixes

- Improve error management when get logs command fails

### Features

- Add new method to stop a cfs session
- Improve functionality to stop a cfs session
- Stop running session checks is session to stop is actually running, otherwise, it gracefulyl stops
- Apply sat command now translates git branches to commit id when
- Creating CFS configurations

### Miscellaneous Tasks

- Release manta version 1.46.18

## [1.46.17] - 2024-08-16

### Bug Fixes

- Improve error management when processing SAT files

### Miscellaneous Tasks

- Release manta version 1.46.17

## [1.46.16] - 2024-08-15

### Bug Fixes

- Fix issue when changing runtie configuration would trigger manta asking user confirmation to reboot the nodes

### Miscellaneous Tasks

- Release manta version 1.46.16

## [1.46.15] - 2024-08-12

### Miscellaneous Tasks

- Release manta version 1.46.15

### Refactor

- Migrate code to migrate nodes between hsm groups to mesa

### Fis

- Bug when creating manta config file and CA root public cert file does not exists

## [1.46.14] - 2024-08-11

### Features

- Filter sat file template data base on cli arguments
- Apply sat file can now filter by image or sessiontemplate

### Miscellaneous Tasks

- Release manta version 1.46.14

## [1.46.13] - 2024-08-04

### Bug Fixes

- Get session table showing formatted stated time in status cell

### Features

- Print config in log debug

### Miscellaneous Tasks

- Release manta version 1.46.13

## [1.46.12] - 2024-08-03

### Bug Fixes

- Cli hsm argument has preference vs hsm in config file
- Log command ignores default hsm group and checks CFS session is linked to any HSM group the user has access to

### Features

- Filter sat file rendering accoring to whether user use arguments
- --image-only or --sessiontemplate-only
- Cli won't hide hsm-group arguments if default hsm has been setup
- Datetime timezone conversion format modified with "seconds"
- Datetime timezone conversion functionality extended to command
- "get session"

### Miscellaneous Tasks

- Release manta version 1.46.12

### Refactor

- Rename test file
- Code housekeeping

## [1.46.11] - 2024-07-31

### Features

- Format datetime when listing configurations and images

### Miscellaneous Tasks

- Release manta version 1.46.11

## [1.46.10] - 2024-07-31

### Bug Fixes

- Authentication bug

### Miscellaneous Tasks

- Release manta version 1.46.10

### Refactor

- Improve cli help text

## [1.46.9] - 2024-07-31

### Miscellaneous Tasks

- Release manta version 1.46.9

### Refactor

- Add aliases to help command

## [1.46.8] - 2024-07-31

### Features

- Update mesa library

### Miscellaneous Tasks

- Release manta version 1.46.8

## [1.46.7] - 2024-07-30

### Features

- Update mesa

### Miscellaneous Tasks

- Release manta version 1.46.7

## [1.46.6] - 2024-07-30

### Bug Fixes

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.46.6

## [1.46.5] - 2024-07-30

### Bug Fixes

- Config autogenerator allows to provide an empty socks5 proxy value
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.46.5

## [1.46.4] - 2024-07-29

### Bug Fixes

- Config param  will search for either full path or file inside /home/msopena/.config/manta/

### Miscellaneous Tasks

- Release manta version 1.46.4

## [1.46.3] - 2024-07-29

### Features

- Add new command  to get the list of kernel parameters for a list of nodes or a cluster
- New argument in  command to filter the list of kernel parameters listed

### Miscellaneous Tasks

- Release manta version 1.46.3

## [1.46.2] - 2024-07-29

### Features

- New output option `table-wide` for command `manta get cluster` to
- Show kernel parameters

### Miscellaneous Tasks

- Release manta version 1.46.2

### Refactor

- Housekeeping code managing config file
- Code cleaning and housekeeping

## [1.46.1] - 2024-07-29

### Bug Fixes

- Bugs managing config file

### Miscellaneous Tasks

- Release manta version 1.46.1

### Refactor

- Clean code

## [1.46.0] - 2024-07-29

### Features

- Config file autogeneration
- Config file autogeneration

### Miscellaneous Tasks

- Release manta version 1.46.0

## [1.45.3] - 2024-07-28

### Bug Fixes

- Bug in subcommand "apply template" where "limit" argument was not

### Features

- Subcommand "apply boot" now has a new argument to set new kernel
- Parameters
- Subcommand "apply template" now sets "reboot" as default operation
- Ignored and instead process all nodes in BOS sessiontemplate
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.45.2
- Release manta version 1.45.3

### Refactor

- Rename subcommand "power node" to "power nodes"

## [1.45.1] - 2024-07-26

### Bug Fixes

- Bug managing urls in config file

### Miscellaneous Tasks

- Release manta version 1.45.1

## [1.45.0] - 2024-07-26

### Features

- Simplify config file
- Update mesa

### Miscellaneous Tasks

- Release manta version 1.45.0

### Refactor

- Clean config file
- Clean config file

## [1.44.0] - 2024-07-25

### Features

- Apply sat command now has arguments to chose whether images or
- Session_template should be processed exclusively
- Migration to BOS v2
- New command `apply template` to create a BOS v2 session based on
- A BOS v2 sessiontemplate

### Miscellaneous Tasks

- Release manta version 1.44.0

## [1.43.0] - 2024-07-23

### Features

- Copy ansible templating functionality and session vars file is both a ninja template and a values file, 'manta apply sat' will render the values file with itself

### Miscellaneous Tasks

- Release manta version 1.43.0

## [1.42.3] - 2024-07-12

### Bug Fixes

- Move deprecated messages in command get nodes to log when output is json

### Miscellaneous Tasks

- Release manta version 1.42.3

## [1.42.2] - 2024-07-12

### Bug Fixes

- Move deprecated messages in command get nodes to log when output is json

### Miscellaneous Tasks

- Release manta version 1.42.2

## [1.42.1] - 2024-07-11

### Features

- Apply sat now accepts ansible_passthrough argument as env var

### Miscellaneous Tasks

- Release manta version 1.42.1

## [1.42.0] - 2024-07-07

### Bug Fixes

- Ansible-passthrough
- Ignore system hsm groups in SAT file, JWT and functions to get all
- HSM groups

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.42.0

## [1.41.6] - 2024-07-07

### Bug Fixes

- Workaround system hsm groups filtering

### Features

- Update mesa
- Get sessions related to xnames

### Miscellaneous Tasks

- Release manta version 1.41.6

### Refactor

- Clean code

## [1.41.5] - 2024-07-05

### Bug Fixes

- Error management when any HSM group in JWT token does not exists

### Miscellaneous Tasks

- Release manta version 1.41.5

## [1.41.4] - 2024-07-05

### Bug Fixes

- Update mesa vesion to fix local repo validation bug

### Miscellaneous Tasks

- Release manta version 1.41.4

## [1.41.3] - 2024-07-05

### Bug Fixes

- Update cargo dist and stdout logs

### Miscellaneous Tasks

- Release manta version 1.41.3

## [1.41.2] - 2024-07-04

### Bug Fixes

- Fix CICD error by mesa dependency in Cargo.toml

### Miscellaneous Tasks

- Release manta version 1.41.2

## [1.41.1] - 2024-07-04

### Miscellaneous Tasks

- Release manta version 1.41.1

### Refactor

- Clean code

## [1.41.0] - 2024-07-03

### Bug Fixes

- Power on nodes not managing power state properly

### Features

- New config parameters to specify audit file location

### Miscellaneous Tasks

- Release manta version 1.41.0

### Refactor

- Update documentation

## [1.40.0] - 2024-07-03

### Bug Fixes

- Improve cli help
- Migrate from BOS v1 to BOS v2
- Arggroup bug
- Fix import
- IMS job creation returns CSM error msg is request failt

### Features

- Add cli command
- Add new command `manta set boot-image`
- Add new command `manta set boot-configuration`
- New env var MANTA_CONFIG to set the path for the configuration file
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.39.0
- Release manta version 1.40.0

### Refactor

- Clean code

## [1.38.1] - 2024-06-28

### Bug Fixes

- Improve deprecated messages

### Miscellaneous Tasks

- Release manta version 1.38.1

## [1.38.0] - 2024-06-28

### Bug Fixes

- Update mesa version

### FEAT

- Command 'apply hw cluster' now can reuse nodes in 'target' HSM group

### Features

- Integrate "pin" and "unpin" features to "apply hw cluster" command

### Miscellaneous Tasks

- Release manta version 1.38.0

### Refactor

- Clean code
- Code housekeeping
- Rename apply_hw_cluster modules according to pin and unpin strategy

## [1.37.0] - 2024-06-12

### FEAT

- Add new config file parameter to store CSM CA public root filename

### FIX

- Improve errors in configuration file

### Miscellaneous Tasks

- Release manta version 1.37.0

## [1.36.3] - 2024-06-12

### FEAT

- Update manta version

### Miscellaneous Tasks

- Release manta version 1.36.3

### Refactor

- Clean code

## [1.36.2] - 2024-06-09

### Bug Fixes

- HSM list to validate in apply sat file no longer takes into
- Consideration the HSM group in configuration file

### FIX

- Delete now does not deletes images from BOS sessiontemplate params.
- Process cli with wrong commands

### Miscellaneous Tasks

- Release manta version 1.36.2

### REFACTOR

- Fix subcommands

### Refactor

- Clean code

## [1.36.1] - 2024-06-02

### Bug Fixes

- Use new mesa library to fix issue getting commit id details form gitea

### Miscellaneous Tasks

- Release manta version 1.36.1

## [1.36.0] - 2024-06-02

### Bug Fixes

- Cli help

### Co-authored-by

- Manuel Sopena Ballesteros <manuel.sopena@cscs.ch>

### Features

- Apply sat command now support pre and port hooks

### Miscellaneous Tasks

- Release manta version 1.36.0

## [1.35.8] - 2024-05-29

### Bug Fixes

- Enable openssl-vendores feature got git2 crate to avoid breaking apple images during CI/CD pipeline
- Update boot parameters

### Miscellaneous Tasks

- Release manta version 1.35.8

## [1.35.7] - 2024-05-28

### Bug Fixes

- Try to fix ci/cd pipeline building openssl-sys

### Miscellaneous Tasks

- Release manta version 1.35.7

## [1.35.6] - 2024-05-28

### Bug Fixes

- Init cargo dist

### Miscellaneous Tasks

- Release manta version 1.35.6

## [1.35.5] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.5

## [1.35.4] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.4

## [1.35.3] - 2024-05-28

### Bug Fixes

- Downgrade cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.3

## [1.35.2] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.2

## [1.35.1] - 2024-05-28

### Bug Fixes

- Bug in apply session processing the wrong HSM group

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.35.1

### Refactor

- Clean code
- Clean crates

## [1.35.0] - 2024-05-23

### Features

- New command 'get cluster <cluster name> --output summary'

### Miscellaneous Tasks

- Release manta version 1.35.0

## [1.34.0] - 2024-05-22

### Features

- Update mesa library
- Filter CFS configurations by name through glob pattern matching

### Miscellaneous Tasks

- Release manta version 1.34.0

## [1.33.0] - 2024-05-21

### Features

- Get configuration with details now shows CFS configuration derivatives (CFS sessions, BOS sessiontemplate and IMS images related to a CFS configuration)

### Miscellaneous Tasks

- Release manta version 1.33.0

## [1.32.5] - 2024-05-20

### Bug Fixes

- Improve the way hw component scarcity scores is calculated
- Print log messages

### Features

- Update hsm group members
- Update mesa version
- Update mesa version
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.32.4
- Release manta version 1.32.5

### Refactor

- Change var names

## [1.32.3] - 2024-05-02

### Bug Fixes

- Update boot only chaning the boot image of one node

### Features

- Refactor template output information
- Improve user feecback when restarting nodes

### Miscellaneous Tasks

- Release manta version 1.32.3

## [1.32.2] - 2024-05-02

### Bug Fixes

- Error management improvement

### Features

- Add semaphores when making multiples calls to CSM APIs to throttle
- The load on the system
- Upgrade mesa version

### Miscellaneous Tasks

- Release manta version 1.32.2

## [1.32.1] - 2024-05-01

### Features

- Filter image by id
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.32.1

### Refactor

- Clean output of command validate-local-repo

## [1.32.0] - 2024-05-01

### Bug Fixes

- Command get cfs configuration details failing in fetching details

### Features

- Local repo validation against gitea
- Form gitea

### Miscellaneous Tasks

- Release manta version 1.32.0

## [1.31.2] - 2024-04-30

### Features

- Format data in manta get configuration -n

### Miscellaneous Tasks

- Release manta version 1.31.2

## [1.31.1] - 2024-04-29

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.31.1

### Refactor

- Manta get cluster --> rename table results column names

## [1.31.0] - 2024-04-28

### Bug Fixes

- Bug in `apply hw cluster` command where nodes in final target hsm group would not be accurate if both target and parent hsm groups have nodes in common
- Disable DHAT crate since it is not beind used

### Features

- Add new method to change boot parameters `manta apply boot`
- Add new argument to set the image id to a cluster or nodes for
- Booting
- Update mesa
- Add tests to `apply hw cluster` functionality

### Miscellaneous Tasks

- Release manta version 1.31.0

### Refactor

- Clean code
- Comment code

## [1.30.1] - 2024-04-25

### Miscellaneous Tasks

- Release manta version 1.30.1

### Refactor

- Remove 2 columns from the CFS session output table
- Rename "DEsired Configuration" to "Running configuration" in
- 'get cluster' command

## [1.30.0] - 2024-04-23

### Bug Fixes

- Update mesa library
- Restore BOS sessiontemplate to v1
- Restore BOS session to v1
- Improve BOS sessiontemplate by processing multiple boot_sets
- Improve BOS sessiontemplate by passing kernel parameters from SAT
- File
- Process multiple BOS sessiontemplate in SAT file

### Miscellaneous Tasks

- Release manta version 1.30.0

## [1.29.5] - 2024-04-17

### Features

- Print most recent CFS session logs related to a cluster

### Miscellaneous Tasks

- Release manta version 1.29.5

### Refactor

- Fix mesa library location

## [1.29.4] - 2024-04-17

### Features

- Add new feature to filter CFS sessions by min_age and max_age parameters

### Miscellaneous Tasks

- Release manta version 1.29.4

## [1.29.3] - 2024-04-17

### Features

- Add new feature to filter CFS sessions by min_age and max_age parameters

### Miscellaneous Tasks

- Release manta version 1.29.3

## [1.29.2] - 2024-04-16

### Bug Fixes

- Bug filtering CFS sessions through HSM groups
- Fix bug filtering bos sessiontemplate by HSM group
- Print BOS sessiontemplate information properly by removing the type column in table

### Features

- Add functionality to filter CFS sessions by state

### Miscellaneous Tasks

- Release manta version 1.29.2

## [1.29.1] - 2024-04-03

### Bug Fixes

- CFS layer branch lookup not printing branch name properly if they
- Had character "/"

### Miscellaneous Tasks

- Release manta version 1.29.1

### Refactor

- Clean code

## [1.29.0] - 2024-04-02

### Bug Fixes

- Stop fetching all HSM groups available to the user if no roles

### Features

- SAT file processing now accepts

### Miscellaneous Tasks

- Release manta version 1.29.0

### Refactor

- Cean code
- Found in JWT token

## [1.28.14] - 2024-03-17

### Miscellaneous Tasks

- Release manta version 1.28.14

### Refactor

- Move tests to /test/ directory

## [1.28.13] - 2024-03-15

### Features

- Add selection prompmt to delete auth token

### Miscellaneous Tasks

- Release manta version 1.28.13

## [1.28.12] - 2024-03-15

### Bug Fixes

- Config unset hsm command

### Miscellaneous Tasks

- Release manta version 1.28.12

## [1.28.11] - 2024-03-15

### Bug Fixes

- Format cfs layer data and clean stoud log traces

### Miscellaneous Tasks

- Release manta version 1.28.11

## [1.28.10] - 2024-03-15

### Features

- Show cfs configuration layer table in different rows
- User new mesa library version

### Miscellaneous Tasks

- Release manta version 1.28.10

## [1.28.9] - 2024-03-14

### Features

- Handle auth tokens for multiple sites at the same time

### Miscellaneous Tasks

- Release manta version 1.28.9

## [1.28.8] - 2024-03-13

### Bug Fixes

- BUG SAT file session_template validation ignoring previous SAT file version

### Miscellaneous Tasks

- Release manta version 1.28.8

## [1.28.7] - 2024-03-12

### Co-authored-by

- Manuel Sopena Ballesteros <manuel.sopena@cscs.ch>

### Features

- Improve SAT file validation to improve user feedback
- Test new cc to build apple target binaries

### Miscellaneous Tasks

- Release manta version 1.28.7

## [1.28.6] - 2024-03-04

### Bug Fixes

- Remove apply artifacts/targets from CI pipeline

### Miscellaneous Tasks

- Release manta version 1.28.6

## [1.28.5] - 2024-03-03

### Features

- Update mesa

### Miscellaneous Tasks

- Release manta version 1.28.5

### Refactor

- Clean code and adapt to new mesa version

## [1.28.4] - 2024-03-01

### Features

- Update command no longer reboot nodes if boot image did not change
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.28.4

### Refactor

- Clean code

## [1.28.3] - 2024-03-01

### Features

- Prepare to substitute apply configuration, apply image and apply cluster to apply sat-file

### Miscellaneous Tasks

- Release manta version 1.28.3

## [1.28.2] - 2024-03-01

### Bug Fixes

- Test apple artifacts

### Miscellaneous Tasks

- Release manta version 1.28.2

## [1.28.1] - 2024-03-01

### Features

- Get configuration command shows layer information including tag
- Names
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.28.1

### Refactor

- Clean code

### Testing

- Test branch

## [1.28.0] - 2024-02-28

### Bug Fixes

- Add dialog asking user to validate SAT file for commands apply
- Image and apply configuration

### Miscellaneous Tasks

- Release manta version 1.28.0

## [1.27.0] - 2024-02-28

### Bug Fixes

- Bug in apply cluster subcommand where it was failing when reading
- Bos sessiontemplate details
- Now interacts with mesa functions to update HSM group members

### Features

- Show more detailed information related to CFS configuration
- Commands add node, remove, node, add, hw, remove, hw and apply hw
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.27.0

### Refactor

- Clean code

## [1.26.0] - 2024-02-25

### Features

- Get hw components subcommands now can print information as a summary of all hw components in a cluster

### Miscellaneous Tasks

- Release manta version 1.26.0

## [1.25.1] - 2024-02-25

### Miscellaneous Tasks

- Release manta version 1.25.1

### Refactor

- Reformat how CFS configuration layer details are printed on screen

## [1.25.0] - 2024-02-24

### Bug Fixes

- Get configuration, get image, get template was filtering way too
- Ci pipeline not generating homebrew installation command till cc
- Crate issue is fixed

### Features

- Get configuration now resolves gitea information like branch and
- Tag name and also checks if commit id used if the most recent one
- Compared to the tip on remote for that branch
- Much informtion
- Update mesa library

### Miscellaneous Tasks

- Release manta version 1.25.0

## [1.24.2] - 2024-02-23

### Bug Fixes

- SAT file templating
- Disable apple targets due to a bug in cc crate used by openssl crate

### Features

- Replace __DATE__ in SAT file vars file and cli vars for timestamp

### Miscellaneous Tasks

- Release manta version 1.24.2

### Refactor

- Clean stdout messages

## [1.24.1] - 2024-02-23

### Bug Fixes

- HSM validation for admin users

### Features

- Integrate parent HSM to config file
- Integratino parent HSM to `add hw`, `add node`, `remove hw`, `remove node` and `apply hw` subcommands

### Miscellaneous Tasks

- Release manta version 1.24.1

### Refactor

- Improve stdout messages

## [1.24.0] - 2024-02-22

### Bug Fixes

- Mesa library
- Manta version

### Features

- Initial woking state
- Improve the function that merges 2 yaml structs by avoiding having to rewrite siblings

### Miscellaneous Tasks

- Release manta version 1.24.0
- Release manta version 1.24.0

## [1.23.0] - 2024-02-20

### Features

- New feature to use the SAT files as jinja2 templates (#37)
- New feature to use the SAT files as jinja2 templates (#37)
- New feature to use the SAT files as jinja2 templates (#37)
- New feature to use the SAT files as jinja2 templates (#37)
- Manuel Sopena Ballesteros <manuel.sopena@cscs.ch>
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.23.0

## [1.22.11] - 2024-02-20

### Bug Fixes

- Bug with manta panicking while creating a cluster if image creation fails
- Error when getting CFS session logs using a CFS session which does not exists
- Update mesa library

### Miscellaneous Tasks

- Release manta version 1.22.11

## [1.22.10] - 2024-02-17

### Bug Fixes

- Command update node fails if user is not restricted to any HSM groups
- Fix error parsing cli opn 'ansible-verbosity' to 'apply image' subcommand

### Miscellaneous Tasks

- Release manta version 1.22.10

## [1.22.9] - 2024-02-16

### Bug Fixes

- Show_config function breaks if the list of HSM groups the user has access to is empty

### Miscellaneous Tasks

- Release manta version 1.22.9

## [1.22.8] - 2024-02-15

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.22.8

## [1.22.7] - 2024-02-13

### Bug Fixes

- Fix stdout messages
- Manta crashes:wa when CFS configuration layer had no commit id

### Miscellaneous Tasks

- Release manta version 1.22.7

## [1.22.6] - 2024-02-13

### Bug Fixes

- Manta crashes:wa when CFS configuration layer had no commit id

### Miscellaneous Tasks

- Release manta version 1.22.6

## [1.22.5] - 2024-02-13

### Bug Fixes

- Manta crashes:wa when CFS configuration layer had no commit id

### Miscellaneous Tasks

- Release manta version 1.22.5

## [1.22.4] - 2024-02-13

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.22.4

## [1.22.3] - 2024-02-10

### Bug Fixes

- Mesa crate

### Miscellaneous Tasks

- Release manta version 1.22.3

## [1.22.2] - 2024-02-10

### Bug Fixes

- Apply cluster command failing if session_template section in SAT file was in old format

### Miscellaneous Tasks

- Release manta version 1.22.2

## [1.22.1] - 2024-02-10

### Miscellaneous Tasks

- Release manta version 1.22.1

### Refactor

- Clean code and stdout messages

## [1.22.0] - 2024-02-09

### Bug Fixes

- Delete data was not filtering BOS sessiontemplate properly
- Cray product catalogs when building images
- Manages git tags
- Create bos sessiontemplate from SAT file

### Features

- Apply image and apply cluster subcommands now manages IMS jobs through recipes and
- Apply configuration, apply image and apply cluster subcommands now
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.22.0

### Refactor

- Clean gitea code since it is moved to mesa
- Clean code
- Clean code
- Clean code
- Move code related to import data from SAT file to its own
- Module
- Add tests to import images in SAT file
- Create module for SAT code

## [1.21.3] - 2024-01-30

### Bug Fixes

- Get configuration command ignoring configurations related to CFS
- Sessions not completed
- Print statement
- Show error if apply cluster failt creating a configuration
- Bos sessiontemplate filter by list of xnames

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.21.3

## [1.21.2] - 2024-01-29

### Bug Fixes

- Format errors when deleting an image which does not exists

### Miscellaneous Tasks

- Release manta version 1.21.2

## [1.21.1] - 2024-01-29

### Bug Fixes

- Error when deleting an image based on a db recod but the artifact does not exists

### Miscellaneous Tasks

- Release manta version 1.21.1

## [1.21.0] - 2024-01-29

### Features

- Add new param to apply cluster to avoid nodes from rebooting

### Miscellaneous Tasks

- Release manta version 1.21.0

## [1.20.35] - 2024-01-28

### Bug Fixes

- Update mesa version in cargo.toml

### Miscellaneous Tasks

- Release manta version 1.20.35

## [1.20.34] - 2024-01-28

### Bug Fixes

- Rename 'force' param in 'delete' command to 'yes'
- Rename aliases for command 'apply configuration'
- Improve user validation to check access to a HSM group

### Fixes

- Migrate backup and migrate restore (#11)
- Migrate backup and migrate restore (#11)
- Migrate backup and migrate restore (#11)
- Migrate backup and migrate restore (#11)
- Migrate backup and migrate restore (#11)

### Miscellaneous Tasks

- Release manta version 1.20.34

### Refactor

- Code checkif user has access to HSM groups and members
- Fix some log messages

## [1.20.33] - 2024-01-27

### Bug Fixes

- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add releases for other OS
- Add releases for other OS
- Remove windows as a target

### Features

- Add openssl vendor feature to git2

### Miscellaneous Tasks

- Release manta version 1.20.7
- Release manta version 1.20.8
- Release manta version 1.20.9
- Release manta version 1.20.10
- Release manta version 1.20.11
- Release manta version 1.20.12
- Release manta version 1.20.13
- Release manta version 1.20.14
- Release manta version 1.20.15
- Release manta version 1.20.16
- Release manta version 1.20.17
- Release manta version 1.20.18
- Release manta version 1.20.19
- Release manta version 1.20.20
- Release manta version 1.20.21
- Release manta version 1.20.22
- Release manta version 1.20.23
- Release manta version 1.20.24
- Release manta version 1.20.25
- Release manta version 1.20.26
- Release manta version 1.20.27
- Release manta version 1.20.28
- Release manta version 1.20.29
- Release manta version 1.20.30
- Release manta version 1.20.31
- Release manta version 1.20.32
- Release manta version 1.20.33

## [1.20.6] - 2024-01-26

### Features

- Sort hsm available list in 'config show' command
- Add new target for mac, the idea is to have a new binary in github releases for mac users

### Miscellaneous Tasks

- Release manta version 1.20.6

## [1.20.5] - 2024-01-24

### Bug Fixes

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.20.5

## [1.20.4] - 2024-01-24

### Bug Fixes

- Update mesa version to fix a bug

### Miscellaneous Tasks

- Release manta version 1.20.4

## [1.20.3] - 2024-01-24

### Bug Fixes

- Bug in 'apply cluster' subcommand where it was filtering wrong images

### Miscellaneous Tasks

- Release manta version 1.20.3

## [1.20.2] - 2024-01-24

### Bug Fixes

- Add migrate subcommand
- Merge migration functionality

### Co-authored-by

- Masber <masber@hotmail.com>

### Miscellaneous Tasks

- Release manta version 1.20.1
- Release manta version 1.20.2

### Refactor

- Apply clippy suggestions

## [1.20.0] - 2024-01-22

### Bug Fixes

- Simplify the collection of the HSM group description data.
- Merge cluster migration branch
- Cli build code fix

### Feature

- Migrate/backup first commit (partial)
- Migrate/backup ignore JetBrains stuff
- Migrate/backup download all files of a bos session template
- Migrate/backup fix count of artifacts in download info
- Migrate/backup add support to produce a file with the list of xnames belonging to the HSM groups in the BOS session template.
- Migrate/backup cleanup
- Migrate/backup more cleanup
- Migrate/restore load backed files into memory

### Miscellaneous Tasks

- Release manta version 1.20.0

## [1.19.3] - 2024-01-21

### Miscellaneous Tasks

- Release manta version 1.19.3

### Refactor

- Clean code

## [1.19.2] - 2024-01-21

### Bug Fixes

- Rollback apply hw so it unpins all nodes in target hsm
- Fix issues related to add hw and remove hw subcommands

### Miscellaneous Tasks

- Release manta version 1.19.2

## [1.19.1] - 2024-01-19

### Bug Fixes

- Fix bug when creating clusters using sat file

### Features

- Add and remove nodes from HSM group
- Add functionality to add or remove nodes to/from an HSM group
- Return the HSM group json for API
- Add new mesa version

### Miscellaneous Tasks

- Release manta version 1.19.1

### Refactor

- Clean code related to subcommand 'apply hw'
- Clean code
- Add apply hw-configuration cli help message
- Clippy fixes
- Clippy fixes
- Clippy fixes
- Change apply/add/remove hw components to/from HSM group to

## [1.19.0] - 2024-01-15

### Features

- Apply hw partially working with first stage migrating hw components from target hsm group to parent, pending the other direction (migrating from parent to target hsm group)
- Apply hw partially working with first stage migrating hw components from target hsm group to parent, pending the other direction (migrating from parent to target hsm group)
- Initial code for apply hw subcommand keeping target hsm members
- Common to user request

### Miscellaneous Tasks

- Release manta version 1.19.0

## [1.18.0] - 2024-01-11

### Bug Fixes

- Disable tests which need to connect to csm apis becuase they are not accessible from github test environment

### Miscellaneous Tasks

- Release manta version 1.18.0

## [1.17.0] - 2024-01-11

### Bug Fixes

- Fix bug passing params to "remove hw" subcommand
- Params

### Miscellaneous Tasks

- Release manta version 1.17.0

### Refactor

- Normalize add, remove and apply hw component subcommands
- Clean code
- Get mesa from repo

## [1.16.2] - 2024-01-10

### Bug Fixes

- Apply session and update mesa library

### Miscellaneous Tasks

- Release manta version 1.16.2

## [1.16.1] - 2024-01-10

### Bug Fixes

- Clean obsolete "use" statements

### Features

- Remove hw components from a target hsm groups and node scores calculated based on scarcity across target and parent hsm groups
- Apply and remove working with simple examples, not fully tested but in good condition
- Add new hw components to a cluster

### Miscellaneous Tasks

- Release manta version 1.16.1

### Refactor

- Clean code
- Update README
- Refactor code

## [1.16.0] - 2024-01-04

### Bug Fixes

- Disable build script because it was breaking cli module load from
- Loading

### Features

- Start migrating hw components features from clstr crate
- Get hw cluster now accepts a new 'pattern' output

### Miscellaneous Tasks

- Release manta version 1.16.0

### Refactor

- : add clippy suggestions

### Testing

- Testing

## [1.15.0] - 2024-01-01

### Bug Fixes

- Get template not filtering by most recent value
- Replace std sleep for tokio sleep

### Features

- Add cluster power management

### Miscellaneous Tasks

- Release manta version 1.15.0

### Refactor

- Cargo fmt
- Use new manta utility functions
- Adapt to new mesa code
- Adapt to new mesa code
- Adopt mesa changes
- Move functions to print table data from mesa to manta
- Adapt to mesa code
- Housekeeping around HSM module
- Adapt to mesa code
- Adapt to new mesa code
- Adapt to new mesa code
- Adapt to new mesa code
- Swap to mesa library

## [1.14.0] - 2023-12-25

### Features

- Get nodes now shows CFS configuration related to image id used to
- Boot the node

### Miscellaneous Tasks

- Release manta version 1.14.0

### Refactor

- Clean code
- Adapt code to new mesa
- Clean code

## [1.13.5] - 2023-12-22

### Miscellaneous Tasks

- Release manta version 1.13.5

### Refactor

- Update mesa version

## [1.13.4] - 2023-12-22

### Bug Fixes

- Import

### Miscellaneous Tasks

- Release manta version 1.13.4

### Refactor

- Update mesa version

## [1.13.3] - 2023-12-21

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.13.3

### Refactor

- High refactoring
- Cfs configuration structs
- Rename method name to get multiple CFS components
- Hoursekeeping around node methods

## [1.13.2] - 2023-12-11

### Bug Fixes

- Fix Cargo.toml

### Miscellaneous Tasks

- Release manta version 1.13.2

### Refactor

- Clean code and update mesa version
- Cargo fmt

## [1.13.1] - 2023-12-10

### Miscellaneous Tasks

- Release manta version 1.13.1

### Refactor

- Refactor code to new mesa method signatures

## [1.13.0] - 2023-12-10

### Features

- Add new flag 'force' to  delete subcommand to make it script
- Friendly
- Delete subcommand sumamry shows more information

### Miscellaneous Tasks

- Release manta version 1.13.0

### Refactor

- Code in delete and get iamge subcommands

## [1.12.12] - 2023-12-08

### Miscellaneous Tasks

- Release manta version 1.12.12

### Refactor

- Adapt code to new mesa cfs config code structure

## [1.12.11] - 2023-12-08

### Miscellaneous Tasks

- Release manta version 1.12.11

### Refactor

- Adapt code to new mesa cfs config code structure

## [1.12.10] - 2023-12-08

### Bug Fixes

- Fix bug with get configuration subcommand

### Miscellaneous Tasks

- Release manta version 1.12.10

## [1.12.9] - 2023-12-07

### Bug Fixes

- Panic when trying to connect to CFS session (ansible) container

### Features

- Node power status methods blocks the runtime
- Each section in sat files processed individually

### Miscellaneous Tasks

- Release manta version 1.12.8
- Release manta version 1.12.9

### Refactor

- Clean code
- Configurations section in SAT file processed independently
- Images section in SAT file processed independently
- Session_templates section in SAT file processed independently

## [1.12.7] - 2023-11-20

### Bug Fixes

- Bug when cluster creation won't realised if a CFS session failed
- And keept waiting it to finish
- Rename CFS session and configuration table headers
- Fix hsm available list send to methods

### Miscellaneous Tasks

- Release manta version 1.12.7

## [1.12.6] - 2023-11-16

### Bug Fixes

- Bug getting cluster with nodes being configured

### Miscellaneous Tasks

- Release manta version 1.12.6

## [1.12.5] - 2023-11-16

### Bug Fixes

- Fix bug getting hsm group from cli param

### Miscellaneous Tasks

- Release manta version 1.12.5

## [1.12.4] - 2023-11-16

### Bug Fixes

- Fix 'get cluster status' sub command

### Miscellaneous Tasks

- Release manta version 1.12.4

### Refactor

- Fix 'get cluster' command help message typo
- Add deprecated message in 'get nodes' subcommand
- Fix 'get hsm' cli help message

## [1.12.3] - 2023-11-16

### Documentation

- Fix README

### Miscellaneous Tasks

- Release manta version 1.12.3

## [1.12.2] - 2023-11-16

### Documentation

- Update README

### Miscellaneous Tasks

- Release manta version 1.12.2

## [1.12.1] - 2023-11-16

### Features

- Duplicate get nodes subcommand into get cluster
- Get cluster status

### Miscellaneous Tasks

- Release manta version 1.12.1

## [1.12.0] - 2023-11-14

### Features

- Add new sub command apply configuration to create CFS
- Configuration from a SAT file

### Miscellaneous Tasks

- Release manta version 1.12.0

## [1.11.0] - 2023-11-13

### Bug Fixes

- Bugs with subcommands ignoring of failing the restrictions
- Regarding HSM available

### Features

- Adopt subcommands to work with a group of target HSM group
- New function to validate requested hsm group
- New function to validate requested hsm group members
- Update mesa version
- Get configuration subcommand output to json format

### Miscellaneous Tasks

- Release manta version 1.11.0

### Refactor

- Clean code and adapt to new mesa version
- Names
- Clean code

## [1.10.6] - 2023-11-10

### Features

- Enable logs while building a cluster
- Git-clone CFS session logs integrated to 'watch-log' command
- Parameter
- Feature was dropped in CSM 1.3)

### Miscellaneous Tasks

- Release manta version 1.10.6

### Refactor

- [**breaking**] Layer-id param removed when fetching ansible logs (this
- Code related to cfs session logs

## [1.10.5] - 2023-11-07

### Bug Fixes

- Fix bug where app did not read socks 5 information

### Miscellaneous Tasks

- Release manta version 1.10.5

## [1.10.4] - 2023-11-07

### Documentation

- Update README with instructions on how to create releases and commit messages best practices for CHANGELOG.md

### Miscellaneous Tasks

- Release manta version 1.10.4

## [1.10.3] - 2023-11-01

### Bug Fixes

- Update manta version
- Add CHANGELOG integration with cargo-release
- Add git cliff configuration to support multiline git commits

### Features

- Add subcommand to change log level

### Miscellaneous Tasks

- Release manta version 1.10.3

## [1.10.2] - 2023-10-31

### Miscellaneous Tasks

- Release manta version 1.10.2

### Refactor

- Refactor code
- Add new functionality to manage different sites
- Use new mesa library with higher level libraries for CSM integration

## [1.10.1] - 2023-10-26

### Bug Fixes

- Fix bug with 'config unset hsm' subcommand not deleting the config entry

### Miscellaneous Tasks

- Release manta version 1.10.1

## [1.10.0] - 2023-10-26

### Miscellaneous Tasks

- Release manta version 1.10.0

## [1.9.15] - 2023-10-26

### Miscellaneous Tasks

- Release manta version 1.9.15

## [1.9.14] - 2023-10-24

### Miscellaneous Tasks

- Release manta version 1.9.14

## [1.9.13] - 2023-10-24

### Bug Fixes

- Fix logging messages in update node subcommand

### Miscellaneous Tasks

- Release manta version 1.9.13

## [1.9.12] - 2023-10-21

### Bug Fixes

- Fix bug deleting elements

### Miscellaneous Tasks

- Release manta version 1.9.12

## [1.9.11] - 2023-10-21

### Bug Fixes

- Fix bug and integrate bos sessiontemplate with structs

### Miscellaneous Tasks

- Release manta version 1.9.11
- Release manta version 1.9.11

### Revert

- Revert version

## [1.9.10] - 2023-10-12

### Bug Fixes

- Fix code with clippy suggestions

### Miscellaneous Tasks

- Release manta version 1.9.10

## [1.9.9] - 2023-10-12

### Miscellaneous Tasks

- Release manta version 1.9.9

## [1.9.8] - 2023-10-12

### Bug Fixes

- Fix bugs deleting data and update mesa version

### Miscellaneous Tasks

- Release manta version 1.9.8

## [1.9.7] - 2023-10-11

### Miscellaneous Tasks

- Release manta version 1.9.7

## [1.9.6] - 2023-10-07

### Miscellaneous Tasks

- Release manta version 1.9.6

## [1.9.5] - 2023-10-05

### Bug Fixes

- Fix bug apply image and apply cluster failing if configuration section missing in sat file

### Miscellaneous Tasks

- Release manta version 0.7.0
- Release manta version 0.7.1
- Release manta version 0.7.2
- Release manta version 0.8.0
- Release manta version 0.8.1
- Release manta version 1.9.5

## [1.9.4] - 2023-10-04

### Miscellaneous Tasks

- Release manta version 1.9.4

## [1.9.3] - 2023-10-03

### Miscellaneous Tasks

- Release manta version 1.9.3

## [1.9.2] - 2023-09-30

### Miscellaneous Tasks

- Release manta version 1.9.2

## [1.9.1] - 2023-09-30

### Miscellaneous Tasks

- Release manta version 1.7.0
- Release manta version 1.8.0
- Release manta version 1.9.0
- Release manta version 1.9.1

## [1.6.0] - 2023-09-13

### Miscellaneous Tasks

- Release manta version 1.6.0

## [1.5.0] - 2023-09-12

### Miscellaneous Tasks

- Release manta version 1.5.0

## [1.4.0] - 2023-09-01

### Bug Fixes

- Fix bug apply image and apply cluster failing if configuration section missing in sat file
- Fix gub: update subcommand not taking the right image for a cfs session target image created by sat bootprep overwritting existing image
- Fix logging messages
- Fix bug creating image where image configuration name was not being updated if using a tag
- Fix typos in command line
- Fix logging messages
- Fix BSS from starting node configuration (ansible) before the node finish booting
- Fix cli command descriptions
- Fix bug with apply session subcommand when optional params were missing
- Fix clippy errors
- Fix bug apply image and apply cluster failing if configuration section missing in sat file

### Miscellaneous Tasks

- Release manta version 0.7.0
- Release manta version 0.7.1
- Release manta version 0.7.2
- Release manta version 0.8.0
- Release manta version 0.8.1
- Release manta version 0.8.2
- Release manta version 0.8.3
- Release manta version 0.8.4
- Release manta version 0.8.5
- Release manta version 0.8.6
- Release manta version 0.8.7
- Release manta version 0.8.8
- Release manta version 0.8.9
- Release manta version 1.0.0
- Release manta version 1.0.1
- Release manta version 1.0.2
- Release manta version 1.0.3
- Release manta version 1.0.4
- Release manta version 1.1.0
- Release manta version 1.1.1
- Release manta version 1.1.2
- Release manta version 1.2.0
- Release manta version 1.2.1
- Release manta version 1.2.2
- Release manta version 1.3.0
- Release manta version 1.3.1
- Release manta version 1.3.2
- Release manta version 1.3.3
- Release manta version 0.7.0
- Release manta version 0.7.1
- Release manta version 0.7.2
- Release manta version 0.8.0
- Release manta version 0.8.1
- Release manta version 1.4.0

### Refactor

- Refactor code to fetch image id from cfs session or cfs configuration name, also change cli commands help
- Refactor code to fetch image id from cfs session or cfs configuration name, also change cli commands help
- Refactor code and add validation when trying to access interactive session to a target node of a cfs session to build an image

## [0.6.30] - 2023-08-30

### Miscellaneous Tasks

- Release manta version 0.6.30

## [0.6.29] - 2023-07-18

### Bug Fixes

- Fix bug fetching commit details

### Miscellaneous Tasks

- Add new parameter to config file to specify the hashicorp vault environment to target to (shasta or prealps)
- Release manta version 0.6.29

## [0.6.28] - 2023-07-05

### Miscellaneous Tasks

- Release manta version 0.6.28

### Refactor

- Refactor code

## [0.6.27] - 2023-07-05

### Miscellaneous Tasks

- Remove unnecessary dependencies
- Release manta version 0.6.27

## [0.6.26] - 2023-06-29

### Miscellaneous Tasks

- Release manta version 0.6.26

## [0.6.25] - 2023-06-28

### Miscellaneous Tasks

- Release manta version 0.6.25

## [0.6.24] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.24

## [0.6.23] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.23

## [0.6.22] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.22

## [0.6.21] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.21

## [0.6.20] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.20

## [0.6.19] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.19

### Testing

- Test rust images in github workload

## [0.6.18] - 2023-06-27

### Miscellaneous Tasks

- Ci/cd pipeline build binaries using most recent rust compiler version
- Release manta version 0.6.18

## [0.6.17] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.17

## [0.6.16] - 2023-06-26

### Miscellaneous Tasks

- Release manta version 0.6.16

## [0.6.15] - 2023-06-25

### Bug Fixes

- Fix bug in cli 'apply image' and 'apply cluster'  subcommands parsing tag param

### Miscellaneous Tasks

- Add tag for sat file and mesa emancipation (#1)
- * remove shasta and manta modules and use the ones from mesa library
- * pending to get tested
- * update README and move mesa from local filesystem to crates.io
- * chore: Release manta version 0.6.14
- ---------
- Manuel Sopena Ballesteros <msopena@cscs.ch>
- Release manta version 0.6.15

## [0.6.13] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.13

### Testing

- Test merging github workflow files

## [0.6.12] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.12

### Testing

- Test merging github workflow files

## [0.6.11] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.11

## [0.6.10] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.10

## [0.6.9] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.9

## [0.6.8] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.8

## [0.6.7] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.7

## [0.6.6] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.6

## [0.6.5] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.5

## [0.6.4] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.4

## [0.6.3] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.3

## [0.6.2] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.2
- Release manta version 0.6.2

## [0.6.1] - 2023-06-21

### Miscellaneous Tasks

- Release manta version 0.6.1

## [0.6.0] - 2023-06-21

### Miscellaneous Tasks

- Release manta version 0.6.0

## [0.5.1] - 2023-06-21

### Bug Fixes

- Fix logs printing
- Fix bug fetching cfs session related to logs
- Fix logging
- Fix bug in keycloak authentication process
- Fix bug with authentication
- Add functionality to support config files
- Fix bug authenticating against keycloak
- Fix bug when trying to get the logs from a layer that does not exists
- Update README
- Update crates
- Fix typo in README
- Fix README and code organization improved by using rust modules
- Fix README and refactor some modules
- Fix typo in README
- Fix bug printing sessions on screen and getting members of a hsm group
- Fix cli programatically
- Fix cli args names errors
- Apply session checks if ansible-limit within hsm groups nodes
- Apply cfs session checks if any node in ansible-limit is part of a cfs session running or pending
- Fix clap config so get template/session/configuration can run without args
- Fix bugs in get session
- Fix bugs in get configuration
- Fix bugs
- Fix bug getting most recent configuration details
- Fix bug listing nodes in a hsm group
- Fix bug with manta not fetching the right gitea url to fetch commit details
- Fix cli options
- Address clippy issues
- Fix bug applying cfs session
- Fix cli help typos
- Fix log subcommand output
- Fix bug formatting hsm members
- Fix bug printing list of bos templates
- Fix README; fix app logging; fix bug when fetching commit details from gitea
- Fix bug fetching commit details from gitea
- Fix bug fetching gitea commit details when applaying a new session
- Fix message when getting nodes for a hsm group which does not exists
- Fix bug identifying image id for most recent configuration
- Fix bug not filtering cfs sessions based on hsm group
- Fix Dockerfile and fix bug reading cfs layers form sat file
- Fix bug where vault authentication module was expecting config file in /home/msopena/polybox/Documents/tests/rust/manta/config instead of /home/msopena/.config/manta/config
- Fix issues reported by clippy
- Fix bug which reads ~/.kube/config when creating a k8s client programatically if socks5 is disabled, this is wrong since k8s clients created programatically should not use kubeconfig file
- Fix bug: 'get nodes' crashes if no CFS session target image available in CSM
- Fix bug related to running sbatch and mpi job after the maintenance
- 03-23
- Update_node method now manages hsm groups
- Clean code
- Fix bug with update node and update hsm which did not use bod session template hence Boss Orchestrator Agent (BOA) was not telling CFS batcher to use the right CFS configuration
- Fix bug when get cfs sessions would show an image id different than the one
- Use by the bos sessicfs ontemplate for the same cfs configuration
- Rename variables for better readability
- Fix bug fetching cfs session logs if non utf chars were sent to client. Fixed by using utf8_lossy
- Fix code following clippy suggestions
- Fix bug related to 'get hsm' which was panicking by missusing unwrap()
- Fix bug getting information from HSM group
- Fix bug with get hsm-group command
- Fix cicd pipeline by installing rustfmt
- Fix apply session logging
- Fix gitlab pipeline typo
- Fix bug: get node subcomand failing with large hsm groups. For some reason CMS was crashing, the call to /cfs/v2/components has now broken down to multiple ones
- Fix bug: subcommand 'logs' failing to validate CFS session
- Fix Dockerfile with updated base image

### Miscellaneous Tasks

- Replacing format! macro with push_str
- Add cargo fmt to cicd pipeline
- Release manta version 0.5.1

### Refactor

- Refactor code to modules
- Refactor code; fix ownership issues; change config to use psitds hsm group
- Refactor var names and clean code
- Refactor get session cli
- Refactor code
- Refactor code
- Refactoring code
- Refactor code
- Refactor code
- Refactor code
- Refactor code
- Refactor code
- Refactor code
- Refactor code
- Refactor files to accomodate build.rs file
- New feature to generate bash autocomplete file upon compilation
- Refactor code to separate shata operations from printing results on screen
- Refactor code to separate shasta operations from printing results on screen
- Refactor code by moving method to fetches k8s secrets
- Refactor cfs session container logs functionality, it now returns a stream
- Refactor code to connect to node's console so it can be reused by other frontends (like cama)
- Refactor operations to handle console for cojin and cama
- Refactor code
- Refactor code

### Testing

- Testing git2-rs
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Testing git2-rs... commit created programatically...
- Test gitlab runner
- Test gitlab runner
- Test gitlab runner
- Test pipeline
- Test pipeline
- Test pipeline
- Test pipeline
- Test pipeline
- Test pipeline
- Test pipeline
- Test pipeline

<!-- generated by git-cliff -->
